use actix_files::NamedFile;
use actix_web::{
    body::BoxBody,
    dev::HttpServiceFactory,
    get,
    web::{scope, Data, Json, Path},
    HttpRequest, HttpResponse, Responder, error::InternalError, http::StatusCode,
};
use serde::de::{Deserialize, Deserializer, Error};

use crate::{db::Db, models::Status};

#[get("/")]
pub async fn hello_world() -> impl Responder {
    "Hello, world!"
}

// actix_web::web::Path can extract anything out of the
// URL path as long as it impls serde::Deserialize
#[get("/echo/{string}/{num}/{maybe}/etc")]
pub async fn echo_path(params: Path<(String, usize, bool)>) -> impl Responder {
    let (string, num, maybe) = params.into_inner();
    format!("got string {}, num {}, and maybe {}", string, num, maybe)
}

#[get("/cargo")]
pub async fn returns_cargo() -> impl Responder {
    NamedFile::open_async("Cargo.toml").await
}
// custom type example
pub struct EvenNumber(i32);

// hand-written deserialize impl, mostly deferring to i32::deserialize
impl<'de> Deserialize<'de> for EvenNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        if value % 2 == 0 {
            Ok(EvenNumber(value))
        } else {
            Err(D::Error::custom("not even"))
        }
    }
}

// actix_web also provides Responder impls for
// - Option<T> where T: Responder
//     - returns T if Some(T), 404 Not Found if None
// - Result<T, E> where T: Responder, E: Into<ActixError>
//     - returns T if Ok(T), otherwise ActixError::from(e) if Err(e)

// example Responder impl
impl Responder for EvenNumber {
    // type Error = InternalError<&'static str>;
    // type Future = Ready<Result<HttpResponse, Self::Error>>;
    type Body = BoxBody;
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let res = HttpResponse::Ok()
            .insert_header(("X-Number-Parity", "Even"))
            .body(format!("returning even number {}", self.0));
        res
    }
}

// but now we can extract EvenNumbers directly from the Path:
#[get("/even/{even_num}")]
pub async fn echo_even(path: Path<EvenNumber>) -> impl Responder {
    let even_num = path.into_inner();
    even_num
}

#[get("/use/db")]
pub async fn use_db(db: Data<Db>) -> impl Responder {
    db.boards().await.map(Json).map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))
}

#[get("/example/json")]
pub async fn return_json() -> Json<Status> {
    Json(Status::Todo)
}

pub fn api() -> impl HttpServiceFactory + 'static {
    scope("/dump")
        .service(hello_world)
        .service(echo_path)
        .service(returns_cargo)
        .service(echo_even)
        .service(use_db)
        .service(return_json)
}
