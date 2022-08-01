use chrono::NaiveDateTime;
use rusqlite::{params, Connection, OpenFlags, Result as DbResult, Row};
use std::sync::Arc;

use crate::conf::GlobalConfig;

#[derive(Debug)]
pub struct ImgMetaDO {
    id: Option<String>,
    time: NaiveDateTime,
    sign: String,
}

impl ImgMetaDO {
    pub fn new(time: NaiveDateTime, sign: String) -> ImgMetaDO {
        ImgMetaDO {
            id: None,
            time,
            sign,
        }
    }

    pub fn with_id(id: String, time: NaiveDateTime, sign: String) -> ImgMetaDO {
        let mut img = Self::new(time, sign);
        img.id = Some(id);
        img
    }
}

pub struct ImgMetaDao {
    conf: Arc<GlobalConfig>,
}

impl ImgMetaDao {
    pub fn create_table(conn: &Connection) -> DbResult<usize> {
        let res = conn.execute(
            "CREATE TABLE img_meta (id primary key, time, timestamp, sign)",
            [],
        );
        res
    }

    pub fn batch_write(domains: Vec<ImgMetaDO>, conn: &Connection) -> Result<(), anyhow::Error> {
        for domain in domains {
            tracing::info!("domain.time:{:?}", domain.time);

            let (time, stamp) = Self::gen_time(domain.time);

            if domain.id.is_none() {
                conn.execute(
                    "INSERT INTO img_meta(id, time, timestamp, sign) values (?1, ?2, ?3, ?4)",
                    params![Self::gen_uuid(), time, stamp, domain.sign],
                )
            } else {
                conn.execute(
                    "INSERT INTO img_meta(id, time, timestamp, sign) values (?1, ?2, ?3, ?4)",
                    params![domain.id.unwrap(), time, stamp, domain.sign],
                )
            }?;
        }
        Ok(())
    }

    pub fn query_all(
        offset: usize,
        limit: usize,
        conn: &Connection,
    ) -> Result<Vec<ImgMetaDO>, anyhow::Error> {
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
            id: Some(id),
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
    use crate::util::init_console_logger;

    use super::*;
    use rand::{thread_rng, Rng};

    fn set_up() {
        tracing::info!("Setting up");
        init_console_logger();

        tracing::info!("Creating table...");
    }

    fn create_table(conn: &Connection) {
        ImgMetaDao::create_table(&conn).unwrap();
    }

    fn gen_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn
    }

    fn tear_down() {
        tracing::info!("Tearing down");
    }

    fn gen_test_datas(len: usize) -> Vec<ImgMetaDO> {
        let mut rng = thread_rng();

        let mut metas = Vec::new();
        let cur = chrono::Local::now();

        let time = cur.naive_local();

        for i in 1..(len + 1) {
            let sign = uuid::Uuid::new_v4();

            let sub_millis = rng.gen_range(0..10000i64);
            let time = time
                .checked_sub_signed(chrono::Duration::milliseconds(sub_millis))
                .expect("Gen test naive datetime");

            let meta = ImgMetaDO::with_id(i.to_string(), time, sign.to_string());
            metas.push(meta);
        }
        metas
    }

    #[test]
    fn test_basic_db() -> DbResult<()> {
        set_up();

        let conn = Connection::open_in_memory()?;
        create_table(&conn);

        let meta_datas = gen_test_datas(100);

        assert!(ImgMetaDao::batch_write(meta_datas, &conn).is_ok());

        let mut id = 1;

        for loop_num in 0..10 {
            let img_metas = ImgMetaDao::query_all(loop_num * 10, 10, &conn).unwrap();
            for img in img_metas {
                assert_eq!(id.to_string(), img.id.unwrap());

                id += 1;
            }
        }
        tear_down();
        Ok(())
    }
}
