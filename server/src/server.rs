use actix_web::{get, web, Responder};
use potential_giggle::{CheckClient, CheckResultJSON};
use serde_json::json;

pub struct SharedState {
    client: CheckClient,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            client: CheckClient::new(),
        }
    }
}

#[get("/{domain_name}")]
pub async fn show_domain_name(
    data: web::Data<SharedState>,
    web::Path(domain_name): web::Path<String>,
) -> impl Responder {
    let client = &data.client;
    let string = match client.check_certificate(&domain_name) {
        Ok(r) => {
            let json = CheckResultJSON::new(&r);
            serde_json::to_string(&json)
        }
        Err(e) => {
            let json = json!({ "error": format!("{:?}", e) });
            serde_json::to_string(&json)
        }
    };
    string.expect("unable to serialize JSON")
}
