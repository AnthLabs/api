use api::{
    common::room_log::RoomLogger, modules::room::websocket::hub::RoomHub, routes::{health::health_routes, room::room_routes}, state::{AppState, SecretStore}
};
use axum::Router;
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mongo_uri = std::env::var("MONGO_URI").expect("missing MONGO_URI");

    let secret_store = SecretStore;
    let database = mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .unwrap()
        .database("api");

    let app_state = AppState {
        secret_store,
        database,
        room_hub: RoomHub::new(),
        room_logger: RoomLogger::new(),
        started_at: std::time::Instant::now(),
    };

    let health_routes = health_routes();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(health_routes)
        .merge(room_routes())
        .layer(cors)
        .with_state(app_state.clone());

    let port = 3000;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(addr).await.unwrap();

    println!("API running on: http://localhost:{}/", port);
    axum::serve(listener, app).await.unwrap();
}
