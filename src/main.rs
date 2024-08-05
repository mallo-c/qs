mod config;
mod level;
mod routes;

use std::error::Error;
use std::net::SocketAddrV4;
use std::sync::Arc;

use crate::config::Config;
use argh::{from_env, FromArgs};
use axum::{routing::get, Router};
use level::LevelManager;
use routes::{root, show_attachment, show_level};

#[derive(FromArgs)]
/// qs is a quest engine
struct Args {
    #[argh(option, from_str_fn(__argh_from_str_fn_config))]
    /// config file
    config: Config,
    #[argh(option, short = 'p', default = "8080")]
    /// port to listen to (default 8080)
    port: u16,
}

fn __argh_from_str_fn_config(file: &str) -> Result<Config, String> {
    Config::from_path(file).map_err(|e| e.to_string())
}

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    level_manager: Arc<LevelManager>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = from_env();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let config = Arc::new(args.config);
    let d = AppState {
        config: Arc::clone(&config),
        level_manager: Arc::new(LevelManager::from_config(&config)?),
    };
    let r = Router::new()
        .route("/", get(root))
        .route("/l/:lev", get(show_level))
        .route("/a/:file", get(show_attachment))
        .with_state(d);
    let l = tokio::net::TcpListener::bind(SocketAddrV4::new("0.0.0.0".parse().unwrap(), args.port))
        .await?;
    axum::serve(l, r.into_make_service()).await?;
    Ok(())
}
