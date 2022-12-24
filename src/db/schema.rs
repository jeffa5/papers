// @generated automatically by Diesel CLI.

diesel::table! {
    labels (paper_id, label_key) {
        paper_id -> Integer,
        label_key -> Text,
        label_value -> Text,
    }
}

diesel::table! {
    notes (id) {
        id -> Integer,
        paper_id -> Integer,
        content -> Text,
    }
}

diesel::table! {
    papers (id) {
        id -> Integer,
        url -> Nullable<Text>,
        filename -> Text,
        title -> Nullable<Text>,
    }
}

diesel::table! {
    tags (paper_id, tag) {
        paper_id -> Integer,
        tag -> Text,
    }
}

diesel::joinable!(labels -> papers (paper_id));
diesel::joinable!(notes -> papers (paper_id));
diesel::joinable!(tags -> papers (paper_id));

diesel::allow_tables_to_appear_in_same_query!(labels, notes, papers, tags,);
