use sqlx::MySql;

pub trait Acquire<'a>: sqlx::Acquire<'a, Database = MySql> {}

impl<'a, A> Acquire<'a> for A where A: sqlx::Acquire<'a, Database = MySql> {}

pub trait Executor<'a>: sqlx::Executor<'a, Database = sqlx::MySql> + Acquire<'a> {}

impl<'a, E> Executor<'a> for E where E: sqlx::Executor<'a, Database = MySql> + Acquire<'a> {}

pub type Transaction<'a> = sqlx::Transaction<'a, MySql>;

pub trait Connection: sqlx::Connection<Database = MySql> {}

impl<C> Connection for C where C: sqlx::Connection<Database = MySql> {}
