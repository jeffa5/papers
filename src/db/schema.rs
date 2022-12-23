// @generated automatically by Diesel CLI.

diesel::table! {
    labels (id) {
        id -> Integer,
        paper_id -> Integer,
        label_key -> Text,
        label_value -> Text,
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
    tags (id) {
        id -> Integer,
        paper_id -> Integer,
        tag -> Text,
    }
}

diesel::joinable!(labels -> papers (paper_id));
diesel::joinable!(tags -> papers (paper_id));

diesel::allow_tables_to_appear_in_same_query!(labels, papers, tags,);
