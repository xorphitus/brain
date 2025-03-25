use axum::{
    routing::post,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Query {
    text: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

async fn hello(Json(query): Json<Query>) -> Json<Response> {
    Json(Response {
        message: format!("Hello! Your query was: {}", query.text)
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/query", post(hello));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await.unwrap();
}
