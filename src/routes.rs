use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::sync::Arc;
use actix_files::NamedFile;
use actix_web::{web, HttpResponse, HttpRequest, Responder};
use askama::Template;
use serde::Deserialize;

use crate::{
    config::Level, State
};
use crate::config::{Attachment, Config, Icon};

mod filters {
    use std::fmt::Display;

    use ammonia::Builder;
    use pulldown_cmark::{html, Parser};

    pub fn render<T>(src: T) -> askama::Result<String>
    where T: Display
    {
        let src = src.to_string();
        let p = Parser::new(&src);
        let mut as_html = String::new();
        html::write_html_fmt(&mut as_html, p)?;
        let b = Builder::default();
        Ok(b.clean(&as_html).to_string())
    }
}

#[derive(Template)]
#[template(path = "level.html")]
struct LevelPage {
    pub config: Arc<Config>,
    pub level: Level,
    pub next: Option<Level>,
    pub attachments: Vec<Attachment>
}

#[derive(Template)]
#[template(path = "wrong.html")]
struct Wrong {
    pub config: Arc<Config>
}

#[derive(Deserialize)]
pub struct To{
    #[serde(default="String::new")]
    pub answer: String,
}

impl Icon {
    pub fn svg(&self) -> &'static str {
        match self {
            Icon::File => include_str!("../media/file.svg"),
            Icon::Link => include_str!("../media/download.svg"),
            Icon::Download => include_str!("../media/download.svg"),
            Icon::Image => include_str!("../media/image.svg"),
            Icon::Media => include_str!("../media/media.svg"),
            Icon::Text => include_str!("../media/text.svg"),
            Icon::Archive => include_str!("../media/archive.svg")
        }
    }
    pub fn data_url(&self) -> String {
        format!("data:image/svg+xml;utf-8;base64,{}", STANDARD.encode(self.svg()))
    }
}

pub async fn show_level(data: web::Data<State>, path: web::Path<(String,)>, query: web::Query<To>) -> impl Responder {
    let (id,) = path.into_inner();
    let lev = match data.config.levels.0.get(&id) {
        None => return HttpResponse::NotFound().body("404 not found"),
        Some(l) => l,
    };
    if let Some(a) = &lev.key {
        if a != &query.answer {
            return HttpResponse::Forbidden().content_type("text/html").body(Wrong{
                config: Arc::clone(&data.config)
            }.render().expect("failed to render"));
        }
    }
    let next = lev.next
        .as_ref()
        .map(|n| data.config.levels.0
            .get(&n.to)
            .expect("Error: level not found")
            .to_owned());
    HttpResponse::Ok().content_type("text/html").body(LevelPage{
        level: lev.clone(),
        config: Arc::clone(&data.config),
        attachments: lev.attachments.clone(),
        next
    }.render().expect("failed to render"))
}

pub async fn show_attachment(req: HttpRequest, data: web::Data<State>, path: web::Path<(String,)>) -> impl Responder {
    let (file,) = path.into_inner();
    let src = match data.config.attachments.get(&file) {
        Some(s) => s,
        None => {
            return HttpResponse::NotFound().finish();
        }
    };
    if src.starts_with("http://") || src.starts_with("https://") {
        return HttpResponse::Found().insert_header(("Location", src.clone())).finish();
    }
    let f = NamedFile::open(src).unwrap();
    f.into_response(&req)
}
