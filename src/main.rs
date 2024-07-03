mod routes;
mod config;
mod level;

use std::error::Error;
use std::net::SocketAddrV4;
use std::sync::Arc;

use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer};
use argh::{FromArgs, from_env};
use level::LevelManager;
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
    Config::from_file(file).map_err(|e| e.to_string())
}

struct State {
    config: Arc<Config>,
    level_manager: LevelManager
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = from_env();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let config = Arc::new(args.config);
    let d = Data::new(State{
        config: Arc::clone(&config),
        level_manager: LevelManager::from_config(&config).unwrap()
    });
    HttpServer::new(move ||{
        App::new()
            .app_data(d.clone())
            .route("/", web::get().to(|data: web::Data<State>| async move {
                                                    HttpResponse::Found()
                                                        .append_header(("Location", format!("/l/{}", data.config.start)))
                                                        .body("")}))
            .route("/l/{lev}", web::get().to(show_level))
            .route("/a/{file}", web::get().to(show_attachment))
    })
        .bind(SocketAddrV4::new("0.0.0.0".parse()?, args.port))?
        .run()
        .await
        .map_err(|error| Box::new(error) as Box<dyn Error>)
}
