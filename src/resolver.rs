use std::collections::HashMap;

use crate::ast::{Expr, Program, Stmt};
use crate::error::{InvalidPipeRhsError, NameError, ResolveError, UndefinedNameError};
use crate::span::Span;

#[derive(Debug, Default)]
pub struct Resolver {
    scopes: Vec<HashMap<String, Span>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: impl Into<String>, span: Span) -> Result<(), NameError> {
        let name = name.into();
        if let Some(first) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&name))
        {
            return Err(NameError {
                name,
                first: first.clone(),
                second: span,
            });
        }
        if let Some(current) = self.scopes.last_mut() {
            current.insert(name, span);
        }
        Ok(())
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains_key(name))
    }

    fn is_pipe_prelude_target(name: &str) -> bool {
        matches!(name, "identity" | "id")
    }

    fn check_pipe_rhs(&self, expr: &Expr) -> Result<(), ResolveError> {
        match expr {
            Expr::Var(name, span) => {
                if Self::is_pipe_prelude_target(name) || self.is_defined(name) {
                    Ok(())
                } else {
                    Err(UndefinedNameError {
                        name: name.clone(),
                        span: *span,
                    }
                    .into())
                }
            }
            other => Err(InvalidPipeRhsError { span: other.span() }.into()),
        }
    }

    fn check_fn_expr(
        &mut self,
        param: &str,
        param_span: Span,
        body: &Expr,
    ) -> Result<(), ResolveError> {
        self.push_scope();
        let result = (|| -> Result<(), ResolveError> {
            self.define(param.to_string(), param_span)?;
            self.check_expr(body)?;
            Ok(())
        })();
        self.pop_scope();
        result
    }

    /// Checks that every [`Expr::Var`] refers to a name already in scope.
    /// Pipeline RHS allows `identity` / `id` without a prior `let`.
    pub fn check_expr(&mut self, expr: &Expr) -> Result<(), ResolveError> {
        match expr {
            Expr::String(_, _) | Expr::Int(_, _) => Ok(()),
            Expr::Var(name, span) => {
                if self.is_defined(name) {
                    Ok(())
                } else {
                    Err(UndefinedNameError {
                        name: name.clone(),
                        span: *span,
                    }
                    .into())
                }
            }
            Expr::List(items, _) => {
                for item in items {
                    self.check_expr(item)?;
                }
                Ok(())
            }
            Expr::Fn {
                param,
                param_span,
                body,
                ..
            } => self.check_fn_expr(param, *param_span, body),
            Expr::Pipe(lhs, rhs, _) => {
                self.check_expr(lhs)?;
                self.check_pipe_rhs(rhs)?;
                Ok(())
            }
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), ResolveError> {
        match stmt {
            Stmt::Let {
                name,
                name_span,
                expr,
                ..
            } => {
                self.check_expr(expr)?;
                self.define(name.clone(), *name_span)?;
                Ok(())
            }
        }
    }

    /// Resolves a program in order: each `let` RHS may only use names bound earlier
    /// in the same program or in outer scopes (e.g. REPL session bindings).
    pub fn resolve_program(&mut self, program: &Program) -> Result<(), ResolveError> {
        for stmt in &program.stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }
}
