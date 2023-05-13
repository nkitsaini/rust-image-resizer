//! Implements a query extracts that can work with untagged enums in serde
//! uses an intermediatary  json represents to bypass the issue with default Query provider
//! temporary solves: https://github.com/nox/serde_urlencoded/issues/66
//! 
use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}, response::IntoResponse};
use serde::de::DeserializeOwned;


#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct QueryProxyExtrator<T> (pub T);

fn url_query_to_serde_value(query: &str) -> serde_json::Map<String, serde_json::Value> {
	let mut rv = serde_json::Map::new();
	for (key, value) in url_encoded_data::UrlEncodedData::parse_str(query).as_pairs() {
		rv.insert(key.to_string(), value.to_string().into());
	}
	rv
}
pub struct QueryCustomRejection(serde_json::Error);
impl IntoResponse for QueryCustomRejection {
	fn into_response(self) -> axum::response::Response {
		(StatusCode::BAD_REQUEST, format!("{:?}", self.0)).into_response()
	}
}

#[async_trait]
impl<T, S> FromRequestParts<S> for QueryProxyExtrator<T>
where
	T: DeserializeOwned,
	S: Send+Sync
{
	type Rejection = QueryCustomRejection;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		let query = parts.uri.query().unwrap_or_default();
		dbg!(query);
		let query_value = url_query_to_serde_value(query);
		dbg!(&query_value);
		let r: T = serde_json::from_value(query_value.into()).map_err(|x| QueryCustomRejection(x))?;
		return Ok(Self(r))
	}
}