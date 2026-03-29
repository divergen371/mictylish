use miette::{IntoDiagnostic, NamedSource, Report, Result};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::eval::{eval_program, EvalEnv};
use crate::parser::parse_program;
use crate::resolver::Resolver;

pub async fn run() -> Result<()> {
    let mut rl = DefaultEditor::new().into_diagnostic()?;
    let mut resolver = Resolver::new();
    let mut env: EvalEnv = EvalEnv::new();
    loop {
        match rl.readline("mictylish> ") {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                if input == ":q" || input == ":quit" || input == "exit" {
                    break;
                }
                let _ = rl.add_history_entry(input);
                match parse_program(input) {
                    Ok(program) => {
                        if let Err(err) = resolver.resolve_program(&program) {
                            eprintln!(
                                "{:?}",
                                Report::new(err)
                                    .with_source_code(NamedSource::new("repl", input.to_string()))
                            );
                        } else {
                            match eval_program(&mut env, &program) {
                                Ok(bindings) => {
                                    for (name, value) in bindings {
                                        println!("{name} = {value}");
                                    }
                                }
                                Err(err) => {
                                    eprintln!(
                                        "{:?}",
                                        Report::new(err).with_source_code(NamedSource::new(
                                            "repl",
                                            input.to_string(),
                                        ))
                                    );
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "{:?}",
                            Report::new(err)
                                .with_source_code(NamedSource::new("repl", input.to_string()))
                        );
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err).into_diagnostic(),
        }
    }
    Ok(())
}
