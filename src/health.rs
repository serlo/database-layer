use actix_web::{get, web, Result};

#[get("/.well-known/health")]
async fn health() -> Result<String> {
    Ok(String::from("Ok"))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}
