// standard
use std::sync::Mutex;
use std::borrow::Borrow;

// thirdparty
use actix_web::{
    http::StatusCode, post, delete, put, web, web::Data, HttpResponse, Responder, 
};
use serde::{Deserialize, Serialize};
use diesel::*;
use diesel::dsl::{select, exists};

// in crate
use crate::lib;
use crate::models::InsertUser;
use crate::schema::tb_user;
use crate::response::{ServerErrorResponse};

#[derive(Deserialize, Serialize, Debug)]
pub struct SignupParam {
    pub email: String, 
    pub password: String, 
    pub nickname: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SignupResponse {
    pub success: bool, 
    pub email_duplicated: bool,
    pub message: String,
}

// 회원가입
#[post("/auth/signup")]
pub async fn signup(web::Json(body): web::Json<SignupParam>, connection: Data<Mutex<PgConnection>>) -> impl Responder {
    let connection = match connection.lock() {
        Err(_) => {
            log::error!("database connection lock error");
            let response = ServerErrorResponse::new();
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response);
        }, 
        Ok(connection) => connection,
    };
    let connection:&PgConnection = Borrow::borrow(&connection);

    // email duplicated check
    let already_exists = select(exists(
        tb_user::dsl::tb_user.filter(tb_user::dsl::email.eq(body.email.clone())))
    ).get_result(connection).unwrap();

    if already_exists { 
        let response = SignupResponse{
            success:false, 
            email_duplicated: true, 
            message: "email already exists".to_owned()
        };
        return HttpResponse::build(StatusCode::OK).json(response);
    } 

    // 회원가입 데이터 삽입
    let insert_value = InsertUser::new(body.email, body.password, body.nickname);

    let execute_result = diesel::insert_into(tb_user::table)
        .values(insert_value)
        .execute(connection);

    if execute_result.is_err() {
        log::error!("signup insert query error");
        let response = ServerErrorResponse::new();
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response);
    }

    let response = SignupResponse{
        success: true, 
        email_duplicated: false, 
        message: "".to_owned()
    };
    HttpResponse::build(StatusCode::OK).json(response)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginParam {
    pub email: String, 
    pub password: String, 
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginResponse {
    pub success: bool, 
    pub login_failed: bool,
    pub access_token: String,
    pub refresh_token: String,
    pub message: String,
}

use crate::models::SelectUser;
use crate::models::InsertRefreshToken;
use crate::schema::tb_refresh_token;

// 로그인
#[post("/auth/login")]
pub async fn login(web::Json(body): web::Json<LoginParam>, connection: Data<Mutex<PgConnection>>) -> impl Responder {
    let connection = match connection.lock() {
        Err(_) => {
            log::error!("database connection lock error");
            let response = ServerErrorResponse::new();
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response);
        }, 
        Ok(connection) => connection,
    };
    let connection:&PgConnection = Borrow::borrow(&connection);

    let LoginParam{email, password} = body;

    let query = 
        tb_user::dsl::tb_user
        .filter(tb_user::dsl::email.eq(&email))
        .filter(tb_user::dsl::use_yn.eq(true));

    let user_result = 
        query.load::<SelectUser>(connection);

    match user_result {
        Ok(users) => {
            let response = if users.is_empty() {
                LoginResponse {
                    success: false, 
                    login_failed: true, 
                    access_token: "".to_owned(),
                    refresh_token: "".to_owned(), 
                    message: "login failed".to_owned(),
                }
            } else {
                let user = &users[0];
                let salt = &user.salt;

                let password = lib::hash(password + salt);
                
                if password == user.password {
                    use epoch_timestamp::Epoch;

                    // 리프레시 토큰 생성 및 DB에 삽입
                    let epoch = (Epoch::now() + Epoch::year(1)) as usize;
                    let refresh_token = lib::jwt::sign(epoch, user.id, user.user_type.clone());

                    let insert_value = InsertRefreshToken{token_value: refresh_token.clone(), user_id: user.id};
                    let execute_result = diesel::insert_into(tb_refresh_token::table)
                        .values(insert_value)
                        .execute(connection);

                    if execute_result.is_err() {
                        log::error!("refresh token insert query error");
                        let response = ServerErrorResponse::new();
                        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response);
                    }

                    // 액세스 토큰 생성
                    let epoch = (Epoch::now() + Epoch::hour(2)) as usize;
                    let access_token = lib::jwt::sign(epoch, user.id, user.user_type.clone());

                    LoginResponse {
                        success: true, 
                        login_failed: false, 
                        access_token: access_token, 
                        refresh_token: refresh_token,
                        message: "success".to_owned(),
                    }
                }
                else {
                    LoginResponse {
                        success: false, 
                        login_failed: true, 
                        access_token: "".to_owned(),
                        refresh_token: "".to_owned(), 
                        message: "login failed".to_owned(),
                    }
                }
            };
            
            HttpResponse::build(StatusCode::OK).json(response)
        }
        Err(error) => {
            log::error!("login select query error: {}", error);
            let response = LoginResponse {
                success: false, 
                login_failed: false, 
                access_token: "".to_owned(),
                refresh_token: "".to_owned(),
                message: error.to_string(),
            };

            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response)
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LogoutParam {
    pub refresh_token: String, 
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogoutResponse {
    pub success: bool, 
    pub message: String,
}

// 로그아웃
#[delete("/auth/logout")]
pub async fn logout(web::Json(body): web::Json<LogoutParam>, connection: Data<Mutex<PgConnection>>) -> impl Responder {
    let connection = match connection.lock() {
        Err(_) => {
            log::error!("database connection lock error");
            let response = ServerErrorResponse::new();
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response);
        }, 
        Ok(connection) => connection,
    };
    let connection:&PgConnection = Borrow::borrow(&connection);

    let token = 
        tb_refresh_token::dsl::tb_refresh_token
        .filter(tb_refresh_token::dsl::token_value.eq(&body.refresh_token))
        .filter(tb_refresh_token::dsl::dead_yn.eq(false));

    let result = connection.transaction(|| {
        diesel::update(token).set(tb_refresh_token::dsl::dead_yn.eq_all(true)).execute(connection)?;
        diesel::update(token).set(tb_refresh_token::dsl::dead_utc.eq_all(epoch_timestamp::Epoch::now() as i64)).execute(connection)
    });

    match result {
        Ok(_) => {
            let response = 
                LogoutResponse {
                    success: true, 
                    message: "logout success".to_owned(),
                };
           
            HttpResponse::build(StatusCode::OK).json(response)
        }
        Err(error) => {
            log::error!("logout error: {}", error);
            let response = LogoutResponse {
                success: false, 
                message: error.to_string(),
            };

            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(response)
        }
    }
}