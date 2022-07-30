use chrono::NaiveDateTime;
use rusqlite::{params, Connection, OpenFlags, Result as DbResult, Row};
use std::sync::Arc;

use crate::conf::GlobalConfig;

#[derive(Debug)]
pub struct ImgMetaDO {
    pub(crate) id: Option<String>,
    pub(crate) time: NaiveDateTime,
    pub(crate) sign: String,
}

pub struct ImgMetaDao {
    conf: Arc<GlobalConfig>,
}

impl ImgMetaDao {
    pub fn create_table(conn: &Connection) -> DbResult<usize> {
        let res = conn.execute(
            "CREATE TABLE img_meta (id integer primary key, time, timestamp, sign)",
            [],
        );
        res
    }

    pub fn batch_write(&self, domains: Vec<ImgMetaDO>) -> Result<(), Vec<ImgMetaDO>> {
        let conn = self.get_conn();

        let mut failed_records = Vec::new();

        for domain in domains {
            let (time, stamp) = Self::gen_time(domain.time);

            let res = conn.execute(
                "INSERT INTO img_meta(id, time, timestamp, sign) values (?1, ?2, ?3, ?4)",
                params![Self::gen_uuid(), time, stamp, domain.sign],
            );

            if let Err(_e) = res {
                tracing::error!("Push img meta records failed!");
                failed_records.push(domain);
            }
        }

        if failed_records.len() > 0 {
            Err(failed_records)
        } else {
            Ok(())
        }
    }

    pub fn query_all(&self, offset: usize, limit: usize) -> Result<Vec<ImgMetaDO>, anyhow::Error> {
        let conn = self.get_conn();

        let mut stmt = conn.prepare("select id, timestamp, sign from img_meta limit ?, ?")?;
        let mut recs = stmt.query([offset, limit])?;

        let mut img_metas = Vec::new();

        while let Some(row) = recs.next()? {
            img_metas.push(Self::convert_to_img_meta_do(row));
        }

        Ok(img_metas)
    }

    fn convert_to_img_meta_do(row: &Row) -> ImgMetaDO {
        let id: String = row.get(0).unwrap();

        let timestamp: i64 = row.get(1).unwrap();

        let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);
        let sign = row.get(2).unwrap_or("".to_string());

        ImgMetaDO {
            id,
            time: naive_datetime,
            sign,
        }
    }

    fn gen_time(time: NaiveDateTime) -> (String, i64) {
        (
            time.format("%Y-%m-%d %H:%M:%S").to_string(),
            time.timestamp(),
        )
    }

    fn gen_uuid() -> String {
        let uuid = uuid::Uuid::new_v4();
        uuid.to_string()
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
    use rusqlite::params;

    fn set_up() {}

    #[test]
    fn test_basic_db() -> DbResult<()> {
        let conn = Connection::open_in_memory()?;

        Ok(())
    }
}
