use std::collections::HashMap;

use crate::ast::{BinOp, Expr, Pattern, Program, Stmt};
use crate::command::CommandSpec;
use crate::error::{
    EvalCommandFailedError, EvalCommandIoError, EvalError, EvalInvalidPipeRhsError,
    EvalIoRequiredError, EvalMatchExhaustedError, EvalPipeNotCallableError, EvalUnboundError,
    EvalUnknownBuiltinError,
};
use crate::runtime::run_command;
use crate::value::{UserFunction, Value};

pub type EvalEnv = HashMap<String, Value>;

fn is_pipe_prelude_target(name: &str) -> bool {
    matches!(name, "identity" | "id")
}

fn is_io_builtin(name: &str) -> bool {
    matches!(name, "run_text")
}

fn apply_function(
    env: &EvalEnv,
    func: &UserFunction,
    arg: Value,
    in_io: bool,
) -> Result<Value, EvalError> {
    let mut local = env.clone();
    local.insert(func.param.clone(), arg);
    eval_inner(&local, &func.body, in_io)
}

pub fn eval_expr(env: &EvalEnv, expr: &Expr) -> Result<Value, EvalError> {
    eval_inner(env, expr, false)
}

fn eval_inner(env: &EvalEnv, expr: &Expr, in_io: bool) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n, _) => Ok(Value::Int(*n)),
        Expr::String(s, _) => Ok(Value::String(s.clone())),
        Expr::Var(name, span) => env.get(name).cloned().ok_or_else(|| {
            EvalUnboundError {
                name: name.clone(),
                span: *span,
            }
            .into()
        }),
        Expr::List(items, _) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(eval_inner(env, item, in_io)?);
            }
            Ok(Value::List(out))
        }
        Expr::Io { body, .. } => eval_inner(env, body, true),
        Expr::Call {
            name,
            name_span,
            args,
            span,
        } => {
            if is_io_builtin(name) && !in_io {
                return Err(EvalIoRequiredError {
                    name: name.clone(),
                    span: *span,
                }
                .into());
            }
            eval_builtin_call(env, name, name_span, args, *span, in_io)
        }
        Expr::BinOp { op, lhs, rhs, .. } => {
            let l = eval_inner(env, lhs, in_io)?;
            let r = eval_inner(env, rhs, in_io)?;
            let result = match op {
                BinOp::Eq => l == r,
                BinOp::NotEq => l != r,
            };
            Ok(Value::Bool(result))
        }
        Expr::Match {
            subject,
            arms,
            span,
        } => {
            let val = eval_inner(env, subject, in_io)?;
            for arm in arms {
                if let Some(bindings) = try_match(&arm.pattern, &val) {
                    let mut local = env.clone();
                    local.extend(bindings);
                    if let Some(guard) = &arm.guard {
                        let cond = eval_inner(&local, guard, in_io)?;
                        if !is_truthy(&cond) {
                            continue;
                        }
                    }
                    return eval_inner(&local, &arm.body, in_io);
                }
            }
            Err(EvalMatchExhaustedError { span: *span }.into())
        }
        Expr::With {
            bindings,
            body,
            else_body,
            ..
        } => {
            let mut local = env.clone();
            for wb in bindings {
                let val = eval_inner(&local, &wb.expr, in_io)?;
                match try_match(&wb.pattern, &val) {
                    Some(new_bindings) => {
                        local.extend(new_bindings);
                    }
                    None => return eval_inner(env, else_body, in_io),
                }
            }
            eval_inner(&local, body, in_io)
        }
        Expr::Fn { param, body, .. } => Ok(Value::Function(UserFunction {
            param: param.clone(),
            body: (**body).clone(),
        })),
        Expr::Pipe(lhs, rhs, _) => {
            let left = eval_inner(env, lhs, in_io)?;
            match rhs.as_ref() {
                Expr::Var(name, span) => {
                    if let Some(bound) = env.get(name) {
                        match bound {
                            Value::Function(func) => apply_function(env, func, left, in_io),
                            _ => Err(EvalPipeNotCallableError {
                                name: name.clone(),
                                span: *span,
                            }
                            .into()),
                        }
                    } else if is_pipe_prelude_target(name) {
                        Ok(left)
                    } else {
                        Err(EvalUnboundError {
                            name: name.clone(),
                            span: *span,
                        }
                        .into())
                    }
                }
                _ => Err(EvalInvalidPipeRhsError { span: rhs.span() }.into()),
            }
        }
    }
}

fn eval_builtin_call(
    env: &EvalEnv,
    name: &str,
    name_span: &miette::SourceSpan,
    args: &[Expr],
    call_span: miette::SourceSpan,
    in_io: bool,
) -> Result<Value, EvalError> {
    match name {
        "run_text" => {
            if args.len() < 1 {
                return Err(EvalUnknownBuiltinError {
                    name: "run_text requires at least 1 argument (program)".to_string(),
                    span: call_span,
                }
                .into());
            }
            let program = match eval_inner(env, &args[0], in_io)? {
                Value::String(s) => s,
                _ => {
                    return Err(EvalUnknownBuiltinError {
                        name: "run_text first argument must be a string".to_string(),
                        span: args[0].span(),
                    }
                    .into())
                }
            };
            let mut cmd_args = Vec::new();
            for arg in &args[1..] {
                match eval_inner(env, arg, in_io)? {
                    Value::String(s) => cmd_args.push(s),
                    other => cmd_args.push(format!("{other}")),
                }
            }
            let spec = CommandSpec::new(&program, cmd_args);
            match run_command(&spec) {
                Ok(output) => {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(Value::String(text.trim_end_matches('\n').to_string()))
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let code = output.status.code().unwrap_or(-1);
                        let help_text = if stderr.is_empty() {
                            None
                        } else {
                            Some(format!("stderr: {stderr}"))
                        };
                        Err(EvalCommandFailedError {
                            program,
                            code,
                            stderr,
                            span: call_span,
                            help_text,
                        }
                        .into())
                    }
                }
                Err(io_err) => Err(EvalCommandIoError {
                    program,
                    reason: io_err.to_string(),
                    span: call_span,
                }
                .into()),
            }
        }
        _ => Err(EvalUnknownBuiltinError {
            name: name.to_string(),
            span: *name_span,
        }
        .into()),
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::String(s) => !s.is_empty(),
        Value::List(items) => !items.is_empty(),
        _ => true,
    }
}

fn try_match(pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
    match pattern {
        Pattern::Wildcard(_) => Some(vec![]),
        Pattern::Int(n, _) => match value {
            Value::Int(v) if v == n => Some(vec![]),
            _ => None,
        },
        Pattern::String(s, _) => match value {
            Value::String(v) if v == s => Some(vec![]),
            _ => None,
        },
        Pattern::Var(name, _) => Some(vec![(name.clone(), value.clone())]),
        Pattern::List(pats, _) => match value {
            Value::List(vals) if vals.len() == pats.len() => {
                let mut bindings = Vec::new();
                for (pat, val) in pats.iter().zip(vals.iter()) {
                    bindings.extend(try_match(pat, val)?);
                }
                Some(bindings)
            }
            _ => None,
        },
    }
}

pub fn eval_program(
    env: &mut EvalEnv,
    program: &Program,
) -> Result<Vec<(String, Value)>, EvalError> {
    let mut out = Vec::new();
    for stmt in &program.stmts {
        match stmt {
            Stmt::Let { name, expr, .. } => {
                let v = eval_expr(env, expr)?;
                env.insert(name.clone(), v.clone());
                out.push((name.clone(), v));
            }
            Stmt::Set { name, expr, .. } => {
                let v = eval_expr(env, expr)?;
                env.insert(name.clone(), v.clone());
                out.push((name.clone(), v));
            }
            Stmt::Expr(expr) => {
                let v = eval_expr(env, expr)?;
                out.push(("_".to_string(), v));
            }
        }
    }
    Ok(out)
}
