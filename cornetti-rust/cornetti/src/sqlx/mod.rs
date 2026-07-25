#![doc = include_str!("../../../spec/sqlx.md")]

#[cfg(not(any(
    feature = "sqlxdb-postgres",
    feature = "sqlxdb-mysql",
    feature = "sqlxdb-sqlite"
)))]
compile_error!(
    "sqlxdb requires one backend feature: sqlxdb-postgres, sqlxdb-mysql, or sqlxdb-sqlite"
);

pub mod confs;
pub mod adapters;
pub mod pagination;
pub mod services;
