mod resizer;
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use resizer::ImageResizeParams;


use tokio::task;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[axum::debug_handler]
async fn image_resizer(params: Query<ImageResizeParams>, body: Bytes) -> axum::response::Response {
    let mut r: Option<axum::response::Response> = None;
    let params = params.0;
    match task::spawn(async { resizer::reszier(body, params) }).await {
        Ok(x) => match x {
            Ok(x) => r = Some((StatusCode::OK, x).into_response()),
            Err(e) => {
                r = Some(
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e:?}")).into_response(),
                )
            }
        },
        Err(_e) => {
            r = Some(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Can't start a new thread",
                )
                    .into_response(),
            )
        }
    }
    r.unwrap()
}

#[axum::debug_handler]
async fn image_resizer_test(_params: Query<ImageResizeParams>, body: Bytes) -> axum::response::Response {
    format!("Hey, {}", body.len()).into_response()
}

fn init_router() -> anyhow::Result<Router> {
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/resize", post(image_resizer))
        .route("/test", post(image_resizer_test))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10)); // 10 mb max

    Ok(router)
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = init_router().unwrap();

    Ok(router.into())
}