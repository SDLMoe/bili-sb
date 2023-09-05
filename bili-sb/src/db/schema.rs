// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vote"))]
    pub struct Vote;
}

diesel::table! {
    segments (id) {
        id -> Uuid,
        cid -> Int8,
        start -> Float4,
        end -> Float4,
        submitter -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        register_time -> Timestamp,
        register_ip -> Cidr,
        last_operation_time -> Nullable<Timestamp>,
    }
}

diesel::table! {
    video_parts (cid) {
        cid -> Int8,
        aid -> Int8,
        #[max_length = 160]
        title -> Varchar,
        duration -> Float4,
    }
}

diesel::table! {
    videos (aid) {
        aid -> Int8,
        #[max_length = 160]
        title -> Varchar,
        update_time -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Vote;

    votes (id, voter) {
        id -> Uuid,
        voter -> Uuid,
        #[sql_name = "type"]
        type_ -> Vote,
    }
}

diesel::joinable!(segments -> users (submitter));
diesel::joinable!(segments -> video_parts (cid));
diesel::joinable!(video_parts -> videos (aid));
diesel::joinable!(votes -> segments (id));
diesel::joinable!(votes -> users (voter));

diesel::allow_tables_to_appear_in_same_query!(
    segments,
    users,
    video_parts,
    videos,
    votes,
);
