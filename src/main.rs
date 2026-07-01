use api::{
    common::room_log::RoomLogger,
    modules::room::websocket::hub::RoomHub,
    routes::{health::health_routes, room::room_routes},
    state::{AppState, SecretStore},
};
use axum::Router;
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

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

    // let app = Router::new()
    //     .merge(health_routes)
    //     .merge(room_routes())
    //     .layer(cors)
    //     .with_state(app_state.clone());

    // let port = 3000;

    let app = Router::new()
        .merge(health_routes)
        .merge(room_routes())
        .nest_service("/media/hls", ServeDir::new("/app/media/hls"))
        .nest_service("/keys", ServeDir::new("/app/media/keys"))
        .layer(cors)
        .with_state(app_state.clone());

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid u16");
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(addr).await.unwrap();

    // println!("API running on: http://localhost:{}/", port);

    println!("API running on port {}", port);

    axum::serve(listener, app).await.unwrap();
}
