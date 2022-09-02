table! {
    token(id) {
        id -> Uuid,
        service -> Text,
        login -> Text,
        password -> Text,
        value -> Text,
        expires_at -> Timestamp,
    }
}
