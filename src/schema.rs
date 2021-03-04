table! {
    Document (id) {
        id -> Int8,
        title -> Text,
        reg_utc -> Int8,
    }
}

table! {
    History (id, writer_id, document_id) {
        id -> Int8,
        writer_id -> Int8,
        document_id -> Int8,
        filepath -> Text,
        increase -> Int8,
        reg_date -> Int8,
    }
}

table! {
    tb_debate (id, document_id, writer_id) {
        id -> Int8,
        document_id -> Int8,
        writer_id -> Int8,
        subject -> Text,
        content -> Text,
        reg_utc -> Int8,
        open_yn -> Bool,
    }
}

table! {
    tb_debate_comment (id, id2, writer_id) {
        id -> Int8,
        id2 -> Int8,
        writer_id -> Int8,
        content -> Text,
        reg_utc -> Int8,
        open_yn -> Bool,
    }
}

table! {
    tb_document (id) {
        id -> Int8,
        title -> Text,
        reg_utc -> Int8,
    }
}

table! {
    tb_document_history (id, writer_id, document_id) {
        id -> Int8,
        writer_id -> Int8,
        document_id -> Int8,
        filepath -> Text,
        increase -> Int8,
        reg_date -> Int8,
    }
}

table! {
    tb_image (id, uploader_id) {
        id -> Int4,
        uploader_id -> Int8,
        domain -> Nullable<Text>,
        path -> Text,
        use_yn -> Bool,
        reg_utc -> Int8,
        Field -> Nullable<Varchar>,
    }
}

table! {
    tb_refresh_token (token_value, user_id) {
        token_value -> Text,
        user_id -> Int8,
        reg_utc -> Int8,
        dead_yn -> Bool,
        dead_utc -> Nullable<Int8>,
    }
}

table! {
    tb_user (id) {
        id -> Int8,
        email -> Varchar,
        salt -> Varchar,
        password -> Text,
        user_type -> Varchar,
        nickname -> Varchar,
        use_yn -> Bool,
        reg_utc -> Int8,
    }
}

table! {
    test (id) {
        id -> Int8,
        text -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    Document,
    History,
    tb_debate,
    tb_debate_comment,
    tb_document,
    tb_document_history,
    tb_image,
    tb_refresh_token,
    tb_user,
    test,
);
