table! {
    webe_accounts (id) {
        id -> Binary,
        email -> Varchar,
        secret -> Tinytext,
        secret_timeout -> Unsigned<Integer>,
        verified -> Bool,
        verify_code -> Nullable<Char>,
        verify_timeout -> Nullable<Unsigned<Integer>>,
    }
}

table! {
    webe_sessions (token) {
        token -> Char,
        account_id -> Binary,
        timeout -> Unsigned<Integer>,
    }
}

joinable!(webe_sessions -> webe_accounts (account_id));

allow_tables_to_appear_in_same_query!(
    webe_accounts,
    webe_sessions,
);
