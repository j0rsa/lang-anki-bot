use super::anki::{Attachment, DuplicateScopeOptions, Fields, Note, Options};

pub struct AnkiClozeNote {
    deck_name: String,
    text: String,
    tags: Vec<String>,
    audio: Option<Attachment>,
    // later picture
}

impl AnkiClozeNote {
    pub fn new(deck_name: String, text: String, tags: String, audio: Option<Attachment>) -> Self {
        let mut string_tags = tags.clone();
        let tags_list = string_tags.split(',').map(|s| s.to_owned()).collect::<Vec<_>>();
        AnkiClozeNote { deck_name, text, audio, tags: tags_list }
    }
}

pub fn anki_cloze_note(anki_cloze_note: AnkiClozeNote) -> Note {
    Note {
        deck_name: anki_cloze_note.deck_name.clone(),
        model_name: "Cloze".to_string(),
        fields: Fields {
            text: anki_cloze_note.text
        },
        options: Options {
            allow_duplicate: false,
            duplicate_scope: "deck".to_string(),
            duplicate_scope_options: Some(DuplicateScopeOptions {
                deck_name: "Default".to_string(),
                check_children: false,
                check_all_models: false
            })
        },
        tags: anki_cloze_note.tags,
        audio: match anki_cloze_note.audio {
            Some(attachment) => vec![attachment.clone()],
            None => vec![],
        },
        picture: vec![
            Attachment {
                url: "".to_string(),
                filename: "".to_string(),
                fields: vec![
                    "Extra".to_string(),
                ]
            }
        ]
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

trait AnkiClozable {
    fn to_cloze(&self) -> AnkiClozeNote;
}