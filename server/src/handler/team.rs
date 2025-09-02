use actix_web::HttpResponse;

pub async fn create() -> HttpResponse {
    HttpResponse::Ok().json("Team created")
}
