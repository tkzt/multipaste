use crate::{conf::Config, schema};
use chrono::{Local, NaiveDateTime};
use crypto::{digest::Digest, sha2::Sha256};
use diesel::{
    connection::SimpleConnection,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    prelude::{Insertable, Queryable, QueryableByName},
    serialize::{IsNull, ToSql},
    sql_types::{SqlType, Text},
    sqlite::Sqlite,
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
    SqliteConnection, TextExpressionMethods,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use glob::glob;
use log::warn;
use rusqlite::{
    types::{FromSqlError, Type as RSType},
    Error as RusqliteError,
};
use serde::{Serialize, Serializer};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{App, Manager, State};

type Result<T, E = diesel::result::Error> = std::result::Result<T, E>;

const DATABASE_URL: &str = "data.db";
const IMG_DIR_PATH: &str = "images";
const MIN_TEXT_HASHING_SIZE: usize = 50;
const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(SqlType, Debug, FromSqlRow, Copy, Clone, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
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

impl ToSql<Text, Sqlite> for RecordType {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        out.set_value(self.to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for RecordType {
    fn from_sql(
        bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        Ok(<String as FromSql<Text, Sqlite>>::from_sql(bytes)
            .map(|s| RecordType::from_string(&s))
            .unwrap()?)
    }
}

#[derive(Debug)]
pub struct RecordStore {
    pool: r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>,
    pub img_dir: PathBuf,
}

#[derive(Queryable, Selectable, QueryableByName, Serialize, Debug)]
#[diesel(table_name = schema::clipboard_record)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ClipboardRecord {
    pub id: i32,
    pub record_type: RecordType,
    // For image, the record value will be like
    // <IMG_DIR>/<sha256_hash>.png
    pub record_value: String,
    pub record_hash: Option<String>,
    pub updated_at: NaiveDateTime,
    pub pinned: bool,
}

#[derive(Insertable)]
#[diesel(table_name = schema::clipboard_record)]
pub struct NewClipboardRecord<'a> {
    pub record_type: &'a RecordType,
    pub record_hash: Option<&'a str>,
    pub record_value: &'a str,
}

impl RecordStore {
    pub fn new(db_path: PathBuf, img_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&img_dir).expect("Failed to create image directory");
        if !db_path.exists() {
            if let Some(path) = db_path.parent() {
                std::fs::create_dir_all(path).expect("Failed to create db directory");
            }
            File::create(&db_path).expect("Failed to create db file");
        }
        let manager =
            diesel::r2d2::ConnectionManager::<SqliteConnection>::new(db_path.to_str().unwrap());
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to initialize pool");
        RecordStore { pool, img_dir }
    }

    pub fn get_conn(
        &self,
    ) -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>> {
        self.pool.get().expect("Failed to get connection")
    }

    pub fn update_max_records_trigger(&self, max_records: u64) -> Result<()> {
        let conn = &mut self.get_conn();
        let stmt = &format!(
            "
            DROP TRIGGER IF EXISTS limit_records_amount;
            CREATE TRIGGER limit_records_amount
            AFTER INSERT ON clipboard_record
            BEGIN
                DELETE FROM clipboard_record
                WHERE id IN (
                    SELECT id FROM clipboard_record
                    ORDER BY pinned DESC, updated_at DESC LIMIT -1 OFFSET {0}
                ) AND (SELECT COUNT(*) FROM clipboard_record) > {0};
            END;",
            max_records
        );
        conn.batch_execute(stmt)
            .expect("Failed to update max records trigger");
        Ok(())
    }

    pub fn init(&self, max_records: u64) -> Result<()> {
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
        record_hash: Option<&str>,
    ) -> Result<bool> {
        let conn = &mut self.get_conn();
        let updated_rows = diesel::update(
            schema::clipboard_record::table.filter(
                schema::clipboard_record::dsl::record_hash
                    .is_not_null()
                    .and(schema::clipboard_record::dsl::record_hash.eq(record_hash))
                    .or(schema::clipboard_record::dsl::record_value.eq(record_value)),
            ),
        )
        .set(schema::clipboard_record::updated_at.eq(Local::now().naive_local()))
        .execute(conn)?;
        log::info!("Updated rows: {}", updated_rows);

        if updated_rows == 0 {
            let inserted = diesel::insert_into(schema::clipboard_record::table)
                .values(&NewClipboardRecord {
                    record_type,
                    record_value,
                    record_hash,
                })
                .returning(ClipboardRecord::as_returning())
                .get_result::<ClipboardRecord>(conn)?;
            log::info!("Inserted record: {:?}", inserted);
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
        let conn = &mut self.get_conn();
        let hash_values: Vec<String> = hashes.iter().map(|h| format!("('{}')", h)).collect();

        let batch_stmt = format!(
            "
            DROP TABLE IF EXISTS temp_hashes;
            CREATE TABLE temp_hashes (record_hash VARCHAR(32));
            INSERT INTO temp_hashes (record_hash) VALUES {};
            ",
            hash_values.join(",")
        );
        conn.batch_execute(&batch_stmt)?;

        #[derive(QueryableByName)]
        struct HashQueryResult {
            #[diesel(sql_type = Text)]
            record_hash: String,
        }

        let image_hashes: Vec<String> = diesel::sql_query(
            "
            SELECT temp_hashes.record_hash
            FROM temp_hashes
            LEFT JOIN clipboard_record ON temp_hashes.record_hash = clipboard_record.record_hash
            WHERE clipboard_record.record_hash IS NULL;
            ",
        )
        .load::<HashQueryResult>(conn)?
        .into_iter()
        .map(|hash_result| hash_result.record_hash)
        .collect();
        Ok(image_hashes)
    }

    fn clean_dangling_images(&self) -> Result<()> {
        if let Some(img_dir) = self.img_dir.to_str() {
            if let Ok(paths) = glob(&format!("{}/*.png", img_dir)) {
                let hashes: Vec<String> = paths
                    .flatten()
                    .filter_map(|image_path| {
                        image_path
                            .file_stem()
                            .and_then(|p| p.to_str())
                            .map(|s| s.to_owned())
                    })
                    .collect();
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
                warn!("Failed to glob images");
            }
        } else {
            warn!("Failed to get image directory");
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
            if let Err(write_err) = std::fs::write(&image_path, image_bytes) {
                warn!("Failed to save image: {:?}", write_err);
            }
        }

        self.clean_dangling_images()?;

        Ok(())
    }

    pub fn pin(&self, id: &i32) -> Result<ClipboardRecord> {
        let conn = &mut self.get_conn();
        let updated = diesel::update(schema::clipboard_record::table.find(id))
            .set(schema::clipboard_record::pinned.eq(true))
            .returning(ClipboardRecord::as_returning())
            .get_result(conn)?;
        Ok(updated)
    }

    pub fn unpin(&self, id: &i32) -> Result<ClipboardRecord> {
        let conn = &mut self.get_conn();
        let updated = diesel::update(schema::clipboard_record::table.find(id))
            .set(schema::clipboard_record::pinned.eq(false))
            .returning(ClipboardRecord::as_returning())
            .get_result(conn)?;
        Ok(updated)
    }

    pub fn delete(&self, id: &i32) -> Result<usize> {
        let conn = &mut self.get_conn();
        let deleted = diesel::delete(schema::clipboard_record::table.find(id)).execute(conn)?;
        Ok(deleted)
    }

    pub fn get_records(&self, keyword: &str) -> Vec<ClipboardRecord> {
        let conn = &mut self.get_conn();
        let records = schema::clipboard_record::table
            .filter(schema::clipboard_record::dsl::record_value.like(format!("%{}%", keyword)))
            .order((
                schema::clipboard_record::dsl::pinned.desc(),
                schema::clipboard_record::dsl::updated_at.desc(),
            ))
            .load::<ClipboardRecord>(conn)
            .unwrap_or(vec![]);
        records
    }

    pub fn get_record(&self, id: &i32) -> Result<ClipboardRecord> {
        let conn = &mut self.get_conn();
        let record = schema::clipboard_record::table
            .find(id)
            .first::<ClipboardRecord>(conn)?;
        Ok(record)
    }
}

pub fn init(app: &App) -> Result<Arc<RecordStore>, Box<dyn std::error::Error>> {
    let config = app.state::<Mutex<Config>>();
    let app_data_path = app.path().app_data_dir().unwrap();

    let db_url = app_data_path.join(DATABASE_URL);
    let img_dir = app_data_path.join(IMG_DIR_PATH);

    log::debug!("DB path: {:?}", db_url);
    log::debug!("Image dir: {:?}", img_dir);

    let store = Arc::new(RecordStore::new(db_url.clone(), img_dir));

    let conn = &mut store.get_conn();
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    store.init(config.lock().unwrap().max_items)?;
    app.manage(store.clone());
    return Ok(store);
}

#[tauri::command]
pub fn pin_record(store: State<Arc<RecordStore>>, id: i32) {
    store.pin(&id).unwrap();
}

#[tauri::command]
pub fn unpin_record(store: State<Arc<RecordStore>>, id: i32) {
    store.unpin(&id).unwrap();
}

#[tauri::command]
pub fn delete_record(store: State<Arc<RecordStore>>, id: i32) {
    let record = store.get_record(&id).unwrap();
    if record.record_type == RecordType::Image {
        std::fs::remove_file(store.img_dir.join(record.record_value)).unwrap();
    }
    store.delete(&id).unwrap();
}

#[tauri::command]
pub fn filter_records(store: State<Arc<RecordStore>>, keyword: String) -> Vec<ClipboardRecord> {
    store.get_records(&keyword)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
    use log::LevelFilter;
    use std::{io::Cursor, thread::sleep, time::Duration};

    const TEXT_VALUE: &str = "foo";
    const SOME_TEXT_NOT_IN_DB: &str = "Dolor culpa ut nisi qui veniam proident Lorem proident enim ea. Consequat adipisicing officia consectetur do sit deserunt. Veniam nostrud laboris ipsum sunt deserunt ex nulla minim nostrud voluptate consequat excepteur. Consequat tempor sint adipisicing minim anim. Ad eu nisi id in culpa qui ut eiusmod minim veniam ea. Esse non voluptate eiusmod officia duis consectetur dolore eu nulla ullamco labore id nulla.";

    struct SharedData {
        text_record_id: i32,
        img_record_id: i32,
    }

    lazy_static::lazy_static! {
        static ref SHARED_STORE: Mutex<RecordStore> = {
            env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
            Mutex::new(RecordStore::new(
            Path::new(DATABASE_URL).to_path_buf(),
            Path::new(IMG_DIR_PATH).to_path_buf())
        )};
        static ref SHARED_DATA: Mutex<SharedData> = Mutex::new(SharedData {text_record_id: 0, img_record_id: 0 });
    }

    #[test]
    fn test_01_save() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let mut data = SHARED_DATA.lock().unwrap();
        store.init(2)?;
        let conn = &mut store.get_conn();

        // text
        let result = store.save_text(TEXT_VALUE);
        assert!(result.is_ok());

        let result = schema::clipboard_record::table
            .filter(
                schema::clipboard_record::dsl::record_value
                    .eq(TEXT_VALUE)
                    .and(
                        schema::clipboard_record::dsl::record_type.eq(RecordType::Text.to_string()),
                    ),
            )
            .first::<ClipboardRecord>(conn);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.id > 0);
        assert!(result.record_type == RecordType::Text);
        assert!(result.updated_at < Local::now().naive_local());

        data.text_record_id = result.id;

        log::info!("Checking image saving");
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
            .write_to(&mut Cursor::new(&mut img_bytes), ImageFormat::Png)
            .unwrap();

        let result = store.save_image(&img_bytes);
        assert!(result.is_ok());

        let img_hash = store.calc_hash(&img_bytes);
        let image_path = format!("{}/{}.png", store.img_dir.to_str().unwrap(), &img_hash);
        let mut query_image_res = move || -> (i32, NaiveDateTime) {
            let img_res = schema::clipboard_record::table
                .filter(
                    schema::clipboard_record::dsl::record_value
                        .eq(&image_path)
                        .and(
                            schema::clipboard_record::dsl::record_type
                                .eq(RecordType::Image.to_string()),
                        ),
                )
                .first::<ClipboardRecord>(conn);
            let img_record = img_res.unwrap();
            (img_record.id, img_record.updated_at)
        };

        let (img_record_id, updated_at) = query_image_res();
        assert!(img_record_id > 0);
        assert!(updated_at < Local::now().naive_local());

        // Make sure the time interval between the first saving and the second
        // is greater than 1 second.
        sleep(Duration::from_secs(1));

        log::info!("Checking repeat saving");
        // Check repeat saving
        store.save_image(&img_bytes)?;
        let (img_record_id_repeat, updated_at_repeat) = query_image_res();
        assert_eq!(img_record_id, img_record_id_repeat);
        assert_ne!(updated_at, updated_at_repeat);

        data.img_record_id = img_record_id;
        Ok(())
    }

    #[test]
    fn test_02_get_record() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();

        let record = store.get_record(&data.text_record_id)?;
        assert_eq!(record.id, data.text_record_id);

        let records = store.get_records(TEXT_VALUE);
        assert!(records
            .iter()
            .any(|record| record.record_value == TEXT_VALUE));

        let records = store.get_records(SOME_TEXT_NOT_IN_DB);
        assert_eq!(records.len(), 0);
        Ok(())
    }

    #[test]
    fn test_03_toggle_pinned() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.pin(&data.text_record_id)?;
        let records = store.get_records("");
        let [record, ..] = records.as_slice() else {
            panic!("Empty records")
        };
        assert_eq!(record.id, data.text_record_id);
        assert_eq!(record.pinned, true);

        store.unpin(&data.text_record_id)?;

        let conn = &mut store.get_conn();
        let the_pinned_result = schema::clipboard_record::table
            .find(record.id)
            .first::<ClipboardRecord>(conn)?;

        assert_eq!(the_pinned_result.pinned, false);
        Ok(())
    }

    #[test]
    fn test_04_max_records() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let mut data = SHARED_DATA.lock().unwrap();

        let text_value = &format!("{}{}", TEXT_VALUE, TEXT_VALUE);
        store.save_text(text_value)?;

        let result = store.get_record(&data.text_record_id);
        assert!(result.is_err());

        let conn = &mut store.get_conn();
        let newly_saved_record_result = schema::clipboard_record::table
            .filter(schema::clipboard_record::dsl::record_value.eq(text_value))
            .first::<ClipboardRecord>(conn)?;
        assert!(newly_saved_record_result.id > 0);

        data.text_record_id = newly_saved_record_result.id;
        Ok(())
    }

    #[test]
    fn test_05_delete_record() -> Result<()> {
        let store = SHARED_STORE.lock().unwrap();
        let data = SHARED_DATA.lock().unwrap();
        store.delete(&data.text_record_id)?;
        store.delete(&data.img_record_id)?;

        let conn = &mut store.get_conn();
        let result = schema::clipboard_record::table
            .filter(
                schema::clipboard_record::dsl::id
                    .eq(data.text_record_id)
                    .or(schema::clipboard_record::dsl::id.eq(data.img_record_id)),
            )
            .first::<ClipboardRecord>(conn);

        assert!(result.is_err());
        Ok(())
    }
}
