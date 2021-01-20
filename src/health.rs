use actix_web::{get, web, Result};

#[get("/.well-known/health")]
async fn health() -> Result<String> {
    Ok("ok".to_string())
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}
