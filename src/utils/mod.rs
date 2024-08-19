use alloc::string::{String, ToString};
use anyhow::Result;
use url::Url;

pub fn http_to_ws(uri: &Url) -> Url {
    let mut ws_uri = uri.clone();
    ws_uri
        .set_scheme(match uri.scheme() {
            "https" => "wss",
            "http" => "ws",
            _ => uri.scheme(),
        })
        .unwrap();
    ws_uri
}

pub fn ws_to_http(uri: &Url) -> Url {
    let mut http_uri = uri.clone();
    http_uri
        .set_scheme(match uri.scheme() {
            "wss" => "https",
            "ws" => "http",
            _ => uri.scheme(),
        })
        .unwrap();
    http_uri
}

pub fn derive_tcp_url(url: &Url, default: Option<u16>) -> Result<String> {
    let host = match url.host_str() {
        Some(host) => host,
        None => return Err(anyhow::anyhow!("Host not found")),
    };
    let port = match url.port() {
        Some(port) => port,
        None => match default {
            Some(port) => port,
            None => match url.scheme() {
                "https" | "wss" => 443,
                "http" | "ws" => 80,
                _ => 80,
            },
        },
    }
    .to_string();
    // let path = url.path().to_string();
    let url = host.to_string() + ":" + &port; // + &path;

    Ok(url)
}
