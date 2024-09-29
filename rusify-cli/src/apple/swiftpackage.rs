use std::ops::Not;
use askama::Template;
use glob::glob;
use std::fs::{copy, create_dir_all, write};
use crate::{Context, Result, path::recreate_dir, templating};
use crate::common::models::Config;
use crate::console::step::run_step;
use crate::console::{MainSpinner, Ticking};

pub(crate) fn create_package_with_output(
    package_name: &str,
    xcframework_name: &str,
    disable_warnings: bool,
    config: &Config,
) -> Result<()> {
    run_step(
        config,
        format!("Creating Swift Package '{package_name}'..."),
        || create_swiftpackage(package_name, xcframework_name, disable_warnings),
    )?;

    let spinner = config.silent.not().then(|| {
        MainSpinner::with_message(format!(
            "Successfully created Swift Package in '{package_name}/'!"
        ))
    });
    spinner.finish();

    Ok(())
}

/// Create artifacts for a swift package given the package name
///
/// **Note**: This method assumes that a directory with the package name and the .xcframework already exists
pub fn create_swiftpackage(
    package_name: &str,
    xcframework_name: &str,
    disable_warnings: bool,
) -> Result<()> {
    let package_manifest = templating::PackageSwift {
        package_name,
        xcframework_name,
        disable_warnings,
    };

    write(
        format!("{}/Package.swift", package_name),
        package_manifest.render().context("Failed to render Package.swift template")?
    ).context("Could not write Package.swift")?;

    create_dir_all(format!("{}/Sources/{}", package_name, package_name))
        .context("Could not create module sources directory")?;

    for swift_file in glob("./generated/sources/*.swift")
        .context("Could not find generated swift source files")?
    {
        let swift_file = swift_file
            .context("Could not access generated swift source file")?;
        let file_name = swift_file
            .file_name()
            .context("Could not get file name")?
            .to_str()
            .context("Could not convert file name to string")?
            .to_string();
        copy(
            swift_file,
            format!("{}/Sources/{}/{}", package_name, package_name, file_name),
        )
        .context("Could not copy generated swift source files")?;
    }

    Ok(())
}

pub fn recreate_output_dir(package_name: &str) -> Result<()> {
    let dir = format!("./{package_name}");
    recreate_dir(dir)
}
