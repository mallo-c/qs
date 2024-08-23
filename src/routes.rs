use askama::Template;
use axum::{
    body::Body, extract::{Path, Query, State}, http::StatusCode, response::{IntoResponse, Response}
};
use mime_guess::from_path;
use serde::Deserialize;
use std::sync::Arc;
use tokio_util::io::ReaderStream;

use crate::config::Config;
use crate::{
    level::{Level, Next},
    AppState,
};

mod filters {
    use std::fmt::Display;

    use pulldown_cmark::{html, Parser};

    pub fn render<T>(src: T) -> askama::Result<String>
    where
        T: Display,
    {
        let src = src.to_string();
        let p = Parser::new(&src);
        let mut as_html = String::new();
        html::write_html_fmt(&mut as_html, p)?;
        Ok(as_html)
    }
}

#[derive(Template)]
#[template(path = "level.html")]
struct LevelPage {
    pub config: Arc<Config>,
    pub level: Arc<Level>,
}

#[derive(Template)]
#[template(path = "wrong.html")]
struct Wrong {
    pub config: Arc<Config>,
    pub msg: String
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFound {
    pub config: Arc<Config>,
}

#[derive(Deserialize)]
pub struct To {
    #[serde(default = "String::new")]
    pub answer: String,
}

pub async fn root(State(s): State<AppState>) -> Response<Body> {
    Response::builder()
        .status(302)
        .header("Location", format!("/l/{}", s.config.start))
        .body(().into())
        .unwrap()
}

pub async fn show_level(
    State(s): State<AppState>,
    Path((id,)): Path<(String,)>,
    Query(to): Query<To>,
) -> Response<String> {
    let lev = if let Some(l) = s.level_manager.get(&id) {
        l
    } else {
        return Response::builder()
            .status(404)
            .header("Content-Type", "text/html")
            .body(
                NotFound {
                    config: Arc::clone(&s.config),
                }
                .render()
                .expect("failed to render"),
            )
            .unwrap();
    };
    if let Err(msg) = lev.key.check(&to.answer) {
        return Response::builder()
            .status(403)
            .header("Content-Type", "text/html")
            .body(
                Wrong {
                    config: Arc::clone(&s.config),
                    msg
                }
                .render()
                .expect("failed to render"),
            )
            .unwrap();
    }
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(
            LevelPage {
                level: lev.clone(),
                config: Arc::clone(&s.config),
            }
            .render()
            .expect("failed to render"),
        )
        .unwrap()
}

pub async fn show_attachment(
    State(s): State<AppState>,
    Path((file,)): Path<(String,)>,
) -> Response {
    let src = if let Some(f) = s.config.attachments.get(&file) {
        f
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let f = tokio::fs::File::open(src).await.unwrap();
    let stream = ReaderStream::new(f);
    let b = Body::from_stream(stream);
    Response::builder()
        .status(200)
        .header(
            "Content-Type",
            from_path(src).first_or_octet_stream().to_string(),
        )
        .body(b)
        .unwrap()
}
