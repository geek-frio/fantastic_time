use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Result as DbResult};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ImgMetaDO {
    pub(crate) id: i32,
    pub(crate) time: NaiveDateTime,
    pub(crate) sign: String,
}

pub struct ImgMetaDao {
    db_path: PathBuf,
}

impl ImgMetaDao {
    pub fn batch_write(&self, domains: Vec<ImgMetaDO>) -> Result<(), Vec<ImgMetaDO>> {
        let conn = self.get_conn();
        todo!()
    }

    pub fn get_conn(&self) -> Connection {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_db() -> DbResult<()> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE img_meta (id integer primary key, time, sign)",
            [],
        )?;
        let img = ImgMetaDO {
            id: 1,
            time: NaiveDateTime::from_timestamp(111111, 1111),
            sign: "sign".to_string(),
        };

        conn.execute(
            "INSERT INTO img_meta(time, sign) values(?1, ?2)",
            params![img.time, img.sign],
        )?;

        let mut stmt = conn.prepare("select id, time, sign from img_meta")?;
        let img_iter = stmt.query_map([], |row| {
            Ok(ImgMetaDO {
                id: row.get(0)?,
                time: row.get(1)?,
                sign: row.get(2)?,
            })
        })?;

        for img in img_iter {
            println!("Img is:{:?}", img);
        }

        Ok(())
    }
}
