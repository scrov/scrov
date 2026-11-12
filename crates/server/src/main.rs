use std::net::SocketAddr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr: SocketAddr = std::env::var("SCROV_ADDR")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));
    let public = scrov_server::public_dir();
    if !public.join("pkg/app.css").exists() {
        eprintln!(
            "warning: {} has no pkg/app.css; run `bun run build:web` first",
            public.display()
        );
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("scrov listening on http://{}", listener.local_addr()?);
    scrov_server::serve(listener, public).await
}
