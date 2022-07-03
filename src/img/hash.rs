use sha2::{Digest, Sha256};
use std::path::Path;

pub fn hashcode_file(path: &Path) -> Result<String, anyhow::Error> {
    let mut hasher = Sha256::new();

    let mut file = std::fs::File::open(path)?;
    let n = std::io::copy(&mut file, &mut hasher)?;

    let hash = hasher.finalize();
    todo!()
}
