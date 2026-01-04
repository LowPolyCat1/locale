use std::process::Command;

pub fn format_generated_code() {
    tracing::info!("Refining generated code in locale-rs...");

    let fmt_status = Command::new("cargo")
        .arg("fmt")
        .arg("-p")
        .arg("locale-rs")
        .status()
        .expect("Failed to execute cargo fmt");

    if fmt_status.success() {
        tracing::info!("Successfully formatted locale-rs.");
    } else {
        panic!("Cargo fmt encountered errors.");
    }

    let clippy_status = Command::new("cargo")
        .arg("clippy")
        .arg("-p")
        .arg("locale-rs")
        .arg("--fix")
        .arg("--allow-dirty")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .status()
        .expect("Failed to execute cargo clippy");

    if clippy_status.success() {
        tracing::info!("Clippy checks passed/fixed for locale-rs.");
    } else {
        tracing::error!("Clippy found issues that require manual attention.");
    }
}
