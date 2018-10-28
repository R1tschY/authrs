table! {
    projects (project) {
        project -> Integer,
        name -> Integer,
        ctime -> Integer,
    }
}

table! {
    ticket_attributes (id) {
        id -> Nullable<Integer>,
        ticket -> Integer,
        field -> Text,
        value -> Text,
    }
}

table! {
    ticket_schema_fields (id) {
        id -> Nullable<Integer>,
        project -> Integer,
        name -> Text,
        elements_mode -> Integer,
    }
}

table! {
    tickets (id) {
        id -> Nullable<Integer>,
        project -> Integer,
        number -> Integer,
        state -> Integer,
        title -> Text,
        body -> Text,
        ctime -> Integer,
    }
}

table! {
    workflow_states (id) {
        id -> Nullable<Integer>,
        project -> Integer,
        state -> Text,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    projects,
    ticket_attributes,
    ticket_schema_fields,
    tickets,
    workflow_states,
);
