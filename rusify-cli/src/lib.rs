mod commands {
    pub mod init;
}
mod apple {
    pub mod apple_target;
    pub mod package;
    pub mod swiftpackage;
    pub mod xcframework;
}
pub(crate) mod console {
    pub mod step;
    pub mod spinners;
    pub mod command;
    pub mod theme;
    pub mod messages;

    pub use command::*;
    pub use spinners::*;
    pub use step::*;
    pub use theme::*;
    pub use messages::*;
}

mod common {
    pub mod metadata;
    pub mod path;
    pub mod models;
    pub mod templating;
}
mod ffi {
    pub mod swift;
}

pub use commands::*;
pub use apple::*;
pub use common::*;
pub use console::*;

pub use anyhow::{Context, Result};