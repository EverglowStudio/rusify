use anyhow::{anyhow, Context, Result};
use cargo_metadata::Package;
use convert_case::{Case, Casing};
use dialoguer::{Input, MultiSelect};

use crate::apple::apple_target::{ApplePlatform, AppleTarget};
use crate::common::{
    metadata::{metadata, MetadataExt},
    models::{Config, FeatureOptions, LibType, Mode},
};
use crate::console::{
    messages::*,
    step::run_step_with_commands,
    theme::prompt_theme,
};
use crate::ffi::swift::generate_swift_bindings_with_output;
use crate::swiftpackage::{create_package_with_output, recreate_output_dir};
use crate::xcframework::create_xcframework_with_output;


#[allow(clippy::too_many_arguments)]
pub fn build_swift_package(
    platforms: Option<Vec<ApplePlatform>>,
    build_target: Option<&str>,
    package_name: Option<String>,
    xcframework_name: String,
    disable_warnings: bool,
    config: Config,
    mode: Mode,
    lib_type: LibType,
    features: FeatureOptions,
) -> Result<()> {
    // TODO: Allow path as optional argument to take other directories than current directory
    // let crates = metadata().uniffi_crates();
    let crates = [metadata()
        .current_crate()
        .context("Current directory is not part of a crate!")?];

    if crates.len() == 1 {
        return build_swift_package_for_crate(
            crates[0],
            platforms.clone(),
            build_target,
            package_name,
            xcframework_name,
            disable_warnings,
            &config,
            mode,
            lib_type,
            features,
        );
    } else if package_name.is_some() {
        return Err(anyhow!("Package name can only be specified when building a single crate!"));
    }

    crates
        .iter()
        .map(|current_crate| {
            info!(&config, "Packaging crate {}", current_crate.name);
            build_swift_package_for_crate(
                current_crate,
                platforms.clone(),
                build_target,
                None,
                xcframework_name.clone(),
                disable_warnings,
                &config,
                mode,
                lib_type.clone(),
                features.clone(),
            )
        })
        .filter_map(|result| result.err())
        .collect::<Vec<_>>()
        .into_iter()
        .fold(Ok(()), |acc, err| {
            if let Err(e) = acc {
                Err(e.context(err))
            } else {
                Err(err)
            }
        })
}

#[allow(clippy::too_many_arguments)]
fn build_swift_package_for_crate(
    current_crate: &Package,
    platforms: Option<Vec<ApplePlatform>>,
    build_target: Option<&str>,
    package_name: Option<String>,
    xcframework_name: String,
    disable_warnings: bool,
    config: &Config,
    mode: Mode,
    lib_type: LibType,
    features: FeatureOptions,
) -> Result<()> {
    let lib = current_crate
        .targets
        .iter()
        .find(|t| t.kind.contains(&"lib".to_owned()))
        .context("No library tag defined in Cargo.toml!")?;

    let crate_name = current_crate.name.to_lowercase();
    let package_name =
        package_name.unwrap_or_else(|| prompt_package_name(&crate_name, config.accept_all));

    let platforms = platforms.unwrap_or_else(|| prompt_platforms(config.accept_all));

    if platforms.is_empty() {
        return Err(anyhow!("At least 1 platform needs to be selected!"));
    }

    if lib_type == LibType::Dynamic {
        warning!(
            &config,
            "Building as dynamic library is discouraged. It might prevent apps that use this library from publishing to the App Store."
        );
    }

    let mut targets: Vec<_> = platforms
        .into_iter()
        .flat_map(|p| p.into_apple_platform_target())
        .map(|p| p.target())
        .collect();

    if let Some(build_target) = build_target {
        targets.retain_mut(|platform_target| match platform_target {
            AppleTarget { architectures, .. } => architectures.iter().any(|arch| *arch == build_target),
        });
        if targets.is_empty() {
            return Err(anyhow!("No matching build target for {}", build_target));
        }
    }

    let crate_name = lib.name.replace('-', "_");
    for target in &targets {
        build_with_output(target, &crate_name, mode, lib_type, config, &features)?;
    }

    generate_swift_bindings_with_output(&targets, &crate_name, mode, lib_type, config)?;

    recreate_output_dir(&package_name).context("Could not create package output directory!")?;
    create_xcframework_with_output(
        &targets,
        &crate_name,
        &package_name,
        &xcframework_name,
        mode,
        lib_type,
        config,
    )?;
    create_package_with_output(&package_name, &xcframework_name, disable_warnings, config)?;

    Ok(())
}

fn prompt_platforms(accept_all: bool) -> Vec<ApplePlatform> {
    let platforms = ApplePlatform::all();
    let items = platforms.map(|p| p.display_name());

    if accept_all {
        return platforms.to_vec();
    }

    let theme = prompt_theme();
    let selector = MultiSelect::with_theme(&theme)
        .items(&items)
        .with_prompt("Select Target Platforms")
        // TODO: Move this to separate class and disable reporting to change style on success
        // .report(false)
        .defaults(&platforms.map(|p| !p.is_experimental()));

    let chosen: Vec<usize> = selector.interact().unwrap();

    chosen.into_iter().map(|i| platforms[i]).collect()
}

fn prompt_package_name(crate_name: &str, accept_all: bool) -> String {
    let default = crate_name.to_case(Case::UpperCamel);

    if accept_all {
        return default;
    }

    let theme = prompt_theme();
    Input::with_theme(&theme)
        .with_prompt("Swift Package Name")
        .default(default)
        .interact_text()
        .unwrap()
}

fn build_with_output(
    target: &AppleTarget,
    lib_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
    features: &FeatureOptions,
) -> Result<()> {
    let mut commands = target.commands(lib_name, mode, lib_type, features);
    for command in &mut commands {
        command.env("CARGO_TERM_COLOR", "always");
    }

    run_step_with_commands(
        config,
        format!("Building target {}", target.display_name()),
        &mut commands,
    )?;

    Ok(())
}