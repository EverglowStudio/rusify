use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::apple::apple_target::AppleTarget;
use crate::common::models::{Config, LibType, Mode};
use crate::console::step::run_step;

pub(crate) fn create_xcframework_with_output(
    targets: &[AppleTarget],
    lib_name: &str,
    package_name: &str,
    xcframework_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
) -> Result<()> {
    run_step(config, "Creating XCFramework...", || {
        // TODO: show command spinner here with xcbuild command
        let output_dir = PathBuf::from(package_name);
        // TODO: make this configurable
        let generated_dir = PathBuf::from("./generated");

        create_xcframework(
            targets,
            lib_name,
            xcframework_name,
            &generated_dir,
            &output_dir,
            mode,
            lib_type,
        )
    })
    .map_err(|e| {
        anyhow::anyhow!(
            "Failed to create XCFramework due to the following error: \n {}",
            e
        )
    })
}

pub fn create_xcframework(
    targets: &[AppleTarget],
    lib_name: &str,
    xcframework_name: &str,
    generated_dir: &Path,
    output_dir: &Path,
    mode: Mode,
    lib_type: LibType,
) -> Result<()> {
    let libs: Vec<_> = targets
        .iter()
        .map(|t| t.library_path(lib_name, mode, lib_type))
        .collect();

    let headers = generated_dir.join("headers");
    let headers = headers
        .to_str()
        .context("Directory for bindings has an invalid name")?;

    let output_dir_name = &output_dir
        .to_str()
        .context("Output directory has an invalid name")?;

    let framework = format!("{output_dir_name}/{xcframework_name}.xcframework");

    let mut xcodebuild = Command::new("xcodebuild");
    xcodebuild.arg("-create-xcframework");

    for lib in &libs {
        xcodebuild.arg("-library");
        xcodebuild.arg(lib);
        xcodebuild.arg("-headers");
        xcodebuild.arg(headers);
    }

    let output = xcodebuild
        .arg("-output")
        .arg(&framework)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute xcodebuild command")?;

    if !output.status.success() {
        anyhow::bail!(
            "xcodebuild command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    } else {
        Ok(())
    }
}
