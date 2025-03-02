use std::fs::{self, create_dir};
use std::path::Path;

use camino::{Utf8Path, Utf8PathBuf};
use uniffi_bindgen::{
    bindings::KotlinBindingGenerator,
    cargo_metadata::CrateConfigSupplier,
};

use crate::{
    android::android_target::{AndroidTarget, library_file_name},
    common::{
        metadata::MetadataExt,
        models::{Config, LibType, Mode},
        path::recreate_dir,
    },
    console::step::run_step,
    metadata::metadata,
    Result,
};

pub(crate) fn generate_kotlin_bindings_with_output(
    targets: &[AndroidTarget],
    lib_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
    package_name: &str,
) -> Result<()> {
    run_step(config, "Generating Kotlin bindings...", || {
        let lib_file = library_file_name(lib_name, lib_type);
        let target = metadata().target_dir();
        let archs = targets
            .first()
            .ok_or_else(|| anyhow::anyhow!("Could not generate UniFFI bindings: No target architecture selected!"))?
            .architectures();
        let arch = archs.first().ok_or_else(|| anyhow::anyhow!("No architectures found for the selected target"))?;
        let lib_path: Utf8PathBuf = format!("{}/{}/{}/{}", target, arch, mode, lib_file).into();

        generate_kotlin_bindings(&lib_path, package_name)
            .map_err(|e| anyhow::anyhow!("Could not generate UniFFI bindings for udl files due to the following error: \n {e}"))
    })
}

pub fn generate_kotlin_bindings(lib_path: &Utf8Path, package_name: &str) -> Result<()> {
    let out_dir = Utf8Path::new("./generated");
    let kotlin_dir = out_dir.join("kotlin");
    
    recreate_dir(out_dir)?;
    create_dir(&kotlin_dir)?;
    
    // Configure Kotlin binding options
    // let mut binding_config = uniffi_bindgen::bindings::kotlin::BindingConfig::default();
    // if !package_name.is_empty() {
    //     binding_config.package_name = Some(package_name.to_string());
    // }
    
    // Generate Kotlin bindings
    let uniffi_outputs = uniffi_bindgen::library_mode::generate_bindings(
        lib_path,
        None,
        &KotlinBindingGenerator { },
        &CrateConfigSupplier::from(metadata().clone()),
        None,
        out_dir,
        false,
    )?;
    
    // Create kotlin directory structure based on package name
    let package_path = package_name.replace('.', "/");
    let package_dir = kotlin_dir.join(&package_path);
    if !Path::new(&package_dir).exists() {
        fs::create_dir_all(&package_dir)?;
    }
    
    // Copy generated Kotlin files to package directory
    for output in uniffi_outputs {
        let crate_name = output.ci.crate_name();
        
        // Move Kotlin files to appropriate package directory
        let kotlin_files_pattern = out_dir.join(format!("{crate_name}Kt.kt"));
        for entry in glob::glob(kotlin_files_pattern.as_str())?.filter_map(Result::ok) {
            let file_name = entry.file_name().unwrap();
            let file_name_str = file_name.to_str().ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in filename"))?;
            fs::copy(&entry, package_dir.join(file_name_str))?;
        }
    }
    
    Ok(())
}