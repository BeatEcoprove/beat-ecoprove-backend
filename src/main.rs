use std::env;

use beat_ecoprove_proxy::proxy::BeatProxy;
use env_logger::Env;
use pingora::server::Server;
use pingora_proxy::http_proxy_service;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("PROXY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PROXY_PORT").unwrap_or_else(|_| "9000".to_string());
    let addr = format!("{}:{}", host, port);

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .init();

    let mut server = Server::new(None)?;
    server.bootstrap();

    let proxy = BeatProxy::new()?;
    let mut proxy_service = http_proxy_service(&server.configuration, proxy);
    proxy_service.add_tcp(&addr);

    log::info!("{}", format!("Proxy listening on {}", addr));

    server.add_service(proxy_service);
    server.run_forever();
}
