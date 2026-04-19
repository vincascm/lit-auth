mod api;
mod config;
mod db;
mod error;
mod statics;

use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    statics::init().await?;

    let cfg = statics::cfg().await?;
    let addr: SocketAddr = cfg.listen_addr.parse()?;
    println!("listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, api::router()).await?;

    Ok(())
}
