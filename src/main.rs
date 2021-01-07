use actix_web::{get, web, App, HttpServer, Result};

#[get("/uuid/{uuid}")]
async fn hello(web::Path(uuid): web::Path<u32>) -> Result<String> {
    Ok(format!("Hello {}", uuid))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
