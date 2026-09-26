use crate::error::{Error, Result};
use std::path::PathBuf;
use std::process::Command;

/// Runs rustfmt over the generated files so they match `cargo fmt`.
pub fn format_generated_code(files: &[PathBuf]) -> Result<()> {
    tracing::info!("Formatting {} generated files...", files.len());
    let output = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .args(files)
        .output()
        .map_err(|e| Error::Rustfmt(format!("could not run rustfmt: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::Rustfmt(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}
