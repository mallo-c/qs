use actix_web::{web, HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;

use crate::{
    config::{Level, Link, Strings}, State
};

mod filters {
    use std::fmt::Display;

    pub fn nl<T>(src: T) -> ::askama::Result<String>
    where T: Display
    {
        Ok(src
            .to_string()
            .trim()
            .split("\n")
            .filter(|s| !s.is_empty())
            .map(|s| format!("<p>{s}</p>"))
            .collect()
        )
    }
}

#[derive(Template)]
#[template(path = "level.html")]
struct LevelPage {
    pub strings: Strings,
    pub links: Vec<Link>,
    pub level: Level,
    pub next: Option<Level>
}

#[derive(Template)]
#[template(path = "wrong.html")]
struct Wrong {
    pub strings: Strings,
    pub links: Vec<Link>,
}

#[derive(Deserialize)]
pub struct To{
    #[serde(default="String::new")]
    pub answer: String,
    #[serde(default="String::new")]
    pub from: String,
}

pub async fn show_level(data: web::Data<State>, path: web::Path<(String,)>, query: web::Query<To>) -> impl Responder {
    let (id,) = path.into_inner();
    let lev = data.config.levels.0.get(&id);
    if let None = lev {
        return HttpResponse::NotFound().body("404 not found");
    }
    let lev = lev.unwrap();
    if let Some(a) = &lev.key {
        if a != &query.answer {
            return HttpResponse::Forbidden().content_type("text/html").body(Wrong{
                strings: data.config.strings.clone(),
                links: data.config.links.clone()
            }.render().unwrap());
        }
    }
    let next: Option<Level>;
    if let Some(n) = &lev.next {
        next = Some(data.config.levels.0.get(&n.to).expect("Error: level not found").to_owned());
    } else {
        next = None;
    }
    HttpResponse::Ok().content_type("text/html").body(LevelPage{
        strings: data.config.strings.clone(),
        level: lev.clone(),
        links: data.config.links.clone(),
        next: next
    }.render().unwrap())
}
