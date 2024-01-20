use reqwest;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;
use structs::requests::UserStatus;

const SERVER_URL: &'static str = "http://localhost:3000";

#[derive(Debug)]
pub(crate) enum ServerRequestError {
    ReqwestError(reqwest::Error),
    InvalidDataError(serde_json::Error),
    InvalidResponseError(serde_json::Error),
    Status(StatusCode, Option<String>),
}

impl Display for ServerRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerRequestError::ReqwestError(err) => err.fmt(f),
            ServerRequestError::InvalidDataError(err) => err.fmt(f),
            ServerRequestError::InvalidResponseError(err) => err.fmt(f),
            ServerRequestError::Status(status, msg) => {
                write!(
                    f,
                    "Status Code {:#?}: {}",
                    status,
                    msg.as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("Неизвестная ошибка.")
                )
            }
        }
    }
}

pub(crate) async fn server_post<T: DeserializeOwned>(
    client: reqwest::Client,
    route: &'static str,
    data: impl Serialize,
    session: Option<String>,
) -> Result<T, ServerRequestError> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    if let Some(session) = session {
        headers.insert(
            "Authorization",
            format!("Bearer {session}").parse().unwrap(),
        );
    }

    let response = client
        .post(&format!("{SERVER_URL}/{route}"))
        .headers(headers)
        .body(
            serde_json::to_value(data)
                .map_err(ServerRequestError::InvalidResponseError)?
                .to_string(),
        )
        .send()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response = match response.status() {
        StatusCode::OK => Ok(response),
        status => Err(ServerRequestError::Status(
            status,
            response.text().await.ok(),
        )),
    }?;
    let response_data = response
        .text()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_value =
        serde_json::from_str(&response_data).map_err(ServerRequestError::InvalidDataError)?;
    Ok(response_value)
}

pub(crate) async fn server_get<T: DeserializeOwned>(
    client: reqwest::Client,
    route: String,
    session: Option<String>,
) -> Result<T, ServerRequestError> {
    let mut headers = HeaderMap::new();

    if let Some(session) = session {
        headers.insert(
            "Authorization",
            format!("Bearer {session}").parse().unwrap(),
        );
    }

    let response = client
        .get(&format!("{SERVER_URL}/{route}"))
        .headers(headers)
        .send()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response = match response.status() {
        StatusCode::OK => Ok(response),
        status => Err(ServerRequestError::Status(
            status,
            response.text().await.ok(),
        )),
    }?;
    let response_data = response
        .text()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_value =
        serde_json::from_str(&response_data).map_err(ServerRequestError::InvalidDataError)?;
    Ok(response_value)
}

pub async fn get_profile_picture(client: reqwest::Client, user: String) -> Option<String> {
    server_get::<UserStatus>(client, format!("status/{user}"), None)
        .await
        .ok()
        .and_then(|status| status.profile_picture)
}
