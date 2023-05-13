mod resizer;
mod overflow_ops;
mod query_extractor;
mod api;

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = crate::api::init_router().unwrap();

    Ok(router.into())
}