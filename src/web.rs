use std::{net::Ipv4Addr, path::PathBuf};

use rocket::{
    fairing::AdHoc,
    get,
    http::ContentType,
    routes,
    serde::{Serialize, json::Json},
    Build, Rocket,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/build/"]
struct Assets;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Health {
    status: &'static str,
    version: &'static str,
}

#[get("/api/health")]
fn health() -> Json<Health> {
    Json(Health { status: "ok", version: env!("CARGO_PKG_VERSION") })
}

fn asset(path: &str) -> Option<(ContentType, Vec<u8>)> {
    let file = Assets::get(path)?;
    let extension = path.rsplit('.').next().unwrap_or("");
    let content_type = ContentType::from_extension(extension).unwrap_or(ContentType::Binary);
    Some((content_type, file.data.into_owned()))
}

#[get("/")]
fn index() -> Option<(ContentType, Vec<u8>)> {
    asset("index.html")
}

#[get("/<path..>", rank = 100)]
fn files(path: PathBuf) -> Option<(ContentType, Vec<u8>)> {
    let path = path.to_str()?;
    asset(path).or_else(|| asset(&format!("{path}.html"))).or_else(|| asset(&format!("{path}/index.html")))
}

fn app(files: crate::network::RunFiles, port: u16) -> Result<Rocket<Build>, String> {
    let simulation = crate::server::Server::launch(files)?;
    let stop_simulation = simulation.clone();
    let config = rocket::Config::figment()
        .merge(("address", Ipv4Addr::LOCALHOST))
        .merge(("port", port))
        .merge(("log_level", "off"));
    Ok(rocket::custom(config)
        .manage(simulation)
        .mount("/", routes![health, index, files, crate::server::snapshot, crate::server::command, crate::server::websocket])
        .attach(AdHoc::on_shutdown("Stop simulation", move |_| Box::pin(async move {
            stop_simulation.stop();
        })))
        .attach(AdHoc::on_liftoff("Vanguard ready", move |rocket| {
            Box::pin(async move {
                println!(r"
 __     __ _    _   _  ____ _   _    _    ____  ____
 \ \   / // \  | \ | |/ ___| | | |  / \  |  _ \|  _ \
  \ \ / // _ \ |  \| | |  _| | | | / _ \ | |_) | | | |
   \ V // ___ \| |\  | |_| | |_| |/ ___ \|  _ <| |_| |
    \_//_/   \_\_| \_|\____|\___//_/   \_\_| \_\____/

  Web UI: http://{}:{}
  Open the dashboard to start. Press Ctrl-C to stop.
", rocket.config().address, rocket.config().port);
            })
        })))
}

pub async fn serve(files: crate::network::RunFiles, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    app(files, port).map_err(std::io::Error::other)?.launch().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::{http::Status, local::asynchronous::Client};

    #[rocket::async_test]
    async fn serves_embedded_frontend_assets_and_health() {
        let files = crate::network::RunFiles { script: "vanguard/agents/ostrich.lua".into(), network: "network.toml".into(), dbg: false };
        let client = Client::tracked(app(files, 0).unwrap()).await.unwrap();
        let response = client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
        assert!(response.into_string().await.unwrap().contains("Vanguard"));

        let response = client.get("/api/health").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let json = response.into_json::<rocket::serde::json::Value>().await.unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));

        let js = Assets::iter().find(|path| path.ends_with(".js")).unwrap();
        let response = client.get(format!("/{js}")).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JavaScript));
        assert_eq!(client.get("/missing.js").dispatch().await.status(), Status::NotFound);
        assert_eq!(client.get("/api/missing").dispatch().await.status(), Status::NotFound);
    }
}
