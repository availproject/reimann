use alloy_primitives::{
    hex::{FromHex},
    B256,
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
mod incremental_merkle_tree;
use incremental_merkle_tree::MerkleTree;
use parking_lot::RwLock;
use serde_json::json;
use std::sync::Arc;

struct AppState {
    pub tree: MerkleTree,
}

async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "name": "Fast SMT Server",
            "success": "true"
        })),
    )
}

#[inline]
async fn add(state: State<Arc<RwLock<AppState>>>, data: Bytes) -> impl IntoResponse {
    let node = match B256::from_hex(data.as_ref()) {
        Ok(node) => node,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Leaf was not parseable as a 32-byte hex string"
                })),
            )
        }
    };
    let mut state = state.write();
    let index = state.tree.len();
    state.tree.append(node);
    let root = state.tree.root();
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "root": root,
            "index": index
        })),
    )
}

#[inline]
async fn query(state: State<Arc<RwLock<AppState>>>, Path(index): Path<u32>) -> impl IntoResponse {
    let state = state.read();
    let root = state.tree.root();
    let proof = state.tree.generate_proof(index.try_into().unwrap());
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "proof": proof,
            "root": root
        })),
    )
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let tree = MerkleTree::new(32);
    let app_state = Arc::new(RwLock::new(AppState { tree }));
    let app = Router::new()
        .route("/", get(root))
        .route("/add", post(add))
        .route("/query/:index", get(query))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
