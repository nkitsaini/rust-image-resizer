mod resizer;
mod api;

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = crate::api::init_router().unwrap();

    Ok(router.into())
}