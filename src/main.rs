mod lib;
mod schema;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
embed_migrations!();

use std::env;
use std::process::exit;
use chrono::Utc;
use lib::downstream::{
    anki::*,
};
use crate::lib::db_config::DbConfig;
use crate::lib::repository::{get_token, save_token};
use diesel_migrations::RunMigrationsError;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let pool = DbConfig::get_pool();
    DbConfig::test_connection(&pool).unwrap();
    DbConfig::migrate_db(&pool).expect("Failed to migrate");




    let anki = Anki::new(env::var("ANKI_URL").expect("ANKI_URL must be set"));
    let deck = env::var("ANKI_DECK").unwrap_or("Default".to_string());



    // save meanings into anki
    // for note in meanings.iter().map(|m| {
    //     m.to_notes(&deck)
    // }) {
    //     anki.add_note(note)
    //         .await
    //         .expect("Failed to add note to anki");
    // }
    anki.sync()
        .await
        .expect("Failed to sync anki");

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_url_picture_parse() {

    }
}
