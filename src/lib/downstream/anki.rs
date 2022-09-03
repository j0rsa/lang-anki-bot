use std::fmt::Debug;
use futures::future::join_all;
use reqwest::header::VacantEntry;
use super::super::errors::{Error, Result};
use serde::{Serialize, Deserialize};

trait AnkiAction{}

#[derive(Serialize, Debug)]
struct Action {
    pub version: i32,
    pub action: String,
    pub params: Option<NoteWrapper>,
}

#[derive(Serialize, Debug)]
struct ShortAction {
    pub version: i32,
    pub action: String,
}

impl AnkiAction for Action {}
impl AnkiAction for ShortAction {}

#[derive(Serialize, Debug)]
pub struct NoteWrapper {
    pub note: Note,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub deck_name: String,
    pub model_name: String,
    pub fields: Fields,
    pub options: Options,
    pub tags: Vec<String>,
    pub audio: Vec<Attachment>,
    pub picture: Vec<Attachment>,
}

#[derive(Serialize, Debug)]
pub struct Fields {
    #[serde(rename = "Text")]
    pub text: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub allow_duplicate: bool,
    //false
    pub duplicate_scope: String,
    //deck
    pub duplicate_scope_options: Option<DuplicateScopeOptions>,

}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateScopeOptions {
    pub deck_name: String,
    //"Default"
    pub check_children: bool,
    //false
    pub check_all_models: bool, //false
}

#[derive(Serialize, Debug, Clone)]
pub struct Attachment {
    pub url: String,
    pub filename: String,
    pub fields: Vec<String>, //[Extra]
}

pub struct Anki {
    client: reqwest::Client,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct AnkiResult<T> {
    pub result: T,
    pub error: Option<String>,
}

impl Anki {
    pub fn new(url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            url,
        }
    }

    async fn run_action<T: AnkiAction + Sized + Serialize + Debug>(&self, action: T) -> Result<reqwest::Response> {
        let path = self.url.clone();
        println!("Running the action: {:#?}", serde_json::to_string(&action));
        self.client.post(&path)
            .json(&action)
            .send()
            .await
            .map_err(|e| Error::Reqwest { e, path })
    }

    pub async fn get_decks(&self) -> Result<Vec<String>> {
        let data = ShortAction {
            version: 6,
            action: "deckNames".to_string(),
        };
        let run = self.run_action(data).await
            .map(|res| async {
                let body = res.text().await.unwrap();
                println!("Response body: {}", body);
                let response: AnkiResult<Vec<String>> = serde_json::from_str(&body)
                    .expect(&*format!("failed to get the correct response. Got {}", body));
                response.result
            });
        match run {
            Ok(res) => Ok(res.await),
            Err(e) => Err(e)
        }
    }

    pub async fn sync(&self) -> Result<()> {
        let data = ShortAction {
            version: 6,
            action: "sync".to_string(),
        };
        match self.run_action(data).await {
            Ok(r) => {
                println!("Response: {:#?}", r);
                Ok(())
            },
            Err(e) => Err(e)
        }

    }

    pub async fn add_note(&self, note: Note) -> Result<()> {
        let data = Action {
            version: 6,
            action: "addNote".to_string(),
            params: Some(NoteWrapper { note }),
        };
        self.run_action(data).await.map(|_|())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_anki_desks() {
        let anki = Anki::new("http://10.43.149.198".to_string());
        let desks = anki.get_decks().await.unwrap();
        assert!(desks.len() > 0);
        println!("{:#?}", desks);
    }

    #[tokio::test]
    async fn test_anki_sync() {
        let anki = Anki::new("http://10.43.149.198".to_string());
        anki.sync().await.unwrap();
    }
}
