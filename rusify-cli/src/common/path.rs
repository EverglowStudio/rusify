use std::ops::Deref;
use std::fs::{create_dir, remove_dir_all};
use std::path::Path;
use std::io;
use camino::{Utf8Path, Utf8PathBuf};
use crate::Result;

pub(crate) fn recreate_dir<P>(dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    match remove_dir_all(&dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.into()),
        _ => create_dir(&dir).map_err(|e| e.into()),
    }
}

pub(crate) trait PathExt {
    fn to_relative(&self) -> Result<Utf8PathBuf>;
    fn find_common_path(&self, other: &Utf8Path) -> Utf8PathBuf;
}

impl PathExt for Utf8Path {
    fn to_relative(&self) -> Result<Utf8PathBuf> {
        let cwd = std::env::current_dir()?;
        let cwd: Utf8PathBuf = cwd.try_into()?;
        let common = self.find_common_path(&cwd);
        let remaining = cwd.strip_prefix(common.deref()).unwrap();
        let prefix = remaining
            .components()
            .map(|_| "..")
            .collect::<Utf8PathBuf>();

        let relative = prefix.join(self.strip_prefix(common).unwrap());

        Ok(relative)
    }

    fn find_common_path(&self, other: &Utf8Path) -> Utf8PathBuf {
        let mut self_components = self.components();
        let mut other_components = other.components();
        let mut common_path = Utf8PathBuf::new();
        while let (Some(self_component), Some(other_component)) =
            (self_components.next(), other_components.next())
        {
            if self_component == other_component {
                common_path.push(self_component);
            } else {
                break;
            }
        }

        common_path
    }
}