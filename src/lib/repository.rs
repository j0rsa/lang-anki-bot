use diesel::pg::PgConnection;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use diesel::r2d2::ConnectionManager;
use super::models::service_credential::ServiceCredential;
use crate::schema::{
    token,
};
use super::errors::Result;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// pub fn get_token(pool: &Pool, service: &String, login: &String) -> Result<Option<ServiceCredential>> {
//     let connection = pool.get()?;
//     token::table
//         .filter(token::login.eq(login))
//         .filter(token::service.eq(service))
//         .first::<ServiceToken>(&connection)
//         .optional()
//         .map(|token| token.map(|token| token.into()))
//         .map_err(|e| e.into())
// }

// pub fn save_token(pool: &Pool, service: &String, login: &String, password: &String, token: &ServiceCredential) -> Result<()> {
//     let connection = pool.get()?;
//     let db_token = ServiceToken::from(service, login, password, token);
//     diesel::insert_into(token::table)
//         .values(&db_token)
//         .on_conflict(token::login)
//         .do_update()
//         .set(&db_token)
//         .execute(&connection)?;
//     Ok(())
// }

#[derive(Insertable, Queryable, AsChangeset, Clone, Debug)]
#[diesel(table_name = token)]
struct ServiceToken {
    pub id: uuid::Uuid,
    pub service: String,
    pub login: String,
    pub password: String,
    pub value: String,
    pub expires_at: chrono::NaiveDateTime,
}

impl Into<ServiceCredential> for ServiceToken {
    fn into(self) -> ServiceCredential {
        let expires = self.expires_at.timestamp_millis() as u128;
        ServiceCredential::new(self.value, "".to_string(), "".to_string(), expires)
    }
}

impl ServiceToken {
    fn from(service: &String, login: &String, password: &String, cred: &ServiceCredential) -> ServiceToken {
        // let timestamp_string = cred.as_ref().expires.to_string();
        let timestamp_string = "".to_string();
        let secs = timestamp_string[..timestamp_string.len() - 6].parse().unwrap();
        let microsecs = timestamp_string[timestamp_string.len() - 6..].parse::<u32>().unwrap();
        ServiceToken {
            id: uuid::Uuid::new_v4(),
            service: service.clone(),
            login: login.clone(),
            value: "".to_string(),
            password: password.clone(),
            expires_at: chrono::NaiveDateTime::from_timestamp_opt(secs, microsecs * 1000).unwrap(),
        }
    }
}
