use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{config::Region, Client};
use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

struct AppState {
    s3_client: Client,
    s3_bucket_name: String,
}

async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "name": "Quick DA Server",
            "success": "true"
        })),
    )
}

#[inline]
async fn submit(state: State<Arc<AppState>>, data: Bytes) -> impl IntoResponse {
    let start = std::time::Instant::now();
    println!("Received data of length: {}", data.len());
    if data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": "false",
                "error": "Empty data",
            })),
        );
    }
    let uuid = Uuid::new_v4();
    let body = aws_sdk_s3::primitives::ByteStream::from(data);
    match state
        .s3_client
        .put_object()
        .bucket(&state.s3_bucket_name)
        .key(uuid)
        .body(body)
        .send()
        .await
    {
        Ok(_) => {
            let elapsed = start.elapsed().as_secs_f64();
            println!("📤 Uploaded to S3 with UUID: {} in {}s", uuid, elapsed);
            (
                StatusCode::CREATED,
                Json(json!({
                    "success": "true"
                })),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": "false"
            })),
        ),
    }
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
        .route("/submit", post(submit))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
