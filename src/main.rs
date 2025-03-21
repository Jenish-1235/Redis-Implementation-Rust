use std::sync::Arc;
use actix_web::{http, web, App, HttpResponse, HttpServer};
use dashmap::DashMap;
use serde::Deserialize;

type Data = Arc<DashMap<String, String>>;

#[derive(Deserialize, Debug)]
struct GetQuery {
    key: String
}

#[derive(Deserialize, Debug)]
struct PutRequest{
    key: String,
    value: String,
}

struct GetResponse {
    status : String,
    key : String,
    value: String,
}

struct FailureResponse {
    status: String,
    message: String
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Arc::new(DashMap::<String, String>::new());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(data.clone()))
            .route("/get", web::get().to(get_value))
            .route("/put", web::post().to(set_value))
    })
        .bind("127.0.0.1:7171")?
        .run()
        .await
}



async fn get_value(query: web::Query<GetQuery> , data: web::Data<Data>) -> HttpResponse {
    if data.contains_key(&query.key) {
        HttpResponse::Ok().content_type("application/json").body(format!("status: {} , key: {}, value: {}", "OK", query.key.clone(), data.get(&query.key).unwrap().clone()))
    }else if !data.contains_key(&query.key) {
        HttpResponse::Ok().content_type("application/json").body(format!("status: {} , message: {}", "ERROR", "Key not found."))
    }else{
        HttpResponse::BadRequest().content_type("application/json").body(format!("status: {} , message: {}", "ERROR", "Error description explaining what went wrong."))
    }
}

async fn set_value(req: web::Json<PutRequest> , data: web::Data<Arc<DashMap<String, String>>>) -> HttpResponse {
    if data.contains_key(&req.key) {
        data.insert(req.0.key.clone(), req.0.value.clone());
        HttpResponse::Ok().content_type("application/json").body(format!("status: {}, message: {}", "OK", "Key updated successfully."))
    }else if !data.contains_key(&req.key) {
        data.insert(req.0.key.clone(), req.0.value.clone());
        HttpResponse::Ok().content_type("application/json").body(format!("status: {}, message: {}", "OK", "Key inserted successfully."))
    }else{
        HttpResponse::build(http::StatusCode::CREATED).content_type("application/json").body(format!("status: {}, message: {}", "ERROR", "Error"))
    }
}