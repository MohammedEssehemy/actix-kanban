use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;

mod db;
mod dump_routes;
mod logger;
mod models;
mod routes;

use db::Db;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    // loads env variables from .env
    dotenv().ok();
    logger::init()?;

    let db = Db::connect().await?;
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .service(dump_routes::api())
            .service(routes::api())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;

    Ok(())
}
