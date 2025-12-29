#[cfg(feature = "postgres")]
pub static COLUMN_PROTECTION_BLOCKED_POSTGRES: [&str; 13] = [
    // System schemas and tables
    "pg_",
    "information_schema.",
    // System columns
    "oid",
    "tableoid",
    "xmin",
    "xmax",
    "cmin",
    "cmax",
    "ctid",
    // Other sensitive prefixes
    "pg_catalog",
    "pg_toast",
    "pg_temp",
    "pg_internal",
];

#[cfg(feature = "sqlite")]
pub static COLUMN_PROTECTION_BLOCKED_SQLITE: [&str; 7] = [
    // System tables
    "sqlite_master",
    "sqlite_schema",
    "sqlite_temp_master",
    "sqlite_sequence",
    // Internal prefixes
    "sqlite_",
    // Internal row identifiers
    "rowid",
    "_rowid_",
];
