use actix_web::{get, web, Responder};
use serde::{Deserialize, Serialize};

use potential_giggle::CheckClient;

#[derive(Serialize, Deserialize)]
struct ErrorJSON {
    error: String,
}

#[get("/{domain_name}")]
pub async fn show_domain_name(web::Path(domain_name): web::Path<String>) -> impl Responder {
    let client = CheckClient::new();
    match client.check_certificate(&domain_name) {
        Ok(r) => serde_json::to_string(&r.to_json()).expect("unable to serialize JSON"),
        Err(e) => serde_json::to_string(&ErrorJSON {
            error: format!("{:?}", e),
        })
        .expect("unable to serialize JSON"),
    }
}
