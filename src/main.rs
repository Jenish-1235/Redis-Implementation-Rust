use std::fmt::format;
use std::sync::Arc;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dashmap::DashMap;
use serde::Deserialize;
use serde_json::json;

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
    println!("{:#?}", query);

    HttpResponse::Ok().content_type("application/json").body(format!("{}" , query.key))
}

async fn set_value(req: web::Json<PutRequest> , data: web::Data<Arc<DashMap<String, String>>>) -> HttpResponse {
    let key = req.key.clone();
    let value = req.value.clone();

    println!("{:#?} {:?}", key, value);
    HttpResponse::Ok().body(format!("Key: {}, Value: {}", key, value))
}