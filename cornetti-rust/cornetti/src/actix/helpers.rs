use crate::errors;

/// Returns a default actix-web route that responds with a JSON 404 `CornettiError`.
pub fn default_404_json() -> actix_web::Route {
    actix_web::web::to(|_req: actix_web::HttpRequest| async {
        actix_web::HttpResponse::NotFound()
            .content_type("application/json")
            .body(
                serde_json::to_string(&errors::not_found::resource_not_found())
                    .unwrap(),
            )
    })
}
