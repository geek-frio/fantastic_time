use crate::img::*;
use anyhow::Error as AnyError;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs::{self, ReadDir};
use std::path::Path;
use tracing::{trace, warn};

const IMAGE_EXT: [&'static str; 3] = ["jpg", "jpeg", "png"];
pub struct DirsScanner;

impl DirsScanner {
    fn scan_entry<P>(root: P) -> Result<(), AnyError>
    where
        P: AsRef<OsStr>,
    {
        let root = Path::new(root.as_ref());
        let dir_content = fs::read_dir(root)?;

        Self::scan_dir_contents(dir_content);
        return Ok(());
    }

    fn scan_dir_contents(dir_content: ReadDir) {
        dir_content.for_each(|item| match item {
            Err(e) => {
                warn!("dir scanning met problem, e:{:?}", e);
            }
            Ok(entry) => {
                let file_type = entry.file_type();
                if let Ok(file_type) = file_type {
                    if file_type.is_symlink() {
                        trace!(entry = ?entry, "Scan skip symlink file");
                        return;
                    }
                    if file_type.is_dir() {
                        let dir = fs::read_dir(entry.path());
                        if let Ok(dir) = dir {
                            println!("scanning:{:?}", entry.path());
                            Self::scan_dir_contents(dir);
                        }
                        return;
                    }
                    if file_type.is_file() {
                        let path_buf = entry.path();
                        let path = path_buf.as_path();
                        Self::process_file(path);
                    }
                }
            }
        });
    }

    fn process_file(path: &Path) {
        let ext = path.extension();
        match ext {
            Some(ext) => {
                if let Some(ext) = ext.to_str() {
                    // only process img file
                    if IMAGE_EXT.contains(&ext) {}
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {}
