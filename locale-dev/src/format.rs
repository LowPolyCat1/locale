use crate::error::{Error, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// Runs rustfmt over the generated files so they match `cargo fmt`, one
/// process per file, all at once.
pub fn format_generated_code(files: &[PathBuf]) -> Result<()> {
    tracing::info!("Formatting {} generated files...", files.len());
    let children = files
        .iter()
        .map(|file| {
            Command::new("rustfmt")
                .args(["--edition", "2024"])
                .arg(file)
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| Error::Rustfmt(format!("could not run rustfmt: {e}")))
        })
        .collect::<Result<Vec<Child>>>()?;

    // Wait for every process before reporting, so none is left running.
    let mut errors = Vec::new();
    for child in children {
        let output = child.wait_with_output()?;
        if !output.status.success() {
            errors.push(String::from_utf8_lossy(&output.stderr).into_owned());
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::Rustfmt(errors.join("\n")))
    }
}
