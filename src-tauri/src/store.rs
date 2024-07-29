use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
use chrono::Local;
use rusqlite::{types::{Type as RSType, FromSqlError}, Error, Result};

type DbPool = Arc<Pool<SqliteConnectionManager>>;


#[derive(Debug)]
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
  fn from_string(s: &str) -> Result<RecordType, Error> {
    match s {
      "image" => Ok(RecordType::Image),
      "text" => Ok(RecordType::Text),
      _ => Err(Error::FromSqlConversionFailure(0, RSType::Text, Box::new(FromSqlError::Other("Invalid record type".into())))),
    }
  }
}


#[derive(Debug)]
pub struct RecordStore {
  pool: DbPool
}


#[derive(Debug)]
pub struct ClipboardRecord {
  pub id: u64,
  pub record_type: RecordType,
  // For image, the record value will be like
  // <cache_dir>/<md5_hash>.png
  pub record_value: Option<String>,
  pub updated_at: u64,
  pub pinned: bool
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
      self.pool.get().expect("Failed to get a connection from the pool.")
  }

  pub fn init(&self) -> Result<()> {
    let conn = self.get_conn();
    conn.execute(
        "create table if not exists clipboard_record (
            id integer primary key autoincrement,
            record_type text not null check (record_type in ('image', 'text')),
            record_value text not null,
            updated_at integer not null,
            pinned integer not null default 0 check (pinned in (0, 1))
        )",
        ()
    )?;
    Ok(())
  }

  pub fn save(&self, value: String) -> Result<()> {
    let conn = self.get_conn();
    conn.execute(
        "insert into clipboard_record (record_type, record_value, updated_at) values (?1, ?2, ?3)",
        [RecordType::Text.to_string(), value, Local::now().timestamp().to_string()],
    )?;
    Ok(())
  }

  pub fn pin_record(&self, id: u64) -> Result<()> {
    let conn = self.get_conn();
    conn.execute(
        "update clipboard_record set pinned = 1 where id =?1",
        [id],
    )?;
    Ok(())
  }

  pub fn unpin_record(&self, id: u64) -> Result<()> {
    let conn = self.get_conn();
    conn.execute(
        "update clipboard_record set pinned = 0 where id =?1",
        [id],
    )?;
    Ok(())
  }

  pub fn delete(&self, id: u64) -> Result<()> {
    let conn = self.get_conn();
    conn.execute(
        "delete from clipboard_record where id = ?1",
        [id.clone()],
    )?;
    Ok(())
  }

  pub fn check_img_repeat(&self, hash: &str) -> Result<bool> {
    let conn = self.get_conn();
    let count: u64 = conn.query_row(
        "select count(*) from clipboard_record where record_type = 'image' and record_value like %/?1.%",
        [hash],
        |r| r.get(0),
    )?;
    Ok(count > 0)
  }

  pub fn get_records(&self) -> Result<Vec<ClipboardRecord>> {
    let conn = self.get_conn();
    let mut stmt = conn.prepare(
        "select id, record_type, record_value, updated_at, pinned
        from clipboard_record order by pinned desc, updated_at asc",
    )?;

    let records = stmt.query_map((), |row| {
      Ok(ClipboardRecord {
          id: row.get(0)?,
          record_type: RecordType::from_string(&row.get::<_, String>(1)?)?,
          record_value: row.get(2)?,
          updated_at: row.get::<_, u64>(3)?,
          pinned: row.get::<_, bool>(4)?
      })
    })?;

    let clipboard_record = records.collect::<Result<Vec<ClipboardRecord>>>()?;
    Ok(clipboard_record)
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  use rusqlite::Error;
  use std::sync::Mutex;

  struct SharedData {
    record_id: u64
  }
  
  lazy_static::lazy_static! {
      static ref SHARED_STORE: Mutex<RecordStore> = Mutex::new(RecordStore::new("./data.db"));
      static ref SHARED_DATA: Mutex<SharedData> = Mutex::new(SharedData {record_id: 0});
  }
  
  #[test]
  fn test_01_save_text() {
    let store = SHARED_STORE.lock().unwrap();
    let mut data = SHARED_DATA.lock().unwrap();
    let _ = store.init();
    let value = "cached/foo.png";

    let _ = store.save(value.to_string());
    let conn = store.get_conn();
  
    let the_record = conn.query_row(
      "select id from clipboard_record where record_value = '?1'",
      [value],
      |r| {
        r.get(0)
      }
    ).unwrap();
    assert!(the_record > 0);
  
    data.record_id = the_record;
  }
  
  
  #[test]
  fn test_02_toggle_pinned() {
    let store = SHARED_STORE.lock().unwrap();
    let data = SHARED_DATA.lock().unwrap();
    let _ = store.pin_record(data.record_id.clone());
    let records = store.get_records().ok().unwrap();
    let [record, ..] = records.as_slice() else {
      panic!("Empty records")
    };
    assert_eq!(record.id, data.record_id);
    assert_eq!(record.pinned, true);
  
    let _ = store.unpin_record(data.record_id.clone());
    let the_record_pinned: bool = store.get_conn().query_row(
      "select pinned from clipboard_record where id = ?1",
      [data.record_id],
      |r|{
        r.get(0)
      }
    ).unwrap();
    assert_eq!(the_record_pinned, false);
  }

  #[test]
  fn test_03_check_repeat() {
    let store = SHARED_STORE.lock().unwrap();
    let is_repeat = store.check_img_repeat("foo");
    assert!(is_repeat.unwrap());
  }
  
  #[test]
  fn test_04_delete_record() {
    let store = SHARED_STORE.lock().unwrap();
    let data = SHARED_DATA.lock().unwrap();
    let _ = store.delete(data.record_id);
    let result: Result<u64, Error> = store.get_conn().query_row(
      "select id from clipboard_record where id = ?1",
      [data.record_id],
      |r| r.get(0)
    );
    assert!(matches!(result, Err(Error::QueryReturnedNoRows)));
  }
}