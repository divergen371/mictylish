use std::collections::HashMap;

use crate::error::NameError;
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
}
