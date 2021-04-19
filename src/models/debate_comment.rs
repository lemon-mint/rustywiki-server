use crate::schema::tb_debate_comment;

#[derive(Insertable, Debug)]
#[table_name = "tb_debate_comment"]
pub struct InsertDebateComment {
    pub debate_id: i64,
    pub writer_id: i64,
    pub content: String,
    pub reg_utc: i64,
    pub use_yn: bool,
}

#[derive(Queryable)]
pub struct SelectDebateComment {
    pub id: i64,
    pub debate_id: i64,
    pub writer_id: i64,
    pub content: String,
    pub reg_utc: i64,
    pub use_yn: bool,
}
