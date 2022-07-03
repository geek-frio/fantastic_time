use crate::img::*;
use anyhow::Error as AnyError;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs::{self, ReadDir};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use tracing::{trace, warn};

pub struct DirsScanner {
    sender: Sender<PathBuf>,
}

impl DirsScanner {
    pub fn new() -> (DirsScanner, Receiver<PathBuf>) {
        let (sender, receiver) = std::sync::mpsc::channel();
        let scanner = DirsScanner { sender };
        (scanner, receiver)
    }

    pub fn scan_entry<P>(&self, root: P) -> Result<(), AnyError>
    where
        P: AsRef<OsStr>,
    {
        let root = Path::new(root.as_ref());
        let dir_content = fs::read_dir(root)?;

        self.scan_dir_contents(dir_content);
        return Ok(());
    }

    pub fn scan_dir_contents(&self, dir_content: ReadDir) {
        dir_content.for_each(|item| match item {
            Err(e) => {
                warn!("dir scanning met problem, e:{:?}", e);
            }

            Ok(entry) => {
                let file_type = entry.file_type();

                if let Ok(file_type) = file_type {
                    if file_type.is_symlink() {
                        trace!(entry = ?entry, "Scan skip symlink file");
                    } else if file_type.is_dir() {
                        let dir = fs::read_dir(entry.path());

                        if let Ok(dir) = dir {
                            println!("scanning:{:?}", entry.path());
                            self.scan_dir_contents(dir);
                        }
                    } else if file_type.is_file() {
                        let path_buf = entry.path();
                        let path = path_buf.as_path();

                        let path_buf: PathBuf = path.into();
                        self.sender.send(path_buf).unwrap();
                    }
                }
            }
        });
    }

    pub fn process_file(path: &Path) -> Result<Option<ImgMeta>, AnyError> {
        const IMAGE_EXT: [&'static str; 3] = ["jpg", "jpeg", "png"];
        let ext = path
            .extension()
            .ok_or(AnyError::msg("get extension failed"))?;
        let ext = ext.to_str().ok_or(AnyError::msg("No extension info"))?;

        if !IMAGE_EXT.contains(&ext) {
            return Ok(None);
        }

        let img_meta = retrive_img_datetime(path)?;
        Ok(Some(img_meta))
    }
}
