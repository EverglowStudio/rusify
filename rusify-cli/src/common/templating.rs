use askama::Template;

#[derive(Template)]
#[template(path = "Cargo.toml.txt", escape = "none")]
pub(crate) struct CargoToml<'a> {
    pub(crate) crate_name: &'a str,
}

#[derive(Template)]
#[template(path = "lib.rs.txt", escape = "none")]
pub(crate) struct LibRs;

#[derive(Template)]
#[template(path = "Package.swift.txt", escape = "none")]
pub(crate) struct PackageSwift<'a> {
    pub(crate) package_name: &'a str,
    pub(crate) xcframework_name: &'a str,
    pub(crate) disable_warnings: bool,
}