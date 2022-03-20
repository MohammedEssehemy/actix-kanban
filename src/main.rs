use actix_web::{ HttpServer, web::{ Data } };
mod logger;
mod models;
mod db;
mod dump_routes;
mod routes;

type StdErr = Box<dyn std::error::Error>;

#[actix_web::main]
async fn main() -> Result<(), StdErr> {
    // loads env variables from .env
    dotenv::dotenv().ok();
    logger::init()?;

    let db = db::Db::connect().await?;
   HttpServer::new(move || actix_web::App::new()
   .app_data(Data::new(db.clone()))
   .service(dump_routes::api())
   .service(routes::api())
)
       .bind(("127.0.0.1", 8000))?
       .run()
       .await?;

    Ok(())
}