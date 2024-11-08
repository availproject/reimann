use axum::{
    body::Bytes, extract::State, http::StatusCode, response::{IntoResponse, Json}, routing::{get, put}, Router
};
use serde_json::{Value, json};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{config::Region, Client};
use uuid::Uuid;
use dotenvy::dotenv;
use std::sync::Arc;

struct AppState {
    s3_client: Client,
    s3_bucket_name: String
}

async fn root() -> impl IntoResponse {
    return (StatusCode::OK, Json(json!({
        "name": "Fast DA Server",
        "success": "true"
    })));
}

async fn submit(state: State<Arc<AppState>>, data: Bytes) -> impl IntoResponse {
    println!("Received data: {:?}", data);
    if data.len() == 0 {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": "No data provided",
            "success": "false"
        })))
    }
    let body = aws_sdk_s3::primitives::ByteStream::from(data);
    match state.s3_client
        .put_object()
        .bucket(&state.s3_bucket_name)
        .key(Uuid::new_v4().to_string())
        .body(body)
        .send()
        .await {
        Ok(_) => {
            return (StatusCode::CREATED, Json(json!({
                "success": "true"
            })))
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": "false"
            })))
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let region_provider = RegionProviderChain::first_try(Region::new(dotenvy::var("AWS_REGION")?));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = Client::new(&shared_config);
    let s3_bucket_name = dotenvy::var("S3_BUCKET")?;
    let app_state = Arc::new(AppState {
        s3_client,
        s3_bucket_name,
    });
    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        .route("/submit", put(submit))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
