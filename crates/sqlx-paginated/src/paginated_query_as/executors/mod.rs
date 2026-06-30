#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

use sqlx::{Database, Executor};

pub trait AsExecutor<DB: Database> {
    type Executor<'a>: Executor<'a, Database = DB>
    where
        Self: 'a;

    fn as_executor(&mut self) -> Self::Executor<'_>;
}
