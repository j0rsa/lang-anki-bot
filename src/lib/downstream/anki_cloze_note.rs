use super::anki::{Attachment, DuplicateScopeOptions, Fields, Note, Options};

pub struct AnkiClozeNote {
    deck_name: String,
    text: String,
    tags: Vec<String>,
    audio: Option<Attachment>,
    // later picture
}

impl AnkiClozeNote {
    pub fn new(deck_name: String, text: String, tags: Option<String>, audio: Option<Attachment>) -> Self {
        let tags_list = match tags {
            None => {
                vec![]
            }
            Some(tags) => {
                tags.split(',').map(|s| s.to_owned()).collect::<Vec<_>>()
            }
        };
        AnkiClozeNote { deck_name, text, audio, tags: tags_list }
    }

    pub fn to_anki_note(self) -> Note {
        Note {
            deck_name: self.deck_name.clone(),
            model_name: "Cloze".to_string(),
            fields: Fields {
                text: self.text
            },
            options: Options {
                allow_duplicate: false,
                duplicate_scope: "deck".to_string(),
                duplicate_scope_options: Some(DuplicateScopeOptions {
                    deck_name: self.deck_name.clone(),
                    check_children: false,
                    check_all_models: false
                })
            },
            tags: self.tags,
            audio: match self.audio {
                Some(attachment) => vec![attachment.clone()],
                None => vec![],
            },
            picture: vec![]
        }
    }
}

pub trait ToAttachment {
    fn to_attachment(&self, fields: Vec<String>) -> Attachment;
}

impl ToAttachment for String {
    fn to_attachment(&self, fields: Vec<String>) -> Attachment  {
        let filename = if self.contains("?") {
            format!("{}.mp3",self.split("=").last().unwrap())
        } else {
            self.split("/").last().unwrap().to_string()
        };

        Attachment {
            url: self.clone(),
            filename,
            fields
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_url_picture_parse() {
        let url = "https://cdn-user77752.skyeng.ru/resized-images/200x150/png/50/5a677c4b4a356e7a4a3fc243deb73676.png".to_string();
        let attachment = url.to_attachment(vec![
            "Extra".to_string()
        ]);
        assert_eq!(attachment.url, url);
        assert_eq!(attachment.filename, "5a677c4b4a356e7a4a3fc243deb73676.png".to_string());
        assert_eq!(attachment.fields, vec![
            "Extra".to_string()
        ]);
    }
}

pub trait AnkiClozable {
    fn to_cloze(&self, deck_name: &String) -> AnkiClozeNote;
}