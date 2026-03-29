use std::collections::HashMap;

use crate::ast::{Expr, Pattern, Program, Stmt};
use crate::error::{
    EvalError, EvalInvalidPipeRhsError, EvalMatchExhaustedError, EvalPipeNotCallableError,
    EvalUnboundError,
};
use crate::value::{UserFunction, Value};

pub type EvalEnv = HashMap<String, Value>;

fn is_pipe_prelude_target(name: &str) -> bool {
    matches!(name, "identity" | "id")
}

fn apply_function(env: &EvalEnv, func: &UserFunction, arg: Value) -> Result<Value, EvalError> {
    let mut local = env.clone();
    local.insert(func.param.clone(), arg);
    eval_expr(&local, &func.body)
}

pub fn eval_expr(env: &EvalEnv, expr: &Expr) -> Result<Value, EvalError> {
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
                out.push(eval_expr(env, item)?);
            }
            Ok(Value::List(out))
        }
        Expr::Match {
            subject,
            arms,
            span,
        } => {
            let val = eval_expr(env, subject)?;
            for arm in arms {
                if let Some(bindings) = try_match(&arm.pattern, &val) {
                    let mut local = env.clone();
                    local.extend(bindings);
                    return eval_expr(&local, &arm.body);
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
                let val = eval_expr(&local, &wb.expr)?;
                match try_match(&wb.pattern, &val) {
                    Some(new_bindings) => {
                        local.extend(new_bindings);
                    }
                    None => return eval_expr(env, else_body),
                }
            }
            eval_expr(&local, body)
        }
        Expr::Fn {
            param, body, ..
        } => Ok(Value::Function(UserFunction {
            param: param.clone(),
            body: (**body).clone(),
        })),
        Expr::Pipe(lhs, rhs, _) => {
            let left = eval_expr(env, lhs)?;
            match rhs.as_ref() {
                Expr::Var(name, span) => {
                    if let Some(bound) = env.get(name) {
                        match bound {
                            Value::Function(func) => apply_function(env, func, left),
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

/// Runs side effects of each statement and extends `env`. Returns `(name, value)` for each `let`.
pub fn eval_program(env: &mut EvalEnv, program: &Program) -> Result<Vec<(String, Value)>, EvalError> {
    let mut out = Vec::new();
    for stmt in &program.stmts {
        match stmt {
            Stmt::Let {
                name,
                expr,
                ..
            } => {
                let v = eval_expr(env, expr)?;
                env.insert(name.clone(), v.clone());
                out.push((name.clone(), v));
            }
        }
    }
    Ok(out)
}
