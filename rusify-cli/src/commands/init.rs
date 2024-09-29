use askama::Template;
use std::fs;
use std::path::Path;

use crate::common::templating;

pub fn init_crate(
    name: String
) {
    let crate_dir = Path::new(&name);
    fs::create_dir_all(crate_dir).unwrap();

    let cargo_toml = templating::CargoToml {
         crate_name: &name 
    };
    let rendered_cargo_toml = cargo_toml.render().unwrap();
    fs::write(crate_dir.join("Cargo.toml"), rendered_cargo_toml).unwrap();

    fs::create_dir_all(crate_dir.join("src")).unwrap();

    let lib_rs = templating::LibRs;
    let rendered_lib_rs = lib_rs.render().unwrap();
    fs::write(crate_dir.join("src/lib.rs"), rendered_lib_rs).unwrap();
}