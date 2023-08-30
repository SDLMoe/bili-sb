// @generated automatically by Diesel CLI.

diesel::table! {
    segments (id) {
        id -> Uuid,
        cid -> Int8,
        start_seg -> Float4,
        end_seg -> Float4,
        upvote -> Int4,
        downvote -> Int4,
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

diesel::joinable!(segments -> users (submitter));
diesel::joinable!(segments -> video_parts (cid));
diesel::joinable!(video_parts -> videos (aid));

diesel::allow_tables_to_appear_in_same_query!(
    segments,
    users,
    video_parts,
    videos,
);
