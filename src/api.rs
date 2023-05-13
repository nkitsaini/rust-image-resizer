
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::resizer:: {self, ImageResizeParams};
use tokio::task;



async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn docs() -> &'static str {
    "Image Resizer API:
POST /resize?width=100&height=20&jpeg_quality=20

jpeg_quality is optional and should range in 1-100
width and height are required and should be non-zero integers
POST body should be the image bytes

Response body will contain output image bytes in jpeg format
"
}


#[axum::debug_handler]
async fn image_resizer(params: Query<ImageResizeParams>, body: Bytes) -> axum::response::Response {
    let params = params.0;
    match task::spawn(async { resizer::reszier(body, params) }).await {
        Ok(x) => match x {
            Ok(x) => (StatusCode::OK, x).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e:?}")).into_response()
            
        },
        Err(_e) => ( StatusCode::INTERNAL_SERVER_ERROR, "Can't start a new thread",) .into_response()
        
    }
}

#[axum::debug_handler]
async fn image_resizer_test(_params: Query<ImageResizeParams>, body: Bytes) -> axum::response::Response {
    format!("Hey, {}", body.len()).into_response()
}


pub fn init_router() -> anyhow::Result<Router> {
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/resize", post(image_resizer))
        .route("/docs", get(docs))
        .route("/test", post(image_resizer_test))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10)); // 10 mb max

    Ok(router)
}