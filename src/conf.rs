use std::path::PathBuf;

const META_DB_NAME: &'static str = "fantasy_db";

pub(crate) struct GlobalConfig {
    pub(crate) meta_path: String,
}

impl GlobalConfig {
    pub(crate) fn db_path(&self) -> PathBuf {
        let mut path_buf = PathBuf::from(self.meta_path.as_str());
        path_buf.push(META_DB_NAME);

        path_buf
    }
}
