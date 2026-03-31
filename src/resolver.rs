use std::collections::HashMap;

use crate::ast::{Expr, MatchArm, Pattern, Program, Stmt, WithBinding};
use crate::error::{
    InvalidPipeRhsError, NameError, ResolveError, SetNotMutableError, SetUndefinedError,
    UndefinedNameError,
};
use crate::span::Span;

#[derive(Debug, Clone)]
struct Binding {
    span: Span,
    mutable: bool,
}

#[derive(Debug, Default)]
pub struct Resolver {
    scopes: Vec<HashMap<String, Binding>>,
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
        self.define_with_mutability(name, span, false)
    }

    pub fn define_mut(&mut self, name: impl Into<String>, span: Span) -> Result<(), NameError> {
        self.define_with_mutability(name, span, true)
    }

    fn define_with_mutability(
        &mut self,
        name: impl Into<String>,
        span: Span,
        mutable: bool,
    ) -> Result<(), NameError> {
        let name = name.into();
        if let Some(first) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&name))
        {
            return Err(NameError {
                name,
                first: first.span,
                second: span,
            });
        }
        if let Some(current) = self.scopes.last_mut() {
            current.insert(name, Binding { span, mutable });
        }
        Ok(())
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains_key(name))
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
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
            Expr::Io { body, .. } => self.check_expr(body),
            Expr::Call { args, .. } => {
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(())
            }
            Expr::Match { subject, arms, .. } => {
                self.check_expr(subject)?;
                for arm in arms {
                    self.check_match_arm(arm)?;
                }
                Ok(())
            }
            Expr::With {
                bindings,
                body,
                else_body,
                ..
            } => self.check_with_expr(bindings, body, else_body),
            Expr::Pipe(lhs, rhs, _) => {
                self.check_expr(lhs)?;
                self.check_pipe_rhs(rhs)?;
                Ok(())
            }
        }
    }

    fn check_match_arm(&mut self, arm: &MatchArm) -> Result<(), ResolveError> {
        self.push_scope();
        let result = (|| -> Result<(), ResolveError> {
            self.define_pattern_bindings(&arm.pattern)?;
            self.check_expr(&arm.body)?;
            Ok(())
        })();
        self.pop_scope();
        result
    }

    fn define_pattern_bindings(&mut self, pat: &Pattern) -> Result<(), ResolveError> {
        match pat {
            Pattern::Wildcard(_) | Pattern::Int(_, _) | Pattern::String(_, _) => Ok(()),
            Pattern::Var(name, span) => {
                self.define(name.clone(), *span)?;
                Ok(())
            }
            Pattern::List(items, _) => {
                for item in items {
                    self.define_pattern_bindings(item)?;
                }
                Ok(())
            }
        }
    }

    fn check_with_expr(
        &mut self,
        bindings: &[WithBinding],
        body: &Expr,
        else_body: &Expr,
    ) -> Result<(), ResolveError> {
        self.push_scope();
        let result = (|| -> Result<(), ResolveError> {
            for wb in bindings {
                self.check_expr(&wb.expr)?;
                self.define_pattern_bindings(&wb.pattern)?;
            }
            self.check_expr(body)?;
            Ok(())
        })();
        self.pop_scope();
        result?;
        self.check_expr(else_body)?;
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), ResolveError> {
        match stmt {
            Stmt::Let {
                name,
                name_span,
                mutable,
                expr,
                ..
            } => {
                self.check_expr(expr)?;
                if *mutable {
                    self.define_mut(name.clone(), *name_span)?;
                } else {
                    self.define(name.clone(), *name_span)?;
                }
                Ok(())
            }
            Stmt::Set {
                name,
                name_span,
                expr,
                ..
            } => {
                self.check_expr(expr)?;
                match self.lookup(name) {
                    Some(binding) if binding.mutable => Ok(()),
                    Some(_) => Err(SetNotMutableError {
                        name: name.clone(),
                        span: *name_span,
                    }
                    .into()),
                    None => Err(SetUndefinedError {
                        name: name.clone(),
                        span: *name_span,
                    }
                    .into()),
                }
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
