use std::sync::Arc;
use askama::Template;
use serde::Deserialize;
use axum::{body::Body, extract::{Path, Query, State}, response::Response};
use mime_guess::from_path;
use tokio::io::AsyncReadExt;

use crate::{
    level::{Level, Next}, AppState
};
use crate::config::Config;

mod filters {
    use std::fmt::Display;

    use pulldown_cmark::{html, Parser};

    pub fn render<T>(src: T) -> askama::Result<String>
    where T: Display
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
    pub level: Arc<Level>
}

#[derive(Template)]
#[template(path = "wrong.html")]
struct Wrong {
    pub config: Arc<Config>
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFound {
    pub config: Arc<Config>
}

#[derive(Deserialize)]
pub struct To{
    #[serde(default="String::new")]
    pub answer: String,
}

pub async fn root(State(s): State<AppState>) -> Response<Body> {
    Response::builder()
        .status(302)
        .header("Location", format!("/l/{}", s.config.start))
        .body(().into())
        .unwrap()
}

pub async fn show_level(State(s): State<AppState>, Path((id,)): Path<(String,)>, Query(to): Query<To>) -> Response<String> {
    let lev = if let Some(l) = s.level_manager.get(&id) { l } else {
        return Response::builder()
            .status(404)
            .header("Content-Type", "text/html")
            .body(NotFound{
                    config: Arc::clone(&s.config)
                }
                .render()
                .expect("failed to render")
            )
            .unwrap();
    };
    if let Some(a) = &lev.key {
        if a != &to.answer {
            return Response::builder()
                .status(403)
                .header("Content-Type", "text/html")
                .body(Wrong{
                        config: Arc::clone(&s.config)
                    }
                    .render()
                    .expect("failed to render")
                )
                .unwrap();
        }
    }
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(LevelPage{
                level: lev.clone(),
                config: Arc::clone(&s.config),
            }
            .render()
            .expect("failed to render")
        )
        .unwrap()
}

pub async fn show_attachment(State(s): State<AppState>, Path((file,)): Path<(String,)>) -> Response<Body> {
    let src = if let Some(f) = s.config.attachments.get(&file) { f } else {
        return Response::builder()
            .status(404)
            .header("Content-Type", "text/html")
            .body(vec![].into())
            .unwrap();
    };
    if src.starts_with("http://") || src.starts_with("https://") {
        Response::builder().status(302).header("Location", src).body(().into()).unwrap()
    } else {
        let mut f = tokio::fs::File::open(src).await.unwrap();
        let mut buf = vec![];
        f.read(&mut buf).await.unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", from_path(src).first_or_octet_stream().to_string())
            .body(buf.into())
            .unwrap()
    }
}
