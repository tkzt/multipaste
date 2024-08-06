use chrono::Local;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    params,
    types::{FromSqlError, Type as RSType},
    Error as RusqliteError, Result,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{App, Manager, State};

type DbPool = Arc<Pool<SqliteConnectionManager>>;

const DB_PATH: &str = "./data.db";

#[derive(Debug, Serialize)]
pub enum RecordType {
    Image,
    Text,
}

impl ToString for RecordType {
    fn to_string(&self) -> String {
        match self {
            RecordType::Image => "image".to_string(),
            RecordType::Text => "text".to_string(),
        }
    }
}

impl RecordType {
    fn from_string(s: &str) -> Result<RecordType, RusqliteError> {
        match s {
            "image" => Ok(RecordType::Image),
            "text" => Ok(RecordType::Text),
            _ => Err(RusqliteError::FromSqlConversionFailure(
                0,
                RSType::Text,
                Box::new(FromSqlError::Other("Invalid record type".into())),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RecordStore {
    pool: DbPool,
}

#[derive(Debug, Serialize)]
pub struct ClipboardRecord {
    pub id: u64,
    pub record_type: RecordType,
    // For image, the record value will be like
    // <cache_dir>/<md5_hash>.png
    pub record_value: String,
    pub hash: Option<String>,
    pub updated_at: u64,
    pub pinned: bool,
}

impl RecordStore {
    pub fn new(db_url: &str) -> Self {
        let manager = SqliteConnectionManager::file(db_url);
        let pool = Pool::new(manager).expect("Failed to create pool.");
        RecordStore {
            pool: Arc::new(pool),
        }
    }

    pub fn get_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.pool
            .get()
            .expect("Failed to get a connection from the pool.")
    }

    pub fn init(&self) -> Result<()> {
        let conn = self.get_conn();
        conn.execute_batch(
            "
        create table if not exists clipboard_record (
            id integer primary key autoincrement,
            record_type text not null check (record_type in ('image', 'text')),
            record_value text not null unique,
            hash varchar(32) unique,
            updated_at integer not null,
            pinned integer not null default 0 check (pinned in (0, 1)),
            unique(record_value, hash)
        );
        create index if not exists idx_hash on clipboard_record(hash);
        ",
        )?;
        Ok(())
    }

    pub fn save(
        &self,
        record_type: &RecordType,
        record_value: &str,
        hash: Option<&str>,
    ) -> Result<()> {
        let conn = self.get_conn();
        conn.execute(
            "
        insert into clipboard_record (record_type, record_value, hash, updated_at)
        values (?1, ?2, ?3, ?4)
        on conflict(record_value, hash) do update set
          updated_at = ?4
      ",
            params![
                record_type.to_string(),
                record_value,
                hash,
                Local::now().timestamp()
            ],
        )?;
        Ok(())
    }

    pub fn pin_record(&self, id: u64) -> Result<()> {
        let conn = self.get_conn();
        conn.execute("update clipboard_record set pinned = 1 where id = ?1", [id])?;
        Ok(())
    }

    pub fn unpin_record(&self, id: u64) -> Result<()> {
        let conn = self.get_conn();
        conn.execute("update clipboard_record set pinned = 0 where id = ?1", [id])?;
        Ok(())
    }

    pub fn delete(&self, id: u64) -> Result<()> {
        let conn = self.get_conn();
        conn.execute("delete from clipboard_record where id = ?1", [id.clone()])?;
        Ok(())
    }

    pub fn get_records(&self) -> Result<Vec<ClipboardRecord>> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare(
            "select id, record_type, record_value, hash, updated_at, pinned
        from clipboard_record order by pinned desc, updated_at desc",
        )?;

        let records = stmt.query_map((), |row| {
            Ok(ClipboardRecord {
                id: row.get(0)?,
                record_type: RecordType::from_string(&row.get::<_, String>(1)?)?,
                record_value: row.get(2)?,
                hash: row.get(3)?,
                updated_at: row.get::<_, u64>(4)?,
                pinned: row.get::<_, bool>(5)?,
            })
        })?;

        let clipboard_record = records.collect::<Result<Vec<ClipboardRecord>>>()?;
        Ok(clipboard_record)
    }
}

pub fn init(app: &App) -> Result<Arc<Mutex<RecordStore>>, Box<dyn std::error::Error>> {
    let store = Arc::new(Mutex::new(RecordStore::new(DB_PATH)));
    store.lock().unwrap().init()?;
    app.handle().manage(store.clone());
    Ok(store)
}

#[tauri::command]
pub fn get_clipboard_records(store: State<Arc<Mutex<RecordStore>>>) -> Vec<ClipboardRecord> {
    let store = store.lock().unwrap();
    store.get_records().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Error;
    use std::{sync::Mutex, thread::sleep, time::Duration};

    struct SharedData {
        record_id: u64,
    }

    lazy_static::lazy_static! {
        static ref SHARED_STORE: Mutex<RecordStore> = Mutex::new(RecordStore::new("./data.db"));
        static ref SHARED_DATA: Mutex<SharedData> = Mutex::new(SharedData {record_id: 0});
    }

    #[test]
    fn test_01_save_text() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let mut data = SHARED_DATA.lock().unwrap();
        store.init()?;
        let value = "cached/foo.png";

        store.save(&RecordType::Image, value, Some("foo"))?;
        let conn = store.get_conn();

        let (record_id, updated_at) = conn
            .query_row(
                "select id, updated_at from clipboard_record where record_value = ?1",
                [value],
                |row| {
                    let id: u64 = row.get(0)?;
                    let updated_at: u64 = row.get(1)?;
                    Ok((id, updated_at))
                },
            )
            .unwrap();
        assert!(record_id > 0);
        sleep(Duration::from_secs(1));

        store.save(&RecordType::Image, value, Some("foo"))?;
        let (record_id_repeat, updated_at_repeat) = conn
            .query_row(
                "select id, updated_at from clipboard_record where record_value = ?1",
                [value],
                |row| {
                    let id: u64 = row.get(0)?;
                    let updated_at: u64 = row.get(1)?;
                    Ok((id, updated_at))
                },
            )
            .unwrap();
        assert_eq!(record_id, record_id_repeat);
        assert_ne!(updated_at, updated_at_repeat);

        data.record_id = record_id;
        Ok(())
    }

    #[test]
    fn test_02_toggle_pinned() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.pin_record(data.record_id.clone())?;
        let records = store.get_records().ok().unwrap();
        let [record, ..] = records.as_slice() else {
            panic!("Empty records")
        };
        assert_eq!(record.id, data.record_id);
        assert_eq!(record.pinned, true);

        store.unpin_record(data.record_id.clone())?;
        let the_record_pinned: bool = store
            .get_conn()
            .query_row(
                "select pinned from clipboard_record where id = ?1",
                [data.record_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(the_record_pinned, false);
        Ok(())
    }

    #[test]
    fn test_03_delete_record() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.delete(data.record_id)?;
        let result: Result<u64, Error> = store.get_conn().query_row(
            "select id from clipboard_record where id = ?1",
            [data.record_id],
            |r| r.get(0),
        );
        assert!(matches!(result, Err(Error::QueryReturnedNoRows)));
        Ok(())
    }
}
