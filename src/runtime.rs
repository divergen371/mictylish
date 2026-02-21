use std::process::{Command, Output};

use crate::command::CommandSpec;

pub fn run_command(spec: &CommandSpec) -> std::io::Result<Output> {
    Command::new(spec.program()).args(spec.args()).output()
}
