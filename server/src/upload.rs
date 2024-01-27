use std::path::Path;

use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    routing::post,
    Router,
};

use tokio::{fs::File, io::AsyncWriteExt};
use tower_http::services::ServeDir;

use crate::AppState;

async fn upload_picture(mut multipart: Multipart) -> Result<String, (StatusCode, String)> {
    let no_file_field = Err((StatusCode::BAD_REQUEST, "no 'file' form field".into()));

    let Ok(Some(field)) = multipart.next_field().await else {
        return no_file_field;
    };

    let Some("file") = field.name() else {
        return no_file_field;
    };

    let bytes = field.bytes().await.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "file too big, max size is 25mb".into(),
        )
    })?;
    let file_id = uuid::Uuid::new_v4().to_string();
    let path_str = format!("content/img-{}", &file_id);
    let path = Path::new(&path_str);
    let mut file = File::create(path).await.unwrap();
    file.write_all(&bytes).await.unwrap();
    Ok(file_id)
}

pub(crate) fn router() -> Router<AppState> {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const FILE_SIZE_LIMIT: usize = 25 * MB;

    Router::new()
        .route("/upload_picture", post(upload_picture))
        .nest_service("/content", ServeDir::new("content"))
        .layer(DefaultBodyLimit::max(FILE_SIZE_LIMIT))
}
