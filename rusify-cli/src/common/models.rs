use std::{fmt::Display, str::FromStr};

#[derive(clap::ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
#[value()]
pub enum LibType {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Debug,
    Release,
}

pub struct Config {
    pub silent: bool,
    pub accept_all: bool,
}

#[derive(Debug, Clone)]
pub struct FeatureOptions {
    pub features: Option<Vec<String>>,
    pub all_features: bool,
    pub no_default_features: bool,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Debug => write!(f, "debug"),
            Mode::Release => write!(f, "release"),
        }
    }
}

impl FromStr for LibType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "staticlib" => Ok(Self::Static),
            "cdylib" => Ok(Self::Dynamic),
            _ => Err(anyhow::anyhow!("Invalid library type: {}", s)),
        }
    }
}

impl LibType {
    /// The identifier used in the crate-type field in Cargo.toml
    pub fn identifier(&self) -> &str {
        match self {
            LibType::Static => "staticlib",
            LibType::Dynamic => "cdylib",
        }
    }

    pub fn file_extension(&self) -> &str {
        match self {
            LibType::Static => "a",
            LibType::Dynamic => "dylib",
        }
    }
}

impl std::fmt::Display for LibType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static => write!(f, "static"),
            Self::Dynamic => write!(f, "dynamic"),
        }
    }
}