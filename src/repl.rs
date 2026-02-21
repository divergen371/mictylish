use miette::{IntoDiagnostic, Result};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub async fn run() -> Result<()> {
    let mut rl = DefaultEditor::new().into_diagnostic()?;
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
                println!("{input}");
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err).into_diagnostic(),
        }
    }
    Ok(())
}
