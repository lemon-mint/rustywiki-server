// standard
use std::borrow::Borrow;
use std::sync::Mutex;

// thirdparty
use actix_web::{get, http::StatusCode, post, web::Data, HttpRequest, HttpResponse, Responder};
use actix_web_validator::{Json, Query, Validate};
use diesel::dsl::{exists, select};
use diesel::*;
use serde::{Deserialize, Serialize};

// in crate
use crate::lib::AuthValue;
use crate::models::{InsertDocument, InsertDocumentHistory, SelectDocument, SelectDocumentHistory};
use crate::response::{ServerErrorResponse, UnauthorizedResponse};
use crate::schema::{tb_document, tb_document_history};

#[derive(Deserialize, Validate, Debug)]
pub struct WriteDocParam {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WriteDocResponse {
    pub success: bool,
    pub is_new_doc: bool,
    pub message: String,
}

#[post("/doc/document")]
pub async fn write_doc(
    Json(body): Json<WriteDocParam>,
    request: HttpRequest,
    connection: Data<Mutex<PgConnection>>,
) -> impl Responder {
    let connection = match connection.lock() {
        Err(_) => {
            log::error!("database connection lock error");
            let response = ServerErrorResponse::new();
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response);
        }
        Ok(connection) => connection,
    };
    let connection: &PgConnection = Borrow::borrow(&connection);

    // 미인증 접근 거부
    let extensions = request.extensions();
    let nonauth = AuthValue::new();
    let auth: &AuthValue = extensions.get::<AuthValue>().unwrap_or(&nonauth);
    if !auth.is_authorized() {
        let response = UnauthorizedResponse::new();
        return HttpResponse::build(StatusCode::UNAUTHORIZED).json(response);
    }

    // 문서 존재여부 확인
    let exists_document_result = select(exists(
        tb_document::dsl::tb_document.filter(tb_document::dsl::title.eq(&body.title)),
    ))
    .get_result(connection);

    // 문서 개수
    let content_length = body.content.chars().count() as i64;

    // 문서 존재 여부로 분기 처리
    match exists_document_result {
        Ok(exists_document) => {
            let result = if exists_document {
                // 문서 최근 수정일 변경 및
                // 문서 히스토리 추가

                let now_utc = epoch_timestamp::Epoch::now() as i64;

                connection.transaction(|| {
                    let document_id: i64 = diesel::update(tb_document::dsl::tb_document)
                        .filter(tb_document::dsl::title.eq(&body.title))
                        .set(tb_document::dsl::update_utc.eq(now_utc))
                        .returning(tb_document::dsl::id)
                        .get_result(connection)?;

                    // 최근 히스토리 조회
                    let latest_history: SelectDocumentHistory =
                        tb_document_history::dsl::tb_document_history
                            .filter(tb_document_history::dsl::document_id.eq(document_id))
                            .filter(tb_document_history::dsl::latest_yn.eq(true))
                            .order(tb_document_history::dsl::reg_utc.desc())
                            .limit(1)
                            .get_result(connection)?;

                    diesel::update(tb_document_history::dsl::tb_document_history)
                        .filter(tb_document_history::dsl::document_id.eq(document_id))
                        .set(tb_document_history::dsl::latest_yn.eq(false))
                        .execute(connection)?;

                    let increase = content_length - latest_history.char_count;

                    let document_history = InsertDocumentHistory {
                        writer_id: auth.user_id,
                        document_id: document_id,
                        content: body.content.clone(),
                        char_count: content_length,
                        increase: increase,
                        rollback_revision_number: None,
                        revision_number: latest_history.revision_number + 1,
                    };

                    let document_history_id: i64 = diesel::insert_into(tb_document_history::table)
                        .values(document_history)
                        .returning(tb_document_history::dsl::id)
                        .get_result(connection)?;

                    diesel::update(tb_document::dsl::tb_document)
                        .filter(tb_document::dsl::title.eq(&body.title))
                        .set(tb_document::dsl::recent_history_id.eq(document_history_id))
                        .execute(connection)
                })
            } else {
                // 문서 최초 생성
                let document = InsertDocument {
                    title: body.title.clone(),
                };

                connection.transaction(|| {
                    let document_id: i64 = diesel::insert_into(tb_document::table)
                        .values(document)
                        .returning(tb_document::dsl::id)
                        .get_result(connection)?;

                    let document_history = InsertDocumentHistory {
                        writer_id: auth.user_id,
                        document_id: document_id,
                        content: body.content.clone(),
                        char_count: content_length,
                        increase: content_length,
                        rollback_revision_number: None,
                        revision_number: 1,
                    };

                    let document_history_id: i64 = diesel::insert_into(tb_document_history::table)
                        .values(document_history)
                        .returning(tb_document_history::dsl::id)
                        .get_result(connection)?;

                    diesel::update(tb_document::dsl::tb_document)
                        .filter(tb_document::dsl::title.eq(&body.title))
                        .set(tb_document::dsl::recent_history_id.eq(document_history_id))
                        .execute(connection)
                })
            };

            match result {
                Ok(_) => {
                    let response = WriteDocResponse {
                        success: true,
                        is_new_doc: !exists_document,
                        message: "문서 등록&수정 성공".into(),
                    };
                    HttpResponse::build(StatusCode::OK).json(response)
                }
                Err(error) => {
                    log::error!("error: {}", error);
                    let response = ServerErrorResponse::new();
                    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response)
                }
            }
        }
        Err(error) => {
            log::error!("error: {}", error);
            let response = ServerErrorResponse::new();
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response)
        }
    }
}

#[derive(Deserialize, Validate, Debug)]
pub struct ReadDocParam {
    pub title: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReadDocResponse {
    pub success: bool,
    pub not_exists: bool,
    pub title: String,
    pub content: String,
    pub last_update_utc: i64,
    pub message: String,
}

#[get("/doc/document")]
pub async fn read_doc(
    Query(query): Query<ReadDocParam>,
    connection: Data<Mutex<PgConnection>>,
) -> impl Responder {
    let connection = match connection.lock() {
        Err(_) => {
            log::error!("database connection lock error");
            let response = ServerErrorResponse::new();
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response);
        }
        Ok(connection) => connection,
    };
    let connection: &PgConnection = Borrow::borrow(&connection);

    let history: Result<SelectDocumentHistory, diesel::result::Error> = (|| {
        let document: SelectDocument = tb_document::dsl::tb_document
            .filter(tb_document::dsl::title.eq(&query.title))
            .get_result(connection)?;

        tb_document_history::dsl::tb_document_history
            .filter(tb_document_history::dsl::id.eq(document.recent_history_id.unwrap_or(-1)))
            .filter(tb_document_history::dsl::latest_yn.eq(true))
            .get_result(connection)
    })();

    match history {
        Ok(history) => {
            let response = ReadDocResponse {
                success: true,
                not_exists: false,
                title: query.title,
                content: history.content,
                last_update_utc: history.reg_utc,
                message: "성공".into(),
            };
            HttpResponse::build(StatusCode::OK).json(response)
        }
        Err(error) => {
            log::error!("query error: {}", error);
            let response = ReadDocResponse {
                success: false,
                not_exists: true,
                title: query.title,
                content: "".into(),
                last_update_utc: 0,
                message: "실패".into(),
            };
            HttpResponse::build(StatusCode::OK).json(response)
        }
    }
}
