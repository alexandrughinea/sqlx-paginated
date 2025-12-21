#[cfg(feature = "postgres")]
mod postgres_dialect;

pub mod query_dialect;
#[cfg(feature = "sqlite")]
mod sqlite_dialect;

#[cfg(feature = "postgres")]
pub use postgres_dialect::*;

#[cfg(feature = "sqlite")]
pub use sqlite_dialect::*;
