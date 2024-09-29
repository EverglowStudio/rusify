use std::io;
use std::fs::{self, create_dir};

use camino::{Utf8Path, Utf8PathBuf};
use uniffi_bindgen::{
    bindings::SwiftBindingGenerator,
    cargo_metadata::CrateConfigSupplier,
};

use crate::{
    apple::apple_target::{AppleTarget, library_file_name},
    common::{
        metadata::MetadataExt,
        models::{Config, LibType, Mode},
        path::recreate_dir,
    },
    console::step::run_step,
    metadata::metadata,
    Result,
};

pub(crate) fn generate_swift_bindings_with_output(
    targets: &[AppleTarget],
    lib_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
) -> Result<()> {
    run_step(config, "Generating Swift bindings...", || {
        let lib_file = library_file_name(lib_name, lib_type);
        let target = metadata().target_dir();
        let archs = targets
            .first()
            .ok_or_else(|| anyhow::anyhow!("Could not generate UniFFI bindings: No target platform selected!"))?
            .architectures();
        let arch = archs.first().ok_or_else(|| anyhow::anyhow!("No architectures found for the selected target"))?;
        let lib_path: Utf8PathBuf = format!("{}/{}/{}/{}", target, arch, mode, lib_file).into();

        generate_swift_bindings(&lib_path)
            .map_err(|e| anyhow::anyhow!("Could not generate UniFFI bindings for udl files due to the following error: \n {e}"))
    })
}

pub fn generate_swift_bindings(lib_path: &Utf8Path) -> Result<()> {
    let out_dir = Utf8Path::new("./generated");
    let headers = out_dir.join("headers");
    let sources = out_dir.join("sources");

    recreate_dir(out_dir)?;
    create_dir(&headers)?;
    create_dir(&sources)?;

    let uniffi_outputs = uniffi_bindgen::library_mode::generate_bindings(
        lib_path,
        None,
        &SwiftBindingGenerator {},
        &CrateConfigSupplier::from(metadata().clone()),
        None,
        out_dir,
        false,
    )?;

    let mut modulemap = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(headers.join("module.modulemap"))?;

    for output in uniffi_outputs {
        let crate_name = output.ci.crate_name();
        fs::copy(
            out_dir.join(format!("{crate_name}.swift")),
            sources.join(format!("{crate_name}.swift")),
        )?;

        let ffi_name = format!("{crate_name}FFI");
        fs::copy(
            out_dir.join(format!("{ffi_name}.h")),
            headers.join(format!("{ffi_name}.h")),
        )?;

        let mut modulemap_part = fs::OpenOptions::new()
            .read(true)
            .open(out_dir.join(format!("{ffi_name}.modulemap")))?;
        io::copy(&mut modulemap_part, &mut modulemap)?;
    }

    Ok(())
}
