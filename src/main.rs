mod api;
mod domain;
mod runtime;
mod store;
use clap::Parser;
use rand::RngCore;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

#[derive(Parser)]
struct Args {
    #[arg(long, env = "ANTIDETECT_CHROMIUM")]
    chromium: PathBuf,
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,
    #[arg(long, default_value_t = 0)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let token = bytes.iter().map(|v| format!("{v:02x}")).collect::<String>();
    let state = Arc::new(api::AppState {
        store: store::ProfileStore::new(args.data_dir.join("profiles"))?,
        runtime: runtime::ChromiumRuntime::new(args.chromium, args.data_dir.join("runtime"))?,
        token: token.clone(),
    });
    let listener =
        tokio::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), args.port))
            .await?;
    let address = listener.local_addr()?;
    println!("Control panel: http://{address}/#{token}");
    axum::serve(listener, api::router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
