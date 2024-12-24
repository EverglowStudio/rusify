use std::str::FromStr;

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OhosArch {
    ARM64,
    ARM32,
    X86_64,
}

impl OhosArch {
    pub fn all() -> [Self; 3] {
        [Self::ARM64, Self::ARM32, Self::X86_64]
    }

    pub fn to_arch(self) -> &'static str {
        match self {
            OhosArch::ARM64 => "arm64-v8a",
            OhosArch::ARM32 => "armeabi-v7a",
            OhosArch::X86_64 => "x86_64",
        }
    }
    pub fn c_target(self) -> &'static str {
        match self {
            OhosArch::ARM64 => "aarch64-linux-ohos",
            OhosArch::ARM32 => "arm-linux-ohos",
            OhosArch::X86_64 => "x86_64-linux-ohos",
        }
    }
    pub fn rust_link_target(self) -> &'static str {
        match self {
            OhosArch::ARM64 => "AARCH64_UNKNOWN_LINUX_OHOS",
            OhosArch::ARM32 => "ARMV7_UNKNOWN_LINUX_OHOS",
            OhosArch::X86_64 => "X86_64_UNKNOWN_LINUX_OHOS",
        }
    }

    pub fn rust_target(self) -> &'static str {
        match self {
            OhosArch::ARM64 => "aarch64-unknown-linux-ohos",
            OhosArch::ARM32 => "armv7-unknown-linux-ohos",
            OhosArch::X86_64 => "x86_64-unknown-linux-ohos",
        }
    }
}

impl FromStr for OhosArch {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let ret = s.to_lowercase();
        match ret.as_ref() {
            "aarch"
            | "arm64"
            | "aarch64-linux-ohos"
            | "aarch64-unknown-linux-ohos"
            | "AARCH64_UNKNOWN_LINUX_OHOS" => Ok(OhosArch::ARM64),
            "arm"
            | "arm32"
            | "arm-linux-ohos"
            | "armv7-unknown-linux-ohos"
            | "ARMV7_UNKNOWN_LINUX_OHOS" => Ok(OhosArch::ARM32),
            "x86_64"
            | "x64"
            | "x86_64-linux-ohos"
            | "x86_64-unknown-linux-ohos"
            | "X86_64_UNKNOWN_LINUX_OHOS" => Ok(OhosArch::X86_64),
            _ => Err(
                "Only supports aarch/arm64, arm/arm32, and x86_64/x64 architectures.".to_string(),
            ),
        }
    }
}
