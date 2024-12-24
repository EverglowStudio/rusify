use crate::common::metadata::{metadata, MetadataExt};
use crate::common::models::{FeatureOptions, LibType, Mode};
use execute::command;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AppleTarget {
    pub universal_name: Option<&'static str>,
    pub architectures: Vec<&'static str>,
    pub display_name: &'static str,
    pub platform: ApplePlatformTarget,
}

const PLATFORM_COUNT: usize = 5;
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum ApplePlatform {
    IOS,
    MacOS,
    TvOS,
    WatchOS,
    VisionOS,
}

#[derive(Clone, Copy, Debug)]
pub struct ApplePlatformTarget {
    pub platform: ApplePlatform,
    pub is_simulator: bool,
}

impl ApplePlatform {
    pub(crate) fn into_apple_platform_target(&self) -> Vec<ApplePlatformTarget> {
        match self {
            ApplePlatform::MacOS => vec![ApplePlatformTarget {
                platform: *self,
                is_simulator: false,
            }],
            _ => vec![
                ApplePlatformTarget {
                    platform: *self,
                    is_simulator: false,
                },
                ApplePlatformTarget {
                    platform: *self,
                    is_simulator: true,
                },
            ],
        }
    }
}

impl AppleTarget {
    fn cargo_build_commands(&self, mode: Mode, features: &FeatureOptions) -> Vec<Command> {
        self.architectures
            .iter()
            .map(|arch| {
                // FIXME: Remove nightly for Tier 3 targets here once build-std is stabilized
                let mut cmd = if self.platform.platform.is_tier_3() {
                    command("cargo +nightly build -Z build-std")
                } else {
                    command("cargo build")
                };
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

                cmd
            })
            .collect()
    }

    fn lipo_commands(&self, lib_name: &str, mode: Mode, lib_type: LibType) -> Vec<Command> {
        if self.architectures.len() <= 1 {
            return vec![];
        }

        let path = self.library_directory(mode);

        let target = metadata().target_dir();
        let target_name = library_file_name(lib_name, lib_type);
        let component_paths: Vec<_> = self
            .architectures
            .iter()
            .map(|arch| format!("{target}/{arch}/{mode}/{target_name}"))
            .collect();
        let args = component_paths.join(" ");
        let target_path = self.library_path(lib_name, mode, lib_type);

        let make_dir = command(format!("mkdir -p {path}"));
        let lipo = command(format!("lipo {args} -create -output {target_path}"));
        vec![make_dir, lipo]
    }

    fn rpath_install_id_commands(
        &self,
        lib_name: &str,
        mode: Mode,
        lib_type: LibType,
    ) -> Vec<Command> {
        if matches!(lib_type, LibType::Dynamic) {
            vec![command(format!(
                "install_name_tool -id @rpath/{} {}",
                library_file_name(lib_name, lib_type),
                self.library_path(lib_name, mode, lib_type)
            ))]
        } else {
            vec![]
        }
    }

    /// Generates all commands necessary to build this target
    ///
    /// This function returns a list of commands that should be executed in their given
    /// order to build this target (and bundle architecture targets with lipo if it is a universal target).
    pub fn commands(
        &self,
        lib_name: &str,
        mode: Mode,
        lib_type: LibType,
        features: &FeatureOptions,
    ) -> Vec<Command> {
        self.cargo_build_commands(mode, features)
            .into_iter()
            .chain(self.lipo_commands(lib_name, mode, lib_type))
            .chain(self.rpath_install_id_commands(lib_name, mode, lib_type))
            .collect()
    }

    /// Returns the names of all target architectures for this target
    ///
    /// If this target is a single target, the returned vector will always contain exactly one element.
    /// The names returned here exactly match the identifiers of the respective official Rust targets.
    pub fn architectures(&self) -> &[&'static str] {
        &self.architectures
    }

    pub fn display_name(&self) -> &'static str {
        self.display_name
    }

    pub fn platform(&self) -> ApplePlatformTarget {
        self.platform
    }

    pub fn library_directory(&self, mode: Mode) -> String {
        let mode = match mode {
            Mode::Debug => "debug",
            Mode::Release => "release",
        };

        let target = metadata().target_dir();

        match self.universal_name {
            Some(universal_name) => format!("{target}/{universal_name}/{mode}"),
            None => format!("{target}/{}/{mode}", self.architectures[0]),
        }
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
    format!("lib{}.{}", lib_name, lib_type.file_extension())
}

impl ApplePlatform {
    pub(crate) fn display_name(&self) -> String {
        let name = match self {
            ApplePlatform::MacOS => "macOS",
            ApplePlatform::IOS => "iOS",
            ApplePlatform::TvOS => "tvOS",
            ApplePlatform::WatchOS => "watchOS",
            ApplePlatform::VisionOS => "visionOS",
        };

        format!(
            "{name}{}",
            if self.is_experimental() {
                " (Experimental)"
            } else {
                ""
            }
        )
    }

    pub(crate) fn is_experimental(&self) -> bool {
        match self {
            Self::MacOS | Self::IOS => false,
            Self::TvOS | Self::WatchOS | Self::VisionOS => true,
        }
    }

    pub(crate) fn all() -> [Self; PLATFORM_COUNT] {
        [
            Self::MacOS,
            Self::IOS,
            Self::TvOS,
            Self::WatchOS,
            Self::VisionOS,
        ]
    }

    pub(crate) fn is_tier_3(&self) -> bool {
        match self {
            ApplePlatform::IOS => false,
            ApplePlatform::MacOS => false,
            ApplePlatform::TvOS => true,
            ApplePlatform::WatchOS => true,
            ApplePlatform::VisionOS => true,
        }
    }
}

impl ApplePlatformTarget {
    pub(crate) fn target(&self) -> AppleTarget {
        use ApplePlatform::*;
        match (self.platform, self.is_simulator) {
            (IOS, false) => AppleTarget {
                universal_name: None,
                architectures: vec!["aarch64-apple-ios"],
                display_name: "iOS",
                platform: *self,
            },
            (IOS, true) => AppleTarget {
                universal_name: Some("universal-ios"),
                architectures: vec!["x86_64-apple-ios", "aarch64-apple-ios-sim"],
                display_name: "iOS Simulator",
                platform: *self,
            },
            (MacOS, _) => AppleTarget {
                universal_name: Some("universal-macos"),
                architectures: vec!["x86_64-apple-darwin", "aarch64-apple-darwin"],
                display_name: "macOS",
                platform: *self,
            },
            (TvOS, false) => AppleTarget {
                universal_name: None,
                architectures: vec!["aarch64-apple-tvos"],
                display_name: "tvOS",
                platform: *self,
            },
            (TvOS, true) => AppleTarget {
                universal_name: Some("universal-tvos-simulator"),
                architectures: vec!["aarch64-apple-tvos-sim", "x86_64-apple-tvos"],
                display_name: "tvOS Simulator",
                platform: *self,
            },
            (WatchOS, false) => AppleTarget {
                universal_name: Some("universal-watchos"),
                architectures: vec![
                    "aarch64-apple-watchos",
                    "arm64_32-apple-watchos",
                    "armv7k-apple-watchos",
                ],
                display_name: "watchOS",
                platform: *self,
            },
            (WatchOS, true) => AppleTarget {
                universal_name: Some("universal-watchos-sim"),
                architectures: vec!["aarch64-apple-watchos-sim", "x86_64-apple-watchos-sim"],
                display_name: "watchOS Simulator",
                platform: *self,
            },
            (VisionOS, false) => AppleTarget {
                universal_name: None,
                architectures: vec!["aarch64-apple-visionos"],
                display_name: "visionOS",
                platform: *self,
            },
            (VisionOS, true) => AppleTarget {
                universal_name: None,
                architectures: vec!["aarch64-apple-visionos-sim"],
                display_name: "visionOS Simulator",
                platform: *self,
            },
        }
    }
}
