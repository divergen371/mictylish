use miette::{IntoDiagnostic, Result};

pub fn glob(pattern: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for entry in glob::glob(pattern).into_diagnostic()? {
        let path = entry.into_diagnostic()?;
        out.push(path.to_string_lossy().to_string());
    }
    Ok(out)
}
