use anyhow::{anyhow, Context, Result};
use cargo_metadata::Package;
// use convert_case::{Case, Casing};
use dialoguer::{Input, MultiSelect};

use crate::android::android_target::{AndroidArch, AndroidTarget};
use crate::common::{
    metadata::{metadata, MetadataExt},
    models::{Config, FeatureOptions, LibType, Mode},
};
use crate::console::{messages::*, step::run_step_with_commands, theme::prompt_theme};
use crate::ffi::kotlin::generate_kotlin_bindings_with_output;
use crate::android::aar::create_aar_with_output;

#[allow(clippy::too_many_arguments)]
pub fn build_android_package(
    architectures: Option<Vec<AndroidArch>>,
    build_target: Option<&str>,
    api_level: u32,
    package_name: Option<String>,
    aar_name: String,
    config: Config,
    mode: Mode,
    lib_type: LibType,
    features: FeatureOptions,
) -> Result<()> {
    // Get the current crate
    let crates = [metadata()
        .current_crate()
        .context("Current directory is not part of a crate!")?];

    if crates.len() == 1 {
        return build_android_package_for_crate(
            crates[0],
            architectures.clone(),
            build_target,
            api_level,
            package_name,
            aar_name,
            &config,
            mode,
            lib_type,
            features,
        );
    } else if package_name.is_some() {
        return Err(anyhow!(
            "Package name can only be specified when building a single crate!"
        ));
    }

    crates
        .iter()
        .map(|current_crate| {
            info!(&config, "Packaging crate {}", current_crate.name);
            build_android_package_for_crate(
                current_crate,
                architectures.clone(),
                build_target,
                api_level,
                None,
                aar_name.clone(),
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
fn build_android_package_for_crate(
    current_crate: &Package,
    architectures: Option<Vec<AndroidArch>>,
    build_target: Option<&str>,
    api_level: u32,
    package_name: Option<String>,
    aar_name: String,
    config: &Config,
    mode: Mode,
    lib_type: LibType,
    features: FeatureOptions,
) -> Result<()> {
    // Verify that ANDROID_NDK_HOME is properly set
    if let Err(e) = std::env::var("ANDROID_NDK_HOME") {
        return Err(anyhow!(
            "ANDROID_NDK_HOME environment variable is not set. Please install Android NDK and set this variable: {}",
            e
        ));
    }
    let lib = current_crate
        .targets
        .iter()
        .find(|t| t.kind.contains(&"lib".to_owned()))
        .context("No library tag defined in Cargo.toml!")?;

    let crate_name = current_crate.name.to_lowercase();
    let default_package_name = format!("com.{}.{}", crate_name, crate_name);
    let package_name = package_name.unwrap_or_else(|| 
        prompt_package_name(&default_package_name, config.accept_all));

    let architectures = architectures.unwrap_or_else(|| prompt_architectures(config.accept_all));

    if architectures.is_empty() {
        return Err(anyhow!("At least 1 architecture needs to be selected!"));
    }

    if lib_type == LibType::Static {
        warning!(
            &config,
            "Building as static library for Android is unusual. Dynamic libraries (.so) are typically used."
        );
    }

    let mut targets: Vec<_> = architectures
        .into_iter()
        .map(|a| a.target(api_level))
        .collect();

    if let Some(build_target) = build_target {
        targets.retain_mut(|android_target| match android_target {
            AndroidTarget { architectures, .. } => {
                architectures.iter().any(|arch| *arch == build_target)
            }
        });
        if targets.is_empty() {
            return Err(anyhow!("No matching build target for {}", build_target));
        }
    }

    let crate_name = lib.name.replace('-', "_");
    for target in &targets {
        build_with_output(target, &crate_name, mode, lib_type, config, &features)?;
    }

    generate_kotlin_bindings_with_output(&targets, &crate_name, mode, lib_type, config, &package_name)?;

    create_aar_with_output(
        &targets,
        &crate_name,
        &package_name,
        &aar_name,
        mode,
        lib_type,
        config,
    )?;

    Ok(())
}

fn prompt_architectures(accept_all: bool) -> Vec<AndroidArch> {
    let architectures = AndroidArch::all();
    let items = architectures.map(|a| a.display_name());

    if accept_all {
        return architectures.to_vec();
    }

    let theme = prompt_theme();
    let selector = MultiSelect::with_theme(&theme)
        .items(&items)
        .with_prompt("Select Target Architectures")
        .defaults(&[true, true, false, true]); // Default to ARM64, ARMv7, and x86_64

    let chosen: Vec<usize> = selector.interact().unwrap();

    chosen.into_iter().map(|i| architectures[i]).collect()
}

fn prompt_package_name(default: &str, accept_all: bool) -> String {
    if accept_all {
        return default.to_string();
    }

    let theme = prompt_theme();
    Input::with_theme(&theme)
        .with_prompt("Kotlin Package Name")
        .default(default.to_string())
        .interact_text()
        .unwrap()
}

fn build_with_output(
    target: &AndroidTarget,
    _lib_name: &str,
    mode: Mode,
    _lib_type: LibType,
    config: &Config,
    features: &FeatureOptions,
) -> Result<()> {
    // First set up the environment with Rust file operations
    target.setup_environment()
        .context(format!("Failed to set up build environment for {}", target.display_name()))?;
    
    // Then run cargo build commands
    let mut commands = target.cargo_build_commands(mode, features);
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