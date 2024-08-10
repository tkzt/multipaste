use chrono::Local;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    params,
    types::{FromSqlError, Type as RSType},
    Error as RusqliteError, Result,
};
use serde::Serialize;
use std::{path::Path, sync::Arc};
use tauri::{App, Manager, State};

type DbPool = Arc<Pool<SqliteConnectionManager>>;

const DB_PATH: &str = "data.db";

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

impl PartialEq for RecordType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
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
        let path = Path::new(db_url);
        if let Some(path) = path.parent() {
            let _ = std::fs::create_dir_all(path);
        }
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
            pinned integer not null default 0 check (pinned in (0, 1))
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
        let updated_rows = conn.execute(
            "
            update clipboard_record
            set updated_at = ?3
            where
                case
                    when ?2 is null then record_value = ?1
                    else hash = ?2
                end
            ",
            params![record_value, hash, Local::now().timestamp()],
        )?;

        if updated_rows == 0 {
            conn.execute(
                "
                insert into clipboard_record (record_type, record_value, hash, updated_at)
                values (?1, ?2, ?3, ?4)
                ",
                params![
                    record_type.to_string(),
                    record_value,
                    hash,
                    Local::now().timestamp()
                ],
            )?;
        }
        Ok(())
    }

    pub fn pin(&self, id: &u64) -> Result<()> {
        let conn = self.get_conn();
        conn.execute("update clipboard_record set pinned = 1 where id = ?1", [id])?;
        Ok(())
    }

    pub fn unpin(&self, id: &u64) -> Result<()> {
        let conn = self.get_conn();
        conn.execute("update clipboard_record set pinned = 0 where id = ?1", [id])?;
        Ok(())
    }

    pub fn delete(&self, id: &u64) -> Result<()> {
        let conn = self.get_conn();
        conn.execute("delete from clipboard_record where id = ?1", [id.clone()])?;
        Ok(())
    }

    pub fn get_records(&self, keyword: &str) -> Result<Vec<ClipboardRecord>> {
        let conn = self.get_conn();
        let mut stmt = conn.prepare("
            select
                id, record_type, record_value, hash, updated_at, pinned
            from clipboard_record
            where
                lower(record_value) like ?1
            order by
                pinned desc, updated_at desc
        ")?;

        let records = stmt.query_map(
            [format!("%{}%", keyword.to_lowercase())],
            |row| {
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

    pub fn get_record(&self, id: &u64) -> Result<ClipboardRecord> {
        let conn = self.get_conn();
        let row = conn.query_row(
            "
                select
                    id, record_type, record_value, hash, updated_at, pinned
                from clipboard_record
                where id = ?1
            ",
            [id],
            |row| {
                Ok(ClipboardRecord {
                    id: row.get(0)?,
                    record_type: RecordType::from_string(&row.get::<_, String>(1)?)?,
                    record_value: row.get(2)?,
                    hash: row.get(3)?,
                    updated_at: row.get::<_, u64>(4)?,
                    pinned: row.get::<_, bool>(5)?,
                })
            },
        )?;
        Ok(row)
    }
}

pub fn init(app: &App) -> Result<Arc<RecordStore>, Box<dyn std::error::Error>> {
    let db_url = if tauri::is_dev() {
        DB_PATH
    } else {
        let db_path = app.path().app_data_dir().unwrap().join(DB_PATH);
        &db_path.to_string_lossy().to_string()
    };
    let store = Arc::new(RecordStore::new(db_url));
    store.init()?;
    return Ok(store);
}

#[tauri::command]
pub fn pin_record(store: State<Arc<RecordStore>>, id: u64) {
    store.pin(&id).unwrap();
}

#[tauri::command]
pub fn unpin_record(store: State<Arc<RecordStore>>, id: u64) {
    store.unpin(&id).unwrap();
}

#[tauri::command]
pub fn delete_record(store: State<Arc<RecordStore>>, id: u64) {
    store.delete(&id).unwrap();
}

#[tauri::command]
pub fn filter_records(store: State<Arc<RecordStore>>, keyword: String) -> Vec<ClipboardRecord> {
    store.get_records(&keyword).unwrap()
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
    fn test_02_get_record() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();

        let record = store.get_record(&data.record_id)?;
        assert_eq!(record.id, data.record_id);

        let records = store.get_records("foo")?;
        assert!(records.iter().any(|record| record.record_value == "cached/foo.png"));

        let records = store.get_records("Dolor culpa ut nisi qui veniam proident Lorem proident enim ea. Consequat adipisicing officia consectetur do sit deserunt. Veniam nostrud laboris ipsum sunt deserunt ex nulla minim nostrud voluptate consequat excepteur. Consequat tempor sint adipisicing minim anim. Ad eu nisi id in culpa qui ut eiusmod minim veniam ea. Esse non voluptate eiusmod officia duis consectetur dolore eu nulla ullamco labore id nulla.")?;
        assert_eq!(records.len(), 0);
        Ok(())
    }

    #[test]
    fn test_03_toggle_pinned() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.pin(&data.record_id)?;
        let records = store.get_records("").ok().unwrap();
        let [record, ..] = records.as_slice() else {
            panic!("Empty records")
        };
        assert_eq!(record.id, data.record_id);
        assert_eq!(record.pinned, true);

        store.unpin(&data.record_id)?;
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
    fn test_04_delete_record() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.delete(&data.record_id)?;
        let result: Result<u64, Error> = store.get_conn().query_row(
            "select id from clipboard_record where id = ?1",
            [data.record_id],
            |r| r.get(0),
        );
        assert!(matches!(result, Err(Error::QueryReturnedNoRows)));
        Ok(())
    }
}
