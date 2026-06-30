use super::AsExecutor;
use sqlx::{PgConnection, PgPool, Postgres};

impl AsExecutor<Postgres> for PgConnection {
    type Executor<'a> = &'a mut PgConnection;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        self
    }
}

impl<'c> AsExecutor<Postgres> for &'c mut PgConnection {
    type Executor<'e>
        = &'e mut PgConnection
    where
        Self: 'e;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        *self
    }
}

impl AsExecutor<Postgres> for PgPool {
    type Executor<'a> = &'a PgPool;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        self
    }
}

impl<'p> AsExecutor<Postgres> for &'p PgPool {
    type Executor<'e>
        = &'e PgPool
    where
        Self: 'e;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        *self
    }
}
