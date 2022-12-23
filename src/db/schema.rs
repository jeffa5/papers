// @generated automatically by Diesel CLI.

diesel::table! {
    papers (id) {
        id -> Integer,
        url -> Nullable<Text>,
        filename -> Text,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        paper_id -> Integer,
        tag -> Text,
    }
}

diesel::joinable!(tags -> papers (paper_id));

diesel::allow_tables_to_appear_in_same_query!(papers, tags,);
