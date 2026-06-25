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
use varland::{
    environment::{Environment, Value},
    parser,
};

type SharedEnvironment = Arc<Mutex<Environment>>;

async fn get_num_cols(State(environment): State<SharedEnvironment>) -> Json<usize> {
    let num_cols = environment.lock().unwrap().num_cols();
    Json(num_cols)
}

async fn get_num_rows(State(environment): State<SharedEnvironment>) -> Json<usize> {
    let num_rows = environment.lock().unwrap().num_rows();
    Json(num_rows)
}

#[derive(Deserialize)]
struct GetRawRequest {
    col: usize,
    row: usize,
}

#[debug_handler]
async fn get_raw(
    State(environment): State<SharedEnvironment>,
    Json(request): Json<GetRawRequest>,
) -> Json<Value> {
    let raw_value = environment.lock().unwrap().get_raw(request.col, request.row);

    Json(raw_value)
}

#[derive(Deserialize)]
struct GetEvaluatedRangeRequest {
    start_col: usize,
    start_row: usize,
    end_col: usize,
    end_row: usize,
}

#[debug_handler]
async fn get_evaluated_range(
    State(environment): State<SharedEnvironment>,
    Json(request): Json<GetEvaluatedRangeRequest>,
) -> Json<Vec<Vec<Result<Value, String>>>> {
    environment
        .lock()
        .unwrap()
        .evaluate_range(
            request.start_col,
            request.start_row,
            request.end_col,
            request.end_row,
        );
    let results = environment
        .lock()
        .unwrap()
        .get_evaluated_range(request.start_col, request.start_row, request.end_col, request.end_row);

    Json(results)
}

#[derive(Deserialize)]
struct SetRawValueRequest {
    col: usize,
    row: usize,
    value: Value,
}

#[debug_handler]
async fn set_raw(
    State(environment): State<SharedEnvironment>,
    Json(request): Json<SetRawValueRequest>,
) -> StatusCode {
    environment
        .lock()
        .unwrap()
        .set_raw(request.col, request.row, request.value);

    StatusCode::CREATED
}

#[debug_handler]
async fn parse(Json(request): Json<String>) -> Json<Result<Value, String>> {
    match parser::parse(&request) {
        Ok(("", value)) => Json(Ok(value)),
        Ok((rest, _)) => Json(Err(format!("Unexpected trailing characters: {rest}"))),
        Err(e) => Json(Err(format!("Failed to parse: {e}"))),
    }
}

#[tokio::main]
async fn main() {
    // let environment = Environment::new_from_csv("2024plays-short.csv").unwrap();
    let environment = Environment::new();
    let environment = Arc::new(Mutex::new(environment));

    let api_router = Router::new()
        .route("/get/num-cols", get(get_num_cols))
        .route("/get/num-rows", get(get_num_rows))
        .route("/get/raw", post(get_raw))
        .route("/get/evaluated-range", post(get_evaluated_range))
        .route("/set/raw", post(set_raw))
        .route("/parse", post(parse));
    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(ServeDir::new("client"))
        .layer(CorsLayer::permissive())
        .with_state(environment);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
