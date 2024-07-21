// record crud
// a record is like:
// {id: number, type: string, content: string, hash: string, timestamp: number, pinned: boolean}

use rusqlite::Connection;

pub struct ClipboardRecord {
  id: u64,
  record_type: String,
  hash: String,
  timestamp: u64,
  pinned: bool
}

pub struct ClipboardStore {
  conn: Connection
}