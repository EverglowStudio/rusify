use anyhow::{Context, Result};
use askama::Template;
use std::fs::{self, create_dir_all, File};
use std::path::{Path, PathBuf};

use crate::android::android_target::AndroidTarget;
use crate::common::models::{Config, LibType, Mode};
use crate::common::path::recreate_dir;
use crate::console::step::run_step;
use crate::common::templating::{AndroidManifest, GradleProperties};

pub(crate) fn create_aar_with_output(
    targets: &[AndroidTarget],
    lib_name: &str,
    package_name: &str,
    aar_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
) -> Result<()> {
    run_step(config, "Creating Android AAR package...", || {
        let output_dir = PathBuf::from(aar_name);
        recreate_dir(&output_dir)?;
        
        create_aar(
            targets,
            lib_name,
            package_name,
            &output_dir,
            mode,
            lib_type,
        )
    })
    .map_err(|e| {
        anyhow::anyhow!(
            "Failed to create AAR package due to the following error: \n {}",
            e
        )
    })
}

fn create_aar(
    targets: &[AndroidTarget],
    lib_name: &str,
    package_name: &str,
    output_dir: &Path,
    mode: Mode,
    lib_type: LibType,
) -> Result<()> {
    // Create the necessary directory structure for the AAR
    let jni_dir = output_dir.join("jni");
    create_dir_all(&jni_dir)?;
    
    // Create manifests directory
    let manifests_dir = output_dir.join("manifests");
    create_dir_all(&manifests_dir)?;
    
    // Generate AndroidManifest.xml
    let android_manifest = AndroidManifest { package_name };
    let manifest_content = android_manifest.render()
        .context("Failed to render AndroidManifest.xml template")?;
    
    // Write AndroidManifest.xml
    let manifest_path = manifests_dir.join("AndroidManifest.xml");
    fs::write(&manifest_path, manifest_content)
        .context("Failed to write AndroidManifest.xml")?;
    
    // Copy native libraries
    for target in targets {
        let arch_dir = jni_dir.join(target.ndk_arch);
        create_dir_all(&arch_dir)?;
        
        let lib_path = target.library_path(lib_name, mode, lib_type);
        let dest_path = arch_dir.join(format!("lib{}.so", lib_name));
        
        fs::copy(lib_path, dest_path)
            .context(format!("Failed to copy library for {}", target.display_name()))?;
    }
    
    // Copy Kotlin sources
    let kotlin_dir = output_dir.join("kotlin");
    create_dir_all(&kotlin_dir)?;
    
    // Create package directory structure
    let package_path = package_name.replace(".", "/");
    let package_dir = kotlin_dir.join(&package_path);
    create_dir_all(&package_dir)?;
    
    // Copy Kotlin files from generated directory
    let generated_dir = Path::new("./generated/kotlin").join(&package_path);
    if generated_dir.exists() {
        for entry in fs::read_dir(generated_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "kt") {
                let dest_path = package_dir.join(path.file_name().unwrap());
                fs::copy(&path, dest_path)?;
            }
        }
    }
    
    // Create gradle.properties
    let gradle_props = GradleProperties { lib_name };
    let gradle_content = gradle_props.render()
        .context("Failed to render gradle.properties template")?;
    
    fs::write(output_dir.join("gradle.properties"), gradle_content)
        .context("Failed to write gradle.properties")?;
    
    // Create a basic R.txt file (required for AAR)
    File::create(output_dir.join("R.txt"))?;
    
    // Create a simple proguard.txt
    fs::write(
        output_dir.join("proguard.txt"),
        "-keep class ".to_string() + package_name + ".** { *; }\n",
    )?;
    
    // Create META-INF directory
    let meta_inf_dir = output_dir.join("META-INF");
    create_dir_all(&meta_inf_dir)?;
    
    // Generate zip file (AAR is a zip file with a specific structure)
    let aar_path = PathBuf::from(format!("{}.aar", output_dir.to_str().unwrap()));
    
    // Create a zip file in Rust without external dependencies
    let aar_file = std::fs::File::create(&aar_path)
        .context("Failed to create AAR file")?;
    
    let mut zip = zip::ZipWriter::new(aar_file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    
    // Helper function to recursively add files to the zip
    fn add_dir_to_zip<T: std::io::Write + std::io::Seek>(
        zip: &mut zip::ZipWriter<T>,
        options: &zip::write::FileOptions,
        dir: &Path,
        root: &Path,
    ) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(root)?;
            
            if path.is_dir() {
                zip.add_directory(
                    relative_path.to_string_lossy().as_ref(),
                    *options,
                )?;
                add_dir_to_zip(zip, options, &path, root)?;
            } else {
                zip.start_file(relative_path.to_string_lossy().as_ref(), *options)?;
                let mut file = std::fs::File::open(&path)?;
                std::io::copy(&mut file, zip)?;
            }
        }
        Ok(())
    }
    
    // Add all files and directories to the zip
    add_dir_to_zip(&mut zip, &options, output_dir, output_dir)
        .context("Failed to add files to AAR archive")?;
    
    // Finalize the zip file
    zip.finish()
        .context("Failed to finalize AAR archive")?;
    
    println!("Created AAR package at: {}", aar_path.display());
    
    Ok(())
}