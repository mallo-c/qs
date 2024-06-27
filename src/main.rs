mod routes;
mod config;

use std::error::Error;
use std::net::SocketAddrV4;
use std::sync::Arc;

use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer};
use argh::{FromArgs, from_env};
use routes::{show_attachment, show_level};
use crate::config::Config;

#[derive(FromArgs)]
/// qs is a quest engine
struct Args {
    #[argh(option, from_str_fn(__argh_from_str_fn_config))]
    /// config file
    config: Config,
    #[argh(option, short='p', default="8080")]
    /// port to listen to (default 8080)
    port: u16,
}

fn __argh_from_str_fn_config(file: &str) -> Result<Config, String> {
    match Config::from_file(file) {
        Ok(c) => Ok(c),
        Err(s) => Err(s.to_string())
    }
}

struct State {
    config: Arc<Config>,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = from_env();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let config = Arc::new(args.config);
    let d = Data::new(State{
        config: Arc::clone(&config),
    });
    HttpServer::new(move ||{
        App::new()
            .app_data(d.clone())
            .route("/", web::get().to(|| async {HttpResponse::Found()
                                                    .append_header(("Location", "/level/start"))
                                                    .body("")}))
            .route("/level/{lev}", web::get().to(show_level))
            .route("/file/{file}", web::get().to(show_attachment))
    })
        .bind(SocketAddrV4::new("0.0.0.0".parse()?, args.port))?
        .run()
        .await
        .map_err(|error| Box::new(error) as Box<dyn Error>)
}
