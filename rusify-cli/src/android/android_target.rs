use crate::common::metadata::{metadata, MetadataExt};
use crate::common::models::{FeatureOptions, LibType, Mode};
use execute::command;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AndroidTarget {
    pub architectures: Vec<&'static str>,
    pub display_name: &'static str,
    pub ndk_arch: &'static str,
    pub api_level: u32,
}

const ARCH_COUNT: usize = 4;
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum AndroidArch {
    ARM64V8A,
    ARMV7A,
    X86,
    X86_64,
}

impl AndroidArch {
    pub(crate) fn display_name(&self) -> String {
        match self {
            AndroidArch::ARM64V8A => "ARM64-v8a",
            AndroidArch::ARMV7A => "ARMv7-a",
            AndroidArch::X86 => "x86",
            AndroidArch::X86_64 => "x86_64",
        }
        .to_string()
    }

    pub(crate) fn all() -> [Self; ARCH_COUNT] {
        [
            Self::ARM64V8A,
            Self::ARMV7A,
            Self::X86,
            Self::X86_64,
        ]
    }

    pub(crate) fn target(&self, api_level: u32) -> AndroidTarget {
        match self {
            AndroidArch::ARM64V8A => AndroidTarget {
                architectures: vec!["aarch64-linux-android"],
                display_name: "ARM64-v8a",
                ndk_arch: "arm64-v8a",
                api_level,
            },
            AndroidArch::ARMV7A => AndroidTarget {
                architectures: vec!["armv7-linux-androideabi"],
                display_name: "ARMv7-a",
                ndk_arch: "armeabi-v7a",
                api_level,
            },
            AndroidArch::X86 => AndroidTarget {
                architectures: vec!["i686-linux-android"],
                display_name: "x86",
                ndk_arch: "x86",
                api_level,
            },
            AndroidArch::X86_64 => AndroidTarget {
                architectures: vec!["x86_64-linux-android"],
                display_name: "x86_64",
                ndk_arch: "x86_64",
                api_level,
            },
        }
    }
}

impl AndroidTarget {
    pub(crate) fn cargo_build_commands(&self, mode: Mode, features: &FeatureOptions) -> Vec<Command> {
        self.architectures
            .iter()
            .map(|arch| {
                let mut cmd = command("cargo build");
                cmd.arg("--target").arg(arch);

                match mode {
                    Mode::Debug => {}
                    Mode::Release => {
                        cmd.arg("--release");
                    }
                }

                if let Some(features) = &features.features {
                    cmd.arg("--features").arg(features.join(","));
                }
                if features.all_features {
                    cmd.arg("--all-features");
                }
                if features.no_default_features {
                    cmd.arg("--no-default-features");
                }
                
                // Pass special environment variables for the ring crate
                cmd.env("CARGO_TERM_COLOR", "always");
                
                // Configure environment variables from current process
                if let Ok(path) = std::env::var("PATH") {
                    cmd.env("PATH", path);
                }
                
                // Compiler variables
                if let Ok(cc) = std::env::var("CC") {
                    cmd.env("CC", cc);
                }
                let cc_var = format!("CC_{}", arch.replace("-", "_"));
                if let Ok(cc_arch) = std::env::var(&cc_var) {
                    cmd.env(cc_var, cc_arch);
                }
                if let Ok(target_cc) = std::env::var("TARGET_CC") {
                    cmd.env("TARGET_CC", target_cc);
                }
                
                // Archiver variables
                if let Ok(ar) = std::env::var("AR") {
                    cmd.env("AR", ar);
                }
                let ar_var = format!("AR_{}", arch.replace("-", "_"));
                if let Ok(ar_arch) = std::env::var(&ar_var) {
                    cmd.env(ar_var, ar_arch);
                }
                if let Ok(target_ar) = std::env::var("TARGET_AR") {
                    cmd.env("TARGET_AR", target_ar);
                }
                
                // Pass CFLAGS variables
                let cflags_var = format!("CFLAGS_{}", arch.replace("-", "_"));
                if let Ok(cflags) = std::env::var(&cflags_var) {
                    cmd.env(cflags_var, cflags);
                }
                if let Ok(target_cflags) = std::env::var("TARGET_CFLAGS") {
                    cmd.env("TARGET_CFLAGS", target_cflags);
                }

                cmd
            })
            .collect()
    }

    fn setup_commands(&self) -> Vec<Command> {
        // Don't use commands for setup - we'll handle this directly
        Vec::new()
    }
    
    pub fn setup_environment(&self) -> Result<(), anyhow::Error> {
        // Check if ANDROID_NDK_HOME is set
        let ndk_home = match std::env::var("ANDROID_NDK_HOME") {
            Ok(path) => path,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Error: ANDROID_NDK_HOME environment variable is not set. Please install Android NDK and set ANDROID_NDK_HOME."
                ));
            }
        };
        
        // Handle platform-specific path separators and directories
        let host_os = if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "linux"
        };
        
        // Handle platform-specific path construction
        let path_sep = if cfg!(target_os = "windows") { "\\" } else { "/" };
        
        // Create toolchain path in platform-specific way
        let toolchain_path = format!(
            "{}{}toolchains{}llvm{}prebuilt{}{}-x86_64", 
            ndk_home, path_sep, path_sep, path_sep, path_sep, host_os
        );
        
        // Create patched paths for tools based on platform
        let arch = self.architectures[0];
        let api = self.api_level;
        
        // Handle Windows-specific exe extension
        let exe_ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
        
        // Create .cargo directory if it doesn't exist
        std::fs::create_dir_all(".cargo")?;
        
        // Create paths for tools
        let ar_path = format!("{}{}bin{}llvm-ar{}", toolchain_path, path_sep, path_sep, exe_ext);
        let clang_path = format!("{}{}bin{}{}{}-clang{}", 
            toolchain_path, path_sep, path_sep, arch, api, exe_ext);
        
        // On Windows, use forward slashes in config file even though the OS uses backslashes
        let ar_config_path = ar_path.replace("\\", "/");
        let clang_config_path = clang_path.replace("\\", "/");
        
        // Create .cargo/config.toml with absolute paths
        let config_content = format!(
            "[target.{}]\n\
            ar = \"{}\"\n\
            linker = \"{}\"\n",
            arch, ar_config_path, clang_config_path
        );
        
        std::fs::write(".cargo/config.toml", config_content)?;
        
        // Verify that the tools exist
        if !std::path::Path::new(&ar_path).exists() {
            return Err(anyhow::anyhow!(
                "Android NDK tool not found: {}. Please check your ANDROID_NDK_HOME setting.", 
                ar_path
            ));
        }
        
        if !std::path::Path::new(&clang_path).exists() {
            return Err(anyhow::anyhow!(
                "Android NDK tool not found: {}. Please check your ANDROID_NDK_HOME setting.", 
                clang_path
            ));
        }
        
        // ====== SPECIAL HANDLING FOR RING CRATE ======
        // The ring crate expects a compiler named exactly aarch64-linux-android-clang (without API level)
        self.setup_ring_specific_fixes(&toolchain_path, path_sep, exe_ext, api)?;
        
        Ok(())
    }
    
    /// Special fixes for the ring crate which needs specific compiler naming
    fn setup_ring_specific_fixes(&self, toolchain_path: &str, path_sep: &str, exe_ext: &str, api: u32) -> Result<(), anyhow::Error> {
        println!("Applying ring-specific fixes...");
        
        // Create a directory for our wrapper scripts
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let wrapper_dir = format!("{}{}android-toolchain-bin", home_dir, path_sep);
        std::fs::create_dir_all(&wrapper_dir)?;
        
        // For each architecture, create wrapper scripts for the compiler and archiver
        for arch in &self.architectures {
            // Create the wrapper script/symlink for clang
            let wrapper_clang_path = format!("{}{}{}{}clang{}", 
                wrapper_dir, path_sep, arch, "-linux-android", exe_ext);
            
            // The actual compiler path with API level
            let real_compiler_path = format!("{}{}bin{}{}{}-clang{}", 
                toolchain_path, path_sep, path_sep, arch, api, exe_ext);
            
            // Create the wrapper script/symlink for AR (archiver)
            let wrapper_ar_path = format!("{}{}{}{}ar{}", 
                wrapper_dir, path_sep, arch, "-linux-android", exe_ext);
            
            // The actual AR path
            let real_ar_path = format!("{}{}bin{}llvm-ar{}", 
                toolchain_path, path_sep, path_sep, exe_ext);
            
            if cfg!(target_os = "windows") {
                // On Windows we need a .bat file to redirect for clang
                let batch_content = format!(
                    "@echo off\r\n\"{0}\" %*\r\n",
                    real_compiler_path.replace("\\", "\\\\")
                );
                std::fs::write(&wrapper_clang_path, batch_content)?;
                
                // On Windows we need a .bat file to redirect for AR
                let ar_batch_content = format!(
                    "@echo off\r\n\"{0}\" %*\r\n",
                    real_ar_path.replace("\\", "\\\\")
                );
                std::fs::write(&wrapper_ar_path, ar_batch_content)?;
            } else {
                // On Unix we can create shell scripts
                
                // Create the clang wrapper script
                let script_content = format!(
                    "#!/bin/sh\nexec \"{}\" \"$@\"\n",
                    real_compiler_path
                );
                std::fs::write(&wrapper_clang_path, script_content)?;
                
                // Create the AR wrapper script
                let ar_script_content = format!(
                    "#!/bin/sh\nexec \"{}\" \"$@\"\n",
                    real_ar_path
                );
                std::fs::write(&wrapper_ar_path, ar_script_content)?;
                
                // Make the scripts executable
                use std::os::unix::fs::PermissionsExt;
                
                let mut clang_perms = std::fs::metadata(&wrapper_clang_path)?.permissions();
                clang_perms.set_mode(0o755);
                std::fs::set_permissions(&wrapper_clang_path, clang_perms)?;
                
                let mut ar_perms = std::fs::metadata(&wrapper_ar_path)?.permissions();
                ar_perms.set_mode(0o755);
                std::fs::set_permissions(&wrapper_ar_path, ar_perms)?;
            }
        }
        
        // Set environment variables for ring's build script
        std::env::set_var("PATH", format!("{}{}{}",
            wrapper_dir, 
            if cfg!(target_os = "windows") { ";" } else { ":" },
            std::env::var("PATH").unwrap_or_default()
        ));
        
        // Set all possible compiler environment variables that might be checked
        let arch = self.architectures[0];
        let wrapper_script_path = format!("{}{}{}{}clang", 
            wrapper_dir, path_sep, arch, "-linux-android");
        
        std::env::set_var("CC", &wrapper_script_path);
        std::env::set_var(&format!("CC_{}", arch.replace("-", "_")), &wrapper_script_path);
        std::env::set_var("TARGET_CC", &wrapper_script_path);
        
        // Also set AR variables
        let ar_wrapper_path = format!("{}{}{}{}ar", 
            wrapper_dir, path_sep, arch, "-linux-android");
            
        std::env::set_var("AR", &ar_wrapper_path);
        std::env::set_var(&format!("AR_{}", arch.replace("-", "_")), &ar_wrapper_path);
        std::env::set_var("TARGET_AR", &ar_wrapper_path);
        
        // Also set CFLAGS variables to help system detection
        let cflags = format!("--target={}{}", arch, api);
        std::env::set_var(&format!("CFLAGS_{}", arch.replace("-", "_")), &cflags);
        std::env::set_var("TARGET_CFLAGS", &cflags);
        
        println!("Ring-specific fixes applied successfully.");
        println!("Wrapper scripts created in: {}", wrapper_dir);
        println!("Compiler wrapper: {}", wrapper_script_path);
        println!("Archiver wrapper: {}", ar_wrapper_path);
        
        Ok(())
    }

    /// Generates all commands necessary to build this target
    ///
    /// This function returns a list of commands that should be executed in their given
    /// order to build this target.
    pub fn commands(
        &self,
        _lib_name: &str,
        mode: Mode,
        _lib_type: LibType,
        features: &FeatureOptions,
    ) -> Vec<Command> {
        let mut commands = self.setup_commands();
        commands.extend(self.cargo_build_commands(mode, features));
        commands
    }

    /// Returns the names of all target architectures for this target
    ///
    /// The names returned here exactly match the identifiers of the respective official Rust targets.
    pub fn architectures(&self) -> &[&'static str] {
        &self.architectures
    }

    pub fn display_name(&self) -> &'static str {
        self.display_name
    }

    pub fn library_directory(&self, mode: Mode) -> String {
        let mode_str = match mode {
            Mode::Debug => "debug",
            Mode::Release => "release",
        };

        let target = metadata().target_dir();
        format!("{target}/{}/{mode_str}", self.architectures[0])
    }

    pub fn library_path(&self, lib_name: &str, mode: Mode, lib_type: LibType) -> String {
        format!(
            "{}/{}",
            self.library_directory(mode),
            library_file_name(lib_name, lib_type)
        )
    }
}

pub fn library_file_name(lib_name: &str, lib_type: LibType) -> String {
    format!("lib{}.{}", lib_name, lib_type.file_extension_android())
}