mod lib;
mod schema;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
// embed_migrations!();

use std::env;
use std::process::exit;
use chrono::Utc;
use lib::downstream::{
    anki::*,
};
use crate::lib::db_config::DbConfig;
// use crate::lib::repository::{get_token, save_token};
// use diesel_migrations::RunMigrationsError;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let pool = DbConfig::get_pool();
    DbConfig::test_connection(&pool).unwrap();
    // DbConfig::migrate_db(&pool).expect("Failed to migrate");




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
    use crate::lib::models::service_credential::ServiceCredential;
    use crate::lib::upstream::lingoda::Lingoda;
    use std::error::Error;
    use std::thread;
    use std::time::Duration;
    use crate::lib::downstream::anki_cloze_note::{ AnkiClozeNote, AnkiClozable};
    use super::*;

    #[tokio::test]
    async fn lingoda_to_anki() -> Result<(), Box<dyn Error>> {
        dotenv::dotenv().ok();
        let anki = Anki::new(env::var("ANKI_URL").expect("No Anki URL specified"));
        let deck = env::var("ANKI_DECK").expect("No Deck specified");
        let cred = ServiceCredential::no_token_new(
            env::var("LINGODA_USER").expect("No username specified").as_ref(),
            env::var("LINGODA_PASSWORD").expect("No password specified").as_ref(),
        );
        let lingoda = Lingoda::new();
        let lesson_id = env::var("LINGODA_LESSON_ID")
            .expect("No lingoda lesson ID specified")
            .parse::<i64>()
            .expect("lesson ID should be a number");

        let anki_notes =
            lingoda.get_lesson_words(Box::new(cred), lesson_id)
                .await?
                .iter()
                .map(|w| w.to_cloze(&deck))
                .map(|acn| acn.to_anki_note())
                .collect::<Vec<Note>>();

        println!("Fetched {} notes", &anki_notes.len());

        for note in anki_notes {
            anki.add_note(note)
                .await
                .expect("Failed to add note to anki");
            // println!("Added note: {:?}", response);
            thread::sleep(Duration::from_millis(400));
        }

        anki.sync()
            .await
            .expect("Failed to sync anki");

        Ok(())
    }
}
