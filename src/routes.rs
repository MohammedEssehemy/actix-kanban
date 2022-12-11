use actix_web::{
    dev::{HttpServiceFactory, Payload},
    error::InternalError,
    http::StatusCode,
    web::{scope, Data, Json, Path},
    FromRequest, HttpRequest, HttpResponse,
};
use futures::{future, Future, FutureExt};
use std::pin::Pin;

use crate::db::Db;
use crate::models::*;
use crate::StdErr;

impl FromRequest for Token {
    type Error = InternalError<&'static str>;

    // we return a Future that is either
    // - immediately ready (on a bad request with a missing or malformed Authorization header)
    // - ready later (pending on a SQL query that validates the request's Bearer token)
    type Future = future::Either<
        future::Ready<Result<Self, Self::Error>>,
        Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + 'static>>,
    >;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // get request headers
        let headers = req.headers();

        // check that Authorization header exists
        let maybe_auth = headers.get("Authorization");
        if maybe_auth.is_none() {
            return future::err(InternalError::new(
                "missing Authorization header",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check Authorization header is valid utf-8
        let auth_config = maybe_auth.unwrap().to_str();
        if auth_config.is_err() {
            return future::err(InternalError::new(
                "malformed Authorization header",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check Authorization header specifies some authorization strategy
        let mut auth_config_parts = auth_config.unwrap().split_ascii_whitespace();
        let maybe_auth_type = auth_config_parts.next();
        if maybe_auth_type.is_none() {
            return future::err(InternalError::new(
                "missing Authorization type",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check that authorization strategy is using a bearer token
        let auth_type = maybe_auth_type.unwrap();
        if auth_type != "Bearer" {
            return future::err(InternalError::new(
                "unsupported Authorization type",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check that bearer token is present
        let maybe_token_id = auth_config_parts.next();
        if maybe_token_id.is_none() {
            return future::err(InternalError::new(
                "missing Bearer token",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // we can fetch managed application data using HttpRequest.app_data::<T>()
        let db = req.app_data::<Data<Db>>();
        if db.is_none() {
            return future::err(InternalError::new(
                "internal error",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
            .left_future();
        }

        // clone these so that we can return an impl Future + 'static
        let db = db.unwrap().clone();
        let token_id = maybe_token_id.unwrap().to_owned();

        async move {
            db.validate_token(token_id)
                .await
                .map_err(|_| InternalError::new("invalid Bearer token", StatusCode::UNAUTHORIZED))
        }
        .boxed_local()
        .right_future()
    }
}

// some convenience functions

fn to_internal_error(e: StdErr) -> InternalError<StdErr> {
    InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
}

fn to_ok(_: ()) -> HttpResponse {
    HttpResponse::new(StatusCode::OK)
}

// board routes

#[actix_web::get("/boards")]
async fn boards(db: Data<Db>, _t: Token) -> Result<Json<Vec<Board>>, InternalError<StdErr>> {
    db.boards().await.map(Json).map_err(to_internal_error)
}

#[actix_web::post("/boards")]
async fn create_board(
    db: Data<Db>,
    create_board: Json<CreateBoard>,
    _t: Token,
) -> Result<Json<Board>, InternalError<StdErr>> {
    db.create_board(create_board.into_inner())
        .await
        .map(Json)
        .map_err(to_internal_error)
}

#[actix_web::get("/boards/{board_id}/summary")]
async fn board_summary(
    db: Data<Db>,
    board_id: Path<i64>,
    _t: Token,
) -> Result<Json<BoardSummary>, InternalError<StdErr>> {
    db.board_summary(board_id.into_inner())
        .await
        .map(Json)
        .map_err(to_internal_error)
}

#[actix_web::delete("/boards/{board_id}")]
async fn delete_board(
    db: Data<Db>,
    board_id: Path<i64>,
    _t: Token,
) -> Result<HttpResponse, InternalError<StdErr>> {
    db.delete_board(board_id.into_inner())
        .await
        .map(to_ok)
        .map_err(to_internal_error)
}

// card routes

#[actix_web::get("/boards/{board_id}/cards")]
async fn cards(
    db: Data<Db>,
    board_id: Path<i64>,
    _t: Token,
) -> Result<Json<Vec<Card>>, InternalError<StdErr>> {
    db.cards(board_id.into_inner())
        .await
        .map(Json)
        .map_err(to_internal_error)
}

#[actix_web::post("/cards")]
async fn create_card(
    db: Data<Db>,
    create_card: Json<CreateCard>,
    _t: Token,
) -> Result<Json<Card>, InternalError<StdErr>> {
    db.create_card(create_card.into_inner())
        .await
        .map(Json)
        .map_err(to_internal_error)
}

#[actix_web::patch("/cards/{card_id}")]
async fn update_card(
    db: Data<Db>,
    card_id: Path<i64>,
    update_card: Json<UpdateCard>,
    _t: Token,
) -> Result<Json<Card>, InternalError<StdErr>> {
    db.update_card(card_id.into_inner(), update_card.into_inner())
        .await
        .map(Json)
        .map_err(to_internal_error)
}

#[actix_web::delete("/cards/{card_id}")]
async fn delete_card(
    db: Data<Db>,
    card_id: Path<i64>,
    _t: Token,
) -> Result<HttpResponse, InternalError<StdErr>> {
    db.delete_card(card_id.into_inner())
        .await
        .map(to_ok)
        .map_err(to_internal_error)
}

// single public function which returns all of the API request handlers

pub fn api() -> impl HttpServiceFactory + 'static {
    scope("/api")
        .service(boards)
        .service(board_summary)
        .service(create_board)
        .service(delete_board)
        .service(cards)
        .service(create_card)
        .service(update_card)
        .service(delete_card)
}
