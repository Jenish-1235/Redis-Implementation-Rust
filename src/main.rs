use std::sync::Arc;
use actix_web::{http, web, App, HttpResponse, HttpServer};
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
    match data.get(&query.key) {
        Some(value) => {
            HttpResponse::Ok().json(json!({
                "status": "OK",
                "key": query.key,
                "value": value.clone(),
            }))
        }
        None => {
            HttpResponse::NotFound().json(json!({
                "status": "ERROR",
                "message": "Key not found.",
            }))
        }
    }
}

async fn set_value(req: web::Json<PutRequest> , data: web::Data<Arc<DashMap<String, String>>>) -> HttpResponse {
    let key_exists = data.contains_key(&req.key);

    data.insert(req.key.clone(), req.value.clone());
    let response = json!({
        "status": "OK",
        "message": if key_exists {
            "Key updated successfully."
        } else{
            "Key inserted successfully."
        }
    });

    let status_code = if key_exists {
        http::StatusCode::OK
    }else{
        http::StatusCode::CREATED
    };

    HttpResponse::build(status_code).json(response)
}