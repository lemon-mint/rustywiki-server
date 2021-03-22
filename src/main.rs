// standard
use std::sync::Mutex;

// thirdparty
#[macro_use]
extern crate diesel;
use actix_web::web::Data;
use actix_web::{get, App, HttpRequest, HttpServer, Responder};

// in crate
mod lib;
mod middleware;
mod models;
mod response;
mod routes;
mod schema;

use lib::AuthValue;

#[get("/")]
async fn test(
    request: HttpRequest, /*, _connection: Data<Mutex<PgConnection>>*/
) -> impl Responder {
    let extensions = request.extensions();
    let auth: &AuthValue = extensions.get::<AuthValue>().unwrap();
    let text = if auth.is_authorized() {
        "인증됨"
    } else {
        "인증 안됨"
    };

    text.to_string()
}

//use diesel::dsl::{exists, select};
use diesel::*;
//use schema::tb_user;
use std::borrow::Borrow;

#[derive(Queryable, Debug)]
pub struct SelectTest {
    pub id: i64,
    pub text: Option<String>,
}

#[get("/foo")]
async fn foo(connection: Data<Mutex<PgConnection>>) -> impl Responder {
    let connection = match connection.lock() {
        Err(_) => {
            log::error!("database connection lock error");
            return "error".to_string();
        }
        Ok(connection) => connection,
    };
    let connection: &PgConnection = Borrow::borrow(&connection);

    // use crate::schema::test;
    // use diesel::dsl::count;

    // // select sum(id) from test
    // let query = test::dsl::test
    //     .group_by(test::dsl::dead_yn)
    //     .select((test::dsl::dead_yn, count(test::dsl::id)));
    // println!(
    //     "query {:?}",
    //     diesel::debug_query::<diesel::pg::Pg, _>(&query)
    // );

    // let result = query.get_results::<(bool, i64)>(connection).unwrap();
    // println!("값 {:?}", result);

    "".to_string()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let _args: Vec<String> = std::env::args().collect();

    let host = "192.168.1.2"; //&args[1];
    let port = 11111; //&args[2];
    let address = format!("{}:{}", host, port);

    let _ = listenfd::ListenFd::from_env();

    let db = Data::new(Mutex::new(lib::establish_connection()));
    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin("http://localhost:11111")
                    .allowed_origin("http://127.0.0.1:11111")
                    .allowed_origin("http://125.133.80.144:11111")
                    .allowed_origin("http://192.168.1.2:11111")
                    .supports_credentials(),
            )
            .wrap(middleware::Logger::new())
            .service(routes::auth::signup)
            .service(routes::auth::login)
            .service(routes::auth::logout)
            .service(routes::auth::refresh)
            .service(foo)
            .service(routes::file::upload_file)
            .service(routes::user::my_info)
            .service(routes::user::close_my_account)
            .service(test)
            .service(routes::doc::write_doc)
            .service(routes::doc::read_doc)
            .service(actix_files::Files::new("/static", "static").show_files_listing())
            .wrap(middleware::Auth::new())
    })
    .bind(address)?
    .run()
    .await
}
