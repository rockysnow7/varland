use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use axum_macros::debug_handler;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};
use varland::environment::{Environment, Value};

type SharedEnvironment = Arc<Mutex<Environment>>;

#[debug_handler]
async fn get_raw_state(
    State(environment): State<SharedEnvironment>,
) -> Json<Vec<Vec<Value>>> {
    let raw_state = environment.lock().unwrap().get_raw_state();

    Json(raw_state)
}

#[debug_handler]
async fn get_evaluated_state(
    State(environment): State<SharedEnvironment>,
) -> Json<Vec<Vec<Result<Value, String>>>> {
    environment.lock().unwrap().evaluate_all();
    let evaluated_state = environment.lock().unwrap().get_evaluated_state();

    Json(evaluated_state)
}

#[derive(Deserialize)]
struct SetRawValueRequest {
    col: usize,
    row: usize,
    value: Value,
}

#[debug_handler]
async fn set_raw_value(
    State(environment): State<SharedEnvironment>,
    Json(request): Json<SetRawValueRequest>,
) -> StatusCode {
    environment
        .lock()
        .unwrap()
        .set_value_raw(request.col, request.row, request.value);

    StatusCode::CREATED
}

#[tokio::main]
async fn main() {
    let environment = Arc::new(Mutex::new(Environment::new()));

    let api_router = Router::new()
        .route("/get-raw-state", get(get_raw_state))
        .route("/get-evaluated-state", get(get_evaluated_state))
        .route("/set-raw-value", post(set_raw_value));
    let app = Router::new()
        .nest("/api", api_router)
        .with_state(environment);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
