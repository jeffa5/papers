// @generated automatically by Diesel CLI.

diesel::table! {
    papers (filename) {
        filename -> Text,
        url -> Nullable<Text>,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        paper_id -> Integer,
        tag -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(papers, tags,);
