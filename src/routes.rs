use std::sync::Arc;
use actix_files::NamedFile;
use actix_web::{web, HttpResponse, HttpRequest, Responder};
use askama::Template;
use serde::Deserialize;

use crate::{
    level::{Level, Next}, State
};
use crate::config::Config;

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

pub async fn show_level(data: web::Data<State>, path: web::Path<(String,)>, query: web::Query<To>) -> impl Responder {
    let (id,) = path.into_inner();
    let lev = match data.level_manager.get(&id) {
        None => return HttpResponse::NotFound().body(NotFound{
            config: Arc::clone(&data.config)
        }.render().expect("failed to render")),
        Some(l) => l,
    };
    if let Some(a) = &lev.key {
        if a != &query.answer {
            return HttpResponse::Forbidden().content_type("text/html").body(Wrong{
                config: Arc::clone(&data.config)
            }.render().expect("failed to render"));
        }
    }
    HttpResponse::Ok().content_type("text/html").body(LevelPage{
        level: lev.clone(),
        config: Arc::clone(&data.config),
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
