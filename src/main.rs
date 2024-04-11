use std::error::Error;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

mod requests;

#[get("/echo")]
async fn echo(req_body: String) -> impl Responder {
    log::debug!("Echo {}", req_body);
    HttpResponse::Ok().body(req_body)
}

#[post("/seed")]
async fn seed() -> impl Responder {
    log::debug!("Seed request");
    let result = requests::seed::seed().await;
    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[post("/drop")]
async fn drop() -> impl Responder {
    log::debug!("Drop request");
    let result = requests::drop::drop().await;
    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[get("/sync")]
async fn sync() -> Result<impl Responder, Box<dyn Error>> {
    log::debug!("Sync request");
    let result = requests::sync::sync().await?;
    Ok(web::Json(result))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(echo)
            .service(seed)
            .service(drop)
            .service(sync)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
