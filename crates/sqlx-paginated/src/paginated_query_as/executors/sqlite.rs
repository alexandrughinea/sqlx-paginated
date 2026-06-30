use super::AsExecutor;
use sqlx::{Sqlite, SqliteConnection, SqlitePool};

impl AsExecutor<Sqlite> for SqliteConnection {
    type Executor<'a> = &'a mut SqliteConnection;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        self
    }
}

impl<'c> AsExecutor<Sqlite> for &'c mut SqliteConnection {
    type Executor<'e>
        = &'e mut SqliteConnection
    where
        Self: 'e;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        *self
    }
}

impl AsExecutor<Sqlite> for SqlitePool {
    type Executor<'a> = &'a SqlitePool;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        self
    }
}

impl<'p> AsExecutor<Sqlite> for &'p SqlitePool {
    type Executor<'e>
        = &'e SqlitePool
    where
        Self: 'e;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        *self
    }
}
