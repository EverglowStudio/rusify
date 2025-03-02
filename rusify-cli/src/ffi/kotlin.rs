use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs::{self, create_dir};
use std::path::Path;

use crate::common::path::recreate_dir;
use crate::common::metadata::{metadata, MetadataExt};
use crate::android::android_target::AndroidTarget;
use crate::console::step::run_step;
use crate::common::models::{Config, LibType, Mode};

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

        println!("Generating Kotlin bindings from library: {}", lib_path);
        if !lib_path.exists() {
            return Err(anyhow::anyhow!("Library file does not exist: {}", lib_path));
        }

        generate_kotlin_bindings(&lib_path, package_name)
            .map_err(|e| anyhow::anyhow!("Could not generate UniFFI bindings for Kotlin due to the following error: \n {e}"))
    })
}

pub fn generate_kotlin_bindings(lib_path: &Utf8Path, package_name: &str) -> Result<()> {
    let out_dir = Utf8Path::new("./generated");
    let kotlin_dir = out_dir.join("kotlin");
    
    recreate_dir(out_dir)?;
    create_dir(&kotlin_dir)?;
    
    println!("Generating Kotlin bindings in {}", out_dir);
    println!("Using package name: {}", package_name);
    println!("Library path: {}", lib_path);
    
    // Create a config file with the package name
    let config_file = out_dir.join("uniffi.toml");
    let config_content = format!(
        "[bindings.kotlin]\npackage_name = \"{}\"\n",
        package_name
    );
    fs::write(&config_file, config_content)?;
    
    println!("Created config file: {}", config_file);
    
    // Use the library_mode to generate bindings
    let binding_generator = uniffi_bindgen::bindings::KotlinBindingGenerator;
    let components = uniffi_bindgen::library_mode::generate_bindings(
        lib_path,
        None,
        &binding_generator,
        &uniffi_bindgen::cargo_metadata::CrateConfigSupplier::from(metadata().clone()),
        Some(&config_file),
        out_dir,
        false,
    )?;
    
    println!("Generated {} component(s)", components.len());
    
    // Create kotlin directory structure based on package name
    let package_path = package_name.replace('.', "/");
    let package_dir = kotlin_dir.join(&package_path);
    if !Path::new(&package_dir).exists() {
        fs::create_dir_all(&package_dir)?;
    }
    
    // Copy all .kt files from the package directory to our kotlin directory
    let expected_package_dir = out_dir.join(package_path.clone());
    if expected_package_dir.exists() {
        println!("Found generated package directory: {}", expected_package_dir);
        let mut found_files = 0;
        if let Ok(entries) = fs::read_dir(&expected_package_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().extension().map_or(false, |ext| ext == "kt") {
                        found_files += 1;
                        println!("  Copying Kotlin file: {}", entry.path().display());
                        let path = entry.path();
                        let file_name = path.file_name().unwrap();
                        let file_name_str = file_name.to_string_lossy();
                        let dest_file = package_dir.join(file_name_str.as_ref());
                        fs::copy(entry.path(), dest_file)?;
                    }
                }
            }
        }
        if found_files == 0 {
            println!("Warning: No Kotlin files found in the package directory");
        }
    } else {
        println!("Warning: Expected package directory not found: {}", expected_package_dir);
        
        // Fallback: try to find the Kotlin files elsewhere
        println!("Checking for Kotlin files in the root output directory...");
        let mut found_files = 0;
        if let Ok(entries) = fs::read_dir(out_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().extension().map_or(false, |ext| ext == "kt") {
                        found_files += 1;
                        println!("  Copying Kotlin file: {}", entry.path().display());
                        let path = entry.path();
                        let file_name = path.file_name().unwrap();
                        let file_name_str = file_name.to_string_lossy();
                        let dest_file = package_dir.join(file_name_str.as_ref());
                        fs::copy(entry.path(), dest_file)?;
                    }
                }
            }
        }
        if found_files == 0 {
            println!("Warning: No Kotlin files found in the root output directory");
            
            // As a last resort, use glob to search for all .kt files
            println!("Using glob to search for all .kt files...");
            let all_kt_files_pattern = out_dir.join("**/*.kt");
            let mut found_files = 0;
            for entry in glob::glob(all_kt_files_pattern.as_str())?.filter_map(Result::ok) {
                found_files += 1;
                println!("  Found Kotlin file: {}", entry.display());
                let file_name = entry.file_name().unwrap();
                let file_name_str = file_name.to_string_lossy();
                let dest_file = package_dir.join(file_name_str.as_ref());
                fs::copy(&entry, dest_file)?;
            }
            if found_files == 0 {
                println!("Warning: No Kotlin files found with glob search");
                println!("This suggests that no Kotlin bindings were generated");
                
                // List all files in the output directory to help debug
                println!("Files in {}:", out_dir);
                if let Ok(entries) = fs::read_dir(out_dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            println!("  {}", entry.path().display());
                            
                            // If it's a directory, list its contents
                            if entry.path().is_dir() {
                                if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                                    for sub_entry in sub_entries {
                                        if let Ok(sub_entry) = sub_entry {
                                            println!("    {}", sub_entry.path().display());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn library_file_name(lib_name: &str, lib_type: LibType) -> String {
    format!("lib{}.{}", lib_name, lib_type.file_extension_android())
}