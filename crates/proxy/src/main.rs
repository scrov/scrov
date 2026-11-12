//! Self-hosted entry point: Pingora in front of the scrov server.
//!
//! By default the site itself runs inside this process on a loopback port and
//! Pingora listens on the public address, adding security headers on the way
//! out. Set `SCROV_UPSTREAM=host:port` to proxy to a server running elsewhere.
//!
//!   SCROV_LISTEN      public address      (default 0.0.0.0:8080)
//!   SCROV_UPSTREAM    existing upstream   (default: start one in-process)
//!   SCROV_PUBLIC_DIR  static files        (default ./public)
//!
//! Pingora runs on Linux and macOS only, and never on Cloudflare Workers; the
//! Worker build (crates/worker) does not include this crate.

use async_trait::async_trait;
use pingora::{
    http::ResponseHeader,
    prelude::{http_proxy_service, HttpPeer, ProxyHttp, Result, Server, Session},
};
use std::net::SocketAddr;

struct Gateway {
    upstream: SocketAddr,
}

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        Ok(Box::new(HttpPeer::new(self.upstream, false, String::new())))
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        response.insert_header("X-Content-Type-Options", "nosniff")?;
        response.insert_header("Referrer-Policy", "strict-origin-when-cross-origin")?;
        response.insert_header("X-Frame-Options", "SAMEORIGIN")?;
        response.insert_header("Server", "scrov (pingora)")?;
        Ok(())
    }
}

/// Runs the site on a loopback port in a background thread and returns its address.
fn start_site() -> std::io::Result<SocketAddr> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    let public = scrov_server::public_dir();

    std::thread::Builder::new()
        .name("scrov-site".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            runtime.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(listener).expect("listener");
                if let Err(error) = scrov_server::serve(listener, public).await {
                    eprintln!("scrov site stopped: {error}");
                }
            });
        })?;
    Ok(addr)
}

fn main() {
    let listen = std::env::var("SCROV_LISTEN").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let upstream = match std::env::var("SCROV_UPSTREAM") {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| panic!("SCROV_UPSTREAM must be host:port, got {value:?}")),
        Err(_) => start_site().expect("could not start the site"),
    };

    let mut server = Server::new(None).expect("pingora server");
    server.bootstrap();
    let mut proxy = http_proxy_service(&server.configuration, Gateway { upstream });
    proxy.add_tcp(&listen);
    server.add_service(proxy);

    println!("scrov proxy on http://{listen} → http://{upstream}");
    server.run_forever();
}
