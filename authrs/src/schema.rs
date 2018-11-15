table! {
    oauth_states (state) {
        state -> Nullable<Text>,
    }
}

table! {
    sessions (token) {
        token -> Text,
        expires -> BigInt,
        user_id -> Integer,
    }
}

table! {
    users (id) {
        id -> Nullable<Integer>,
        login_provider -> Nullable<Text>,
        login -> Text,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    oauth_states,
    sessions,
    users,
);
