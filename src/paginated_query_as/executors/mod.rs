use sqlx::{pool::PoolConnection, Database, Executor, Pool};

/// A wrapper trait that provides a SQLx executor from a pool, pool connection or regular connection.
pub trait AsExecutor<DB: Database> {
    type Executor<'a>: Executor<'a, Database = DB>
    where
        Self: 'a;

    fn as_executor(&mut self) -> Self::Executor<'_>;
}

impl<DB, E> AsExecutor<DB> for &mut E
where
    DB: Database,
    for<'a> &'a mut E: Executor<'a, Database = DB>,
{
    type Executor<'a>
        = &'a mut E
    where
        Self: 'a;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        *self
    }
}

impl<DB> AsExecutor<DB> for Pool<DB>
where
    DB: Database,
    for<'a> &'a Pool<DB>: Executor<'a, Database = DB>,
{
    type Executor<'a>
        = &'a Pool<DB>
    where
        Self: 'a;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        self
    }
}

impl<DB> AsExecutor<DB> for &Pool<DB>
where
    DB: Database,
    for<'a> &'a Pool<DB>: Executor<'a, Database = DB>,
{
    type Executor<'a>
        = &'a Pool<DB>
    where
        Self: 'a;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        *self
    }
}

impl<DB> AsExecutor<DB> for PoolConnection<DB>
where
    DB: Database,
    for<'a> &'a mut DB::Connection: Executor<'a, Database = DB>,
{
    type Executor<'a>
        = &'a mut DB::Connection
    where
        Self: 'a;

    fn as_executor(&mut self) -> Self::Executor<'_> {
        &mut **self
    }
}
