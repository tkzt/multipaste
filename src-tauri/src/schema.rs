// @generated automatically by Diesel CLI.

diesel::table! {
    clipboard_record (id) {
        id -> Integer,
        record_type -> Text,
        record_value -> Text,
        record_hash -> Nullable<Text>,
        updated_at -> Timestamp,
        pinned -> Bool,
    }
}
