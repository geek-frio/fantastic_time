use chrono::NaiveDateTime;
use rusqlite::{params, Connection, OpenFlags, Result as DbResult};
use std::path::PathBuf;
use std::sync::Arc;

use crate::conf::GlobalConfig;

#[derive(Debug)]
pub struct ImgMetaDO {
    pub(crate) id: i32,
    pub(crate) time: NaiveDateTime,
    pub(crate) sign: String,
}

pub struct ImgMetaDao {
    conf: Arc<GlobalConfig>,
}

impl ImgMetaDao {
    const TABLE_NAME: &'static str = "img_meta";

    pub fn init(&self) {
        let conn = self.get_conn();

        let sql = format!("select 1 from {}", Self::TABLE_NAME);
        let res = conn.execute(sql.as_str(), []);

        if let Err(_) = res {
            Self::create_table(&conn).expect("初始化图片Meta仓库失败");
        }
    }

    pub fn create_table(conn: &Connection) -> DbResult<usize> {
        let res = conn.execute(
            "CREATE TABLE img_meta (id integer primary key, time, sign)",
            [],
        );
        res
    }

    pub fn batch_write(&self, domains: Vec<ImgMetaDO>) -> Result<(), Vec<ImgMetaDO>> {
        let conn = self.get_conn();

        // let mut stmt = conn.prepare("INSERT INTO ")
        // conn.execute(sql, params)
        todo!()
    }

    pub fn get_conn(&self) -> Connection {
        let path_buf = self.conf.db_path();

        Connection::open_with_flags(
            &path_buf,
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .expect("建立fantasy档案库连接失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_conf() -> GlobalConfig {
        let conf = GlobalConfig {
            meta_path: "C:\\Users\\frio\\tmp".to_string(),
        };
    }

    #[test]
    fn test_init() {
        let conf = Arc::new(init_conf());
        let dao = ImgMetaDao { conf };

        dao.init();
    }

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
