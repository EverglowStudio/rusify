mod commands {
    pub mod init;
}
pub mod apple {
    pub mod apple_target;
    pub mod package;
    pub mod swiftpackage;
    pub mod xcframework;
}
pub mod android {
    pub mod android_target;
    pub mod package;
    pub mod aar;
}
pub mod ohos {
    pub mod ohos_arch;
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
    pub mod kotlin;
}

pub use commands::*;
pub use common::*;
pub use console::*;

pub use anyhow::{Context, Result};