
use diesel::pg::PgConnection;
use diesel::r2d2::{
    ConnectionManager,
    Pool
};
use std::env;
use std::process::exit;
use diesel::migration::RunMigrationsError;
use reqwest::redirect::Policy;
use crate::embedded_migrations;
use super::errors::Error;

#[derive(Clone, Debug)]
pub struct DbConfig;

impl DbConfig {
    pub fn get_pool() -> Pool<ConnectionManager<PgConnection>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().max_size(15)
            .build(manager)
            .expect("Failed to create pool.")
    }

    pub fn test_connection(pool: &Pool<ConnectionManager<PgConnection>>) -> Result<(), Error> {
        // https://dev.to/werner/practical-rust-web-development-connection-pool-46f4
        pool.clone().get()?;
        Ok(())
    }

    pub fn migrate_db(pool: &Pool<ConnectionManager<PgConnection>>) -> Result<(), RunMigrationsError>{
        embedded_migrations::run(&pool.clone().get().unwrap())
    }
}
