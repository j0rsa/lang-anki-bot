use crate::lib::downstream::anki_cloze_note::{AnkiClozable, AnkiClozeNote};
use crate::lib::models::service_credential::ServiceCredential;
use crate::lib::upstream::lingoda::vocabulary_items::VocabularyItemsVocabularyItems;
use futures::TryFutureExt;
use graphql_client::reqwest::post_graphql;
use graphql_client::GraphQLQuery;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::Url;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

pub struct Lingoda {
    jar: Arc<Jar>,
    client: reqwest::Client,
}

impl Lingoda {
    pub fn new() -> Self {
        let jar = Arc::new(Jar::default());
        Self {
            jar: Arc::clone(&jar),
            client: reqwest::ClientBuilder::new()
                .cookie_store(true)
                .cookie_provider(Arc::clone(&jar))
                .build()
                .unwrap(),
        }
    }

    async fn collect_cookies(&self) -> Result<(), reqwest::Error> {
        let res = self.client.get("https://learn.lingoda.com/").send().await;
        res.map(|_| ())
    }

    async fn login(
        &self,
        service_credential: ServiceCredential,
    ) -> Result<ServiceCredential, reqwest::Error> {
        self.collect_cookies()
            .and_then(|_| {
                let login =
                    LoginRequest::new(&service_credential.username, &service_credential.password);
                self.client
                    .post("https://learn.lingoda.com/login_check")
                    .json(&login)
                    .send()
            })
            .await
            .and_then(|_| {
                let cookie_strings = self
                    .jar
                    .cookies(&Url::parse("https://learn.lingoda.com").unwrap())
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split("; ")
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                let cookies: HashMap<String, String> = cookie_strings
                    .iter()
                    .map(|s| {
                        let parts: Vec<String> = s.split("=").map(|s| s.to_string()).collect();
                        (parts[0].to_string(), parts[1].to_string())
                    })
                    .collect();
                let token = cookies.get("BEARER").unwrap().to_string();
                let exp = cookies.get("BEARER_EXP").unwrap().to_string();
                Ok(ServiceCredential::new(
                    service_credential.username,
                    service_credential.password,
                    token,
                    exp.parse::<u128>().unwrap() * 1000,
                ))
            })
    }

    pub async fn get_lesson_words(
        &self,
        mut cred: Box<ServiceCredential>,
        id: i64,
    ) -> Result<Vec<VocabularyItemsVocabularyItems>, Box<dyn Error>> {
        if cred.is_expired() {
            let old_cred = cred.clone().as_ref().clone();
            cred = Box::new(self.login(old_cred).await?);
        }
        let variables = vocabulary_items::Variables {
            learning_unit_id: None,
            lesson_ids: vec![id],
        };

        let token = cred.token.expect("Token should not be empty").value;

        let client = reqwest::Client::builder()
            .user_agent("graphql-rust/0.11.0")
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
                ))
                .collect(),
            )
            .build()?;

        let response_body = post_graphql::<vocabularyItems, _>(
            &client,
            "https://learn.lingoda.com/graphql",
            variables,
        )
        .await
        .unwrap();
        // println!("{:?}", response_body);
        Ok(response_body
            .data
            .expect("missing response data")
            .vocabulary_items)
    }
}

#[allow(non_camel_case_types)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "resources/lingoda_schema.graphql",
    query_path = "resources/vocabulary_items_query.graphql",
    response_derives = "Debug"
)]
pub struct vocabularyItems;

#[derive(Debug, Serialize)]
pub(crate) struct LoginRequest {
    #[serde(rename = "_username")]
    username: String,
    #[serde(rename = "_password")]
    password: String,
    #[serde(rename = "_remember_me")]
    remember_me: bool, //false
}

impl LoginRequest {
    pub(crate) fn new(username: &String, password: &String) -> Self {
        LoginRequest {
            username: username.clone(),
            password: password.clone(),
            remember_me: false,
        }
    }
}

impl AnkiClozable for VocabularyItemsVocabularyItems {
    fn to_cloze(&self, deck_name: &String) -> AnkiClozeNote {
        let text = masked_text_v2(self).unwrap_or(masked_text_v1(self));
        AnkiClozeNote::new(deck_name.clone(), text, None, None)
    }
}

fn masked_text_v1(item: &VocabularyItemsVocabularyItems) -> String {
    let translation = match item.item {
        Some(ref item) => item.clone(),
        None => "".to_string(),
    };
    format!(
        "{}<br/><br/>- {}<br/><br/>{{{{c1::{}}}}}<br/>{{{{c1::{}}}}}",
        item.test_question, translation, item.title, item.sample_sentence_one
    )
}

fn masked_text_v2(item: &VocabularyItemsVocabularyItems) -> Option<String> {
    let gap_words: Vec<String> = item.test_question.split(" ").map(|s| s.to_string()).collect();
    let words: Vec<String> = item.sample_sentence_one.split(" ").map(|s| s.to_string()).collect();
    if gap_words.len() != words.len() {
        return None;
    }
    let translation = match item.item {
        Some(ref item) => item.clone(),
        None => "".to_string(),
    };
    let result: Vec<String> = gap_words.iter()
        .zip(words.iter())
        .map( |(a,b)| {
            if a==b {
                a.to_string()
            } else {
                let last_char = b.chars().last().unwrap();

                if last_char.is_alphabetic() {
                    format!("{{{{c1::{}}}}}", b)
                }else {
                    let mut everything_but_last = b.to_string();
                    everything_but_last.pop();
                    format!("{{{{c1::{}}}}}{}", everything_but_last, last_char)
                }
            }
        })
        .collect();
    Some(format!("{}<br/><br/>- {}", result.join(" "), translation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use crate::lib::upstream::lingoda::vocabulary_items::{
        VocabularyItemsVocabularyItemsCefrLevel,
        VocabularyItemWordClassType,
    };

    #[tokio::test]
    async fn test_cookies_collection() {
        let lingoda = Lingoda::new();
        assert_eq!(lingoda.collect_cookies().await.unwrap(), ());
    }

    #[tokio::test]
    async fn test_login() {
        dotenvy::dotenv().ok();
        let lingoda = Lingoda::new();
        let cred = ServiceCredential::no_token_new(
            env::var("LINGODA_USER")
                .expect("No username specified")
                .as_ref(),
            env::var("LINGODA_PASSWORD")
                .expect("No password specified")
                .as_ref(),
        );
        let token = lingoda.login(cred).await.unwrap().token;
        assert!(token.is_some());
        println!("{:#?}", token);
        let exp = token.unwrap().expires_ms;
        assert_eq!(exp / 1000 * 1000, exp)
    }

    /// Example
    /// [
    ///     VocabularyItemsVocabularyItems {
    ///             title: "verfügbare Urlaubstage",
    ///             word_class: phrase,
    ///             gender: None,
    ///             sample_sentence_one: "Du hast 21 verfügbare Urlaubstage. So viele sind auch in deinem Arbeitsvertrag vorgesehen.",
    ///             item: Some(
    ///                 "available holidays",
    ///             ),
    ///             test_question: "Du hast 21 _______________ _______________. So viele sind auch in deinem Arbeitsvertrag vorgesehen.",
    ///             cefr_level: VocabularyItemsVocabularyItemsCefrLevel {
    ///                 name: "B1",
    ///             },
    ///         },
    /// ]
    #[tokio::test]
    async fn test_get_lesson_words() {
        dotenvy::dotenv().ok();
        let lingoda = Lingoda::new();
        let cred = ServiceCredential::no_token_new(
            env::var("LINGODA_USER")
                .expect("No username specified")
                .as_ref(),
            env::var("LINGODA_PASSWORD")
                .expect("No password specified")
                .as_ref(),
        );
        let res = lingoda.get_lesson_words(Box::new(cred), 4492).await;
        println!("{:#?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.len() > 0);
    }

    #[test]
    fn masked_text_v1_test() {
        let item = VocabularyItemsVocabularyItems{
            cefr_level: VocabularyItemsVocabularyItemsCefrLevel {
                name: "B1".to_string(),
            },
            gender: None,
            item: Some("available holidays".to_string()),
            sample_sentence_one: "Du hast 21 verfügbare Urlaubstage. So viele sind auch in deinem Arbeitsvertrag vorgesehen.".to_string(),
            test_question: "Du hast 21 _______________ _______________. So viele sind auch in deinem Arbeitsvertrag vorgesehen.".to_string(),
            title: "verfügbare Urlaubstage".to_string(),
            word_class: VocabularyItemWordClassType::phrase,
        };
        let expected_text = "\
        Du hast 21 _______________ _______________. So viele sind auch in deinem Arbeitsvertrag vorgesehen.<br/>\
        <br/>\
        - available holidays<br/>\
        <br/>\
        {{c1::verfügbare Urlaubstage}}<br/>\
        {{c1::Du hast 21 verfügbare Urlaubstage. So viele sind auch in deinem Arbeitsvertrag vorgesehen.}}\
        ";

        assert_eq!(masked_text_v1(&item), expected_text)
    }

    #[test]
    fn masked_text_v2_test_1() {
        let item = VocabularyItemsVocabularyItems{
            cefr_level: VocabularyItemsVocabularyItemsCefrLevel {
                name: "B1".to_string(),
            },
            gender: None,
            item: Some("available holidays".to_string()),
            sample_sentence_one: "Du hast 21 verfügbare Urlaubstage. So viele sind auch in deinem Arbeitsvertrag vorgesehen.".to_string(),
            test_question: "Du hast 21 _______________ _______________. So viele sind auch in deinem Arbeitsvertrag vorgesehen.".to_string(),
            title: "verfügbare Urlaubstage".to_string(),
            word_class: VocabularyItemWordClassType::phrase,
        };
        let expected_text = "\
        Du hast 21 {{c1::verfügbare}} {{c1::Urlaubstage}}. So viele sind auch in deinem Arbeitsvertrag vorgesehen.<br/>\
        <br/>\
        - available holidays\
        ";

        assert_eq!(masked_text_v2(&item).unwrap(), expected_text)
    }

    #[test]
    fn masked_text_v2_test_2() {
        let item = VocabularyItemsVocabularyItems{
            cefr_level: VocabularyItemsVocabularyItemsCefrLevel {
                name: "B1".to_string(),
            },
            gender: None,
            item: Some("How boring!".to_string()),
            sample_sentence_one: "Wie langweilig der Film sein kann!".to_string(),
            test_question: "________________ der Film sein kann!".to_string(),
            title: "Wie langweilig!".to_string(),
            word_class: VocabularyItemWordClassType::phrase,
        };
        let expected_text = "\
        {{c1::Wie langweilig}} der Film sein kann!<br/>\
        <br/>\
        - How boring!\
        ";

        let text_v2 = masked_text_v2(&item);
        assert!(text_v2.is_some());
        assert_eq!(text_v2.unwrap(), expected_text)
    }

    #[test]
    fn masked_text_v2_test_3() {
        let item = VocabularyItemsVocabularyItems{
            cefr_level: VocabularyItemsVocabularyItemsCefrLevel {
                name: "B1".to_string(),
            },
            gender: None,
            item: Some("test".to_string()),
            sample_sentence_one: "Johannes flirtet immer mit mir, aber ich will ihn nicht daten. Wir sollten besser Freunde bleiben.".to_string(),
            test_question: "Johannes flirtet immer mit mir, aber ich will ihn nicht  _______________. Wir sollten besser Freunde bleiben.".to_string(),
            title: "test".to_string(),
            word_class: VocabularyItemWordClassType::phrase,
        };
        let expected_text = "\
        Johannes flirtet immer mit mir, aber ich will ihn nicht {{c1::daten}}. Wir sollten besser Freunde bleiben.<br/>\
        <br/>\
        - test\
        ";

        let text_v2 = masked_text_v2(&item);
        assert!(text_v2.is_some());
        assert_eq!(text_v2.unwrap(), expected_text)
    }

    #[test]
    fn masked_text_v2_test_fail() {
        let item = VocabularyItemsVocabularyItems{
            cefr_level: VocabularyItemsVocabularyItemsCefrLevel {
                name: "B1".to_string(),
            },
            gender: None,
            item: Some("available holidays".to_string()),
            sample_sentence_one: "Du hast 21 verfügbare Urlaubstage. So viele sind auch in deinem Arbeitsvertrag vorgesehen.".to_string(),
            test_question: "Du hast 21 _______________. So viele sind auch in deinem Arbeitsvertrag vorgesehen.".to_string(),
            title: "verfügbare Urlaubstage".to_string(),
            word_class: VocabularyItemWordClassType::phrase,
        };

        assert_eq!(masked_text_v2(&item), None)
    }
}
