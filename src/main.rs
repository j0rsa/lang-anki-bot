mod lib;
mod schema;

#[macro_use]
extern crate diesel;
// #[macro_use]
extern crate diesel_migrations;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
// embed_migrations!();

use crate::lib::actions;
use crate::lib::models::service_credential::ServiceCredential;
use crate::lib::upstream::lingoda::Lingoda;
use futures::StreamExt;
use lib::downstream::anki::*;
use std::borrow::Borrow;
use std::env;
use std::error::Error;
use telegram_bot::{
    Api, CanSendMessage, Integer, Message, MessageKind, ToSourceChat, UpdateKind, UserId,
};
// use crate::lib::db_config::DbConfig;
// use crate::lib::repository::{get_token, save_token};
// use diesel_migrations::RunMigrationsError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenvy::dotenv().ok();

    // let pool = DbConfig::get_pool();
    // DbConfig::test_connection(&pool).unwrap();
    // DbConfig::migrate_db(&pool).expect("Failed to migrate");

    let anki = Anki::new(env::var("ANKI_URL").expect("ANKI_URL must be set"));
    let deck = env::var("ANKI_DECK").unwrap_or("Default".to_string());
    let cred = ServiceCredential::no_token_new(
        env::var("LINGODA_USER")
            .expect("No username specified")
            .as_ref(),
        env::var("LINGODA_PASSWORD")
            .expect("No password specified")
            .as_ref(),
    );
    let lingoda = Lingoda::new();
    let allowed_user_ids = env::var("TELEGRAM_ALLOWED_USERS")
        .expect("No allowed user specified")
        .split(",")
        .map(|x| x.parse().expect("Invalid allowed user id"))
        .collect::<Vec<Integer>>();

    let allowed_users = allowed_user_ids
        .iter()
        .map(|user| UserId::new(user.to_owned()))
        .collect::<Vec<UserId>>();

    let pack = LangPack {
        anki: Box::new(anki),
        deck: Box::new(deck),
        service_credentials: Box::new(cred),
        lingoda: Box::new(lingoda),
        allowed_users,
    };

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        match update.kind {
            UpdateKind::Message(ref message) => match message.kind {
                MessageKind::Text { ref data, .. } => {
                    process_message(api.clone(), &pack, &message, data.clone()).await;
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}

struct LangPack {
    anki: Box<Anki>,
    deck: Box<String>,
    service_credentials: Box<ServiceCredential>,
    lingoda: Box<Lingoda>,
    allowed_users: Vec<UserId>,
}

async fn process_message(api: Api, pack: &LangPack, message: &Message, text: String) {
    if !pack.allowed_users.contains(&message.from.id) {
        api.send(message.to_source_chat().text(format!(
            "Sorry, I cant talk to you. Your id({}) is not in allow list",
            message.from.id
        )))
        .await
        .expect("Unable to send message");
        return;
    }
    if text.starts_with("lingoda lesson ") {
        let id_res = get_lesson_id(text);
        match id_res {
            Ok(id) => {
                process_lingoda_lesson(api, pack, message, id)
                    .await
                    .expect("Unable to process lingoda lesson request");
            }
            _ => {
                api.send(
                    message
                        .to_source_chat()
                        .text("Sorry, I didn't get it. Try typing 'lingoda lesson <number>'"),
                )
                .await
                .expect("Unable to send message");
                return;
            }
        }
    }
}

fn get_lesson_id(string: String) -> Result<i64, ()> {
    if let Some(id_string) = string.get(15..string.len()) {
        id_string.trim().parse::<i64>().map_err(|_| ())
    } else {
        Err(())
    }
}

async fn process_lingoda_lesson(
    api: Api,
    pack: &LangPack,
    message: &Message,
    lesson_id: Integer,
) -> Result<(), Box<dyn Error>> {
    match actions::lingoda_to_anki(
        pack.lingoda.borrow(),
        pack.anki.borrow(),
        pack.deck.borrow(),
        pack.service_credentials.clone(),
        lesson_id,
    )
    .await
    {
        Ok(()) => api.send(message.to_source_chat().text("Done! 😊")),
        Err(ref err) => api.send(
            message
                .to_source_chat()
                .text(format!("Oh no, the error occurred: {}", err)),
        ),
    }
    .await
    .expect("Unable to send a lingoda lesson message!");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lib::downstream::anki_cloze_note::AnkiClozable;
    use crate::lib::models::service_credential::ServiceCredential;
    use crate::lib::upstream::lingoda::Lingoda;
    use std::error::Error;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn lingoda_to_anki() -> Result<(), Box<dyn Error>> {
        dotenvy::dotenv().ok();
        let anki = Anki::new(env::var("ANKI_URL").expect("No Anki URL specified"));
        let deck = env::var("ANKI_DECK").expect("No Deck specified");
        let cred = ServiceCredential::no_token_new(
            env::var("LINGODA_USER")
                .expect("No username specified")
                .as_ref(),
            env::var("LINGODA_PASSWORD")
                .expect("No password specified")
                .as_ref(),
        );
        let lingoda = Lingoda::new();
        let lesson_id = env::var("LINGODA_LESSON_ID")
            .expect("No lingoda lesson ID specified")
            .parse::<i64>()
            .expect("lesson ID should be a number");

        let anki_notes = lingoda
            .get_lesson_words(Box::new(cred), lesson_id)
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
        }

        anki.sync().await.expect("Failed to sync anki");

        Ok(())
    }

    #[test]
    fn lingoda_id_test() {
        assert_eq!(get_lesson_id("lingoda lesson 123".to_owned()), Ok(123));
        assert_eq!(get_lesson_id("lingoda lesson   123   ".to_owned()), Ok(123));
        assert_eq!(
            get_lesson_id("lingoda lesson:   123   ".to_owned()),
            Ok(123)
        );
        assert_eq!(
            get_lesson_id("lingoda lesson=   123   ".to_owned()),
            Ok(123)
        );
        assert_eq!(
            get_lesson_id("lingoda lesson= some  123   ".to_owned()),
            Err(())
        );
    }
}
