use sqlx::MySql;

pub trait Acquire<'a>: sqlx::Acquire<'a, Database = MySql> {}

impl<'a, A> Acquire<'a> for A where A: sqlx::Acquire<'a, Database = MySql> {}

pub trait Executor<'a>: sqlx::Executor<'a, Database = sqlx::MySql> + Acquire<'a> {}

impl<'a, E> Executor<'a> for E where E: sqlx::Executor<'a, Database = MySql> + Acquire<'a> {}
