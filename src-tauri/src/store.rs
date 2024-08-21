use chrono::Local;
use crypto::{digest::Digest, sha2::Sha256};
use log::warn;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    params,
    types::{FromSqlError, Type as RSType},
    Error as RusqliteError, Result,
};
use serde::{Serialize, Serializer};
use std::{
    fs, path::{Path, PathBuf}, sync::{Arc, Mutex}
};
use tauri::{App, Manager, State};
use glob::glob;

use crate::conf::Config;

type DbPool = Arc<Pool<SqliteConnectionManager>>;

const DB_PATH: &str = "data.db";
const IMG_DIR_PATH: &str = "images";
const MIN_TEXT_HASHING_SIZE: usize = 50;

#[derive(Debug)]
pub enum RecordType {
    Image,
    Text,
}

impl Serialize for RecordType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
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
    pub img_dir: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct ClipboardRecord {
    pub id: u64,
    pub record_type: RecordType,
    // For image, the record value will be like
    // <IMG_DIR>/<sha256_hash>.png
    pub record_value: String,
    pub hash: Option<String>,
    pub updated_at: u64,
    pub pinned: bool,
}

impl RecordStore {
    pub fn new(db_url: PathBuf, img_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&img_dir).unwrap();
        if let Some(path) = db_url.parent() {
            std::fs::create_dir_all(path).unwrap();
        }
        let manager = SqliteConnectionManager::file(db_url);
        let pool = Pool::new(manager).expect("Failed to create pool.");
        RecordStore {
            pool: Arc::new(pool),
            img_dir,
        }
    }

    pub fn get_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.pool
            .get()
            .expect("Failed to get a connection from the pool.")
    }

    pub fn update_max_records_trigger(&self, max_records: u64) -> Result<()> {
        let conn = self.get_conn();
        let stmt = &format!(
            "
            drop trigger if exists limit_records_amount;
            create trigger limit_records_amount
            after insert on clipboard_record
            begin
                delete from clipboard_record
                where id in (
                    select id from clipboard_record
                    order by pinned desc, updated_at desc limit -1 offset {0}
                ) and (select count(*) from clipboard_record) > {0};
            end;",
            max_records
        );
        conn.execute_batch(stmt)?;
        Ok(())
    }

    pub fn init(&self, max_records: u64) -> Result<()> {
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
        self.update_max_records_trigger(max_records)?;
        Ok(())
    }

    fn calc_hash(&self, bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.input(bytes);
        hasher.result_str()
    }

    fn save(
        &self,
        record_type: &RecordType,
        record_value: &str,
        hash: Option<&str>,
    ) -> Result<bool> {
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
        Ok(updated_rows > 0)
    }

    pub fn save_text(&self, text: &str) -> Result<()> {
        let text_hash = if text.len() <= MIN_TEXT_HASHING_SIZE {
            None
        } else {
            let text_hash = self.calc_hash(text.as_bytes());
            Some(text_hash)
        };
        self.save(&RecordType::Text, text, text_hash.as_deref())?;
        Ok(())
    }

    fn filter_dangling_images(&self, hashes: &Vec<String>) -> Result<Vec<String>> {
        let conn = self.get_conn();
        let hash_values: Vec<String> = hashes.iter().map(|h| format!("('{}')", h)).collect();
        let batch_stmt = format!("
            drop table if exists temp_hashes;
            create table temp_hashes (hash varchar(32));
            insert into temp_hashes (hash) values {};
        ", hash_values.join(","));
        conn.execute_batch(&batch_stmt)?;
        let mut filter_stmt = conn.prepare("
            select temp_hashes.hash
            from temp_hashes
            left join clipboard_record on temp_hashes.hash = clipboard_record.hash
            where clipboard_record.hash is null;
        ")?;
        let image_hashes = filter_stmt.query_map([], |row| {
            row.get::<_, String>(0)
        })?;
        Ok(image_hashes.flatten().collect::<Vec<String>>())
    }

    fn clean_dangling_images(&self) -> Result<()> {
        if let Some(img_dir) = self.img_dir.to_str() {
            if let Ok(paths) = glob(&format!("{}/*.png", img_dir)) {
                let hashes: Vec<String> = paths
                    .flatten()
                    .filter_map(
                        |image_path| image_path.file_stem().and_then(
                            |p| p.to_str()
                        ).map(|s| s.to_owned())
                    ).collect();
                let dangling_hashes = self.filter_dangling_images(&hashes)?;
                dangling_hashes.iter().for_each(|hash| {
                    let image_path = Path::new(&format!("{}/{}.png", img_dir, hash)).to_path_buf();
                    if image_path.exists() {
                        if let Err(_) = fs::remove_file(&image_path) {
                            warn!("Failed to remove file {:?}", image_path);
                        }
                    }
                })
            } else {
                warn!("Failed to glob images")
            }
        } else {
            warn!("Failed to get image directory")
        }
        Ok(())
    }

    pub fn save_image(&self, image_bytes: &[u8]) -> Result<()> {
        let image_hash = self.calc_hash(image_bytes);
        let image_path = self.img_dir.join(format!("{}.png", image_hash));

        let exists = self.save(
            &RecordType::Image,
            image_path.to_str().unwrap(),
            Some(&image_hash),
        )?;
        if !exists {
            std::fs::write(&image_path, image_bytes).unwrap();
        }

        self.clean_dangling_images()?;

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
        let mut stmt = conn.prepare(
            "
            select
                id, record_type, record_value, hash, updated_at, pinned
            from clipboard_record
            where
                lower(record_value) like ?1
            order by
                pinned desc, updated_at desc
        ")?;

        let records = stmt.query_map([format!("%{}%", keyword.to_lowercase())], |row| {
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
    let config = app.state::<Mutex<Config>>();
    let app_data_path = app.path().app_data_dir().unwrap();
    let db_url = app_data_path.join(DB_PATH);
    let img_dir = app_data_path.join(IMG_DIR_PATH);
    let store = Arc::new(RecordStore::new(db_url, img_dir));
    store.init(config.lock().unwrap().max_items)?;
    app.manage(store.clone());
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
    let record = store.get_record(&id).unwrap();
    if record.record_type == RecordType::Image {
        std::fs::remove_file(store.img_dir.join(record.record_value)).unwrap();
    }
    store.delete(&id).unwrap();
}

#[tauri::command]
pub fn filter_records(store: State<Arc<RecordStore>>, keyword: String) -> Vec<ClipboardRecord> {
    store.get_records(&keyword).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, ImageOutputFormat, Rgba};
    use rusqlite::Error;
    use std::{io::Cursor, path::Path, sync::Mutex, thread::sleep, time::Duration};

    const TEXT_VALUE: &str = "foo";
    const SOME_TEXT_NOT_IN_DB: &str = "Dolor culpa ut nisi qui veniam proident Lorem proident enim ea. Consequat adipisicing officia consectetur do sit deserunt. Veniam nostrud laboris ipsum sunt deserunt ex nulla minim nostrud voluptate consequat excepteur. Consequat tempor sint adipisicing minim anim. Ad eu nisi id in culpa qui ut eiusmod minim veniam ea. Esse non voluptate eiusmod officia duis consectetur dolore eu nulla ullamco labore id nulla.";

    struct SharedData {
        text_record_id: u64,
        img_record_id: u64,
    }

    lazy_static::lazy_static! {
        static ref SHARED_STORE: Mutex<RecordStore> = Mutex::new(RecordStore::new(
            Path::new(DB_PATH).to_path_buf(),
            Path::new(IMG_DIR_PATH).to_path_buf())
        );
        static ref SHARED_DATA: Mutex<SharedData> = Mutex::new(SharedData {text_record_id: 0, img_record_id: 0 });
    }

    #[test]
    fn test_01_save() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let mut data = SHARED_DATA.lock().unwrap();
        store.init(2)?;
        let conn = store.get_conn();

        // text
        store.save_text(TEXT_VALUE)?;
        let record_id = conn
            .query_row(
                "
                select
                    id
                from
                    clipboard_record
                where
                    record_value = ?1
                    and record_type = 'text'
                ",
                [TEXT_VALUE],
                |row| {
                    let id: u64 = row.get(0)?;
                    Ok(id)
                },
            )
            .unwrap();
        assert!(record_id > 0);

        // image
        let img_value = ImageBuffer::from_fn(8, 8, |x, y| {
            if (x * y) % 2 == 0 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 255, 0, 255])
            }
        });
        let dynamic_image = DynamicImage::ImageRgba8(img_value);
        let mut img_bytes: Vec<u8> = Vec::new();
        dynamic_image
            .write_to(&mut Cursor::new(&mut img_bytes), ImageOutputFormat::Png)
            .unwrap();

        store.save_image(&img_bytes)?;
        let img_hash = store.calc_hash(&img_bytes);
        let query_image_res = || -> (u64, u64) {
            conn.query_row(
                "
                select
                    id, updated_at
                from
                    clipboard_record
                where
                    hash = ?1
                    and record_type = 'image'
                ",
                [&img_hash],
                |row| {
                    let id: u64 = row.get(0)?;
                    let updated_at: u64 = row.get(1)?;
                    Ok((id, updated_at))
                },
            )
            .unwrap()
        };
        let (img_record_id, updated_at) = query_image_res();
        assert!(record_id > 0);

        // Make sure the time interval between the first saving and the second
        // is greater than 1 second.
        sleep(Duration::from_secs(1));

        // Check repeat saving
        store.save_image(&img_bytes)?;
        let (img_record_id_repeat, updated_at_repeat) = query_image_res();
        assert_eq!(img_record_id, img_record_id_repeat);
        assert_ne!(updated_at, updated_at_repeat);

        data.text_record_id = record_id;
        data.img_record_id = img_record_id;
        Ok(())
    }

    #[test]
    fn test_02_get_record() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();

        let record = store.get_record(&data.text_record_id)?;
        assert_eq!(record.id, data.text_record_id);

        let records = store.get_records(TEXT_VALUE)?;
        assert!(records
            .iter()
            .any(|record| record.record_value == TEXT_VALUE));

        let records = store.get_records(SOME_TEXT_NOT_IN_DB)?;
        assert_eq!(records.len(), 0);
        Ok(())
    }

    #[test]
    fn test_03_toggle_pinned() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.pin(&data.text_record_id)?;
        let records = store.get_records("").ok().unwrap();
        let [record, ..] = records.as_slice() else {
            panic!("Empty records")
        };
        assert_eq!(record.id, data.text_record_id);
        assert_eq!(record.pinned, true);

        store.unpin(&data.text_record_id)?;
        let the_record_pinned: bool = store
            .get_conn()
            .query_row(
                "select pinned from clipboard_record where id = ?1",
                [data.text_record_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(the_record_pinned, false);
        Ok(())
    }

    #[test]
    fn test_04_max_records() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let mut data = SHARED_DATA.lock().unwrap();

        let text_value = &format!("{}{}", TEXT_VALUE, TEXT_VALUE);
        store.save_text(text_value)?;

        let result = store.get_record(&data.text_record_id);
        assert!(matches!(result, Err(Error::QueryReturnedNoRows)));

        let newly_saved_record_id = store
            .get_conn()
            .query_row(
                "select id from clipboard_record where record_value = ?1",
                [text_value],
                |r| r.get::<_, u64>(0),
            )
            .unwrap();
        assert!(newly_saved_record_id > 0);

        data.text_record_id = newly_saved_record_id;
        Ok(())
    }

    #[test]
    fn test_05_delete_record() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.delete(&data.text_record_id)?;
        store.delete(&data.img_record_id)?;
        let result: Result<u64, Error> = store.get_conn().query_row(
            "select id from clipboard_record where id = ?1 or id = ?2",
            [data.text_record_id, data.img_record_id],
            |r| r.get(0),
        );
        assert!(matches!(result, Err(Error::QueryReturnedNoRows)));
        Ok(())
    }
}
