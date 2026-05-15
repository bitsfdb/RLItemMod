mod engine;

use axum::{
    routing::{get, post},
    Json, Router,
    extract::Query,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use crate::engine::psynet::PsynetClient;

#[derive(Deserialize)]
struct FetchParams {
    token: String,
    account: String,
}

struct AppState {
    items: Mutex<Value>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    let state = Arc::new(AppState {
        items: Mutex::new(json!({"Items": []})),
    });

    let app = Router::new()
        .route("/items.json", get(handle_get_items))
        .route("/fetch", post(handle_fetch_catalog))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("VelocityRL API Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_get_items(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<Value> {
    let items = state.items.lock().unwrap();
    Json(items.clone())
}

async fn handle_fetch_catalog(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(params): Json<FetchParams>,
) -> Result<Json<Value>, String> {
    let mut client = PsynetClient::new(params.token);
    client.login(&params.account).await.map_err(|e| e.to_string())?;
    
    let products = client.get_all_products().await.map_err(|e| e.to_string())?;
    
    let mut new_items = Vec::new();
    for p in products {
        new_items.push(json!({
            "ID": p["ProductID"],
            "Product": p["Label"].as_str().unwrap_or("Unknown"),
            "Quality": p["Quality"].as_str().unwrap_or("Common"),
            "Slot": p["Slot"].as_str().unwrap_or("Unknown"),
            "AssetPackage": "", 
            "AssetPath": "",
            "image_url": p["Thumbnail"].as_str().unwrap_or("")
        }));
    }

    let mut items_data = state.items.lock().unwrap();
    *items_data = json!({ "Items": new_items });

    Ok(Json(items_data.clone()))
}
