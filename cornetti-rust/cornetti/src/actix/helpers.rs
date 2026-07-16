use crate::core::models::CornettiError;
use serde_json;

/// Returns a default actix-web route that responds with a JSON 404 `CornettiError`.
pub fn default_404_json() -> actix_web::Route {
    actix_web::web::to(|_req: actix_web::HttpRequest| async {
        actix_web::HttpResponse::NotFound()
            .content_type("application/json")
            .body(
                serde_json::to_string(&CornettiError {
                    status: 404,
                    detail: "Not found".into(),
                })
                .unwrap(),
            )
    })
}
