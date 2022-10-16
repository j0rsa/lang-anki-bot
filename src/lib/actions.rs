use std::error::Error;
use std::thread;
use std::time::Duration;
use crate::{Anki, Lingoda, Note, ServiceCredential};
use crate::lib::upstream::lingoda::vocabulary_items::VocabularyItemsVocabularyItems;
use crate::lib::downstream::anki_cloze_note::AnkiClozable;

pub(crate) async fn lingoda_to_anki(
    lingoda: &Lingoda,
    anki: &Anki,
    deck: &String,
    cred: Box<ServiceCredential>,
    lesson_id: i64
) -> Result<(), Box<dyn Error>> {
    let words: Vec<VocabularyItemsVocabularyItems> = lingoda
        .get_lesson_words(cred, lesson_id)
        .await?;
    let anki_notes = words
        .iter()
        .map(|w| w.to_cloze(deck))
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

    anki.sync().await.expect("Failed to sync anki");

    Ok(())
}