use aether_lib::peer::Aether;
use crossbeam::thread;
use hyper::{Body, Client, Method, Request};
use log::info;
use serde::{Deserialize, Serialize};
use std::io::stdin;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use simple_logger::SimpleLogger;

const NAME_SERVER: &str = "http://149.129.129.226:5000";

#[derive(Serialize, Deserialize)]
struct UsernameResponse {
    username: String,
}

#[derive(Serialize, Deserialize)]
struct PublicKeyResponse {
    publickey: String,
}

async fn get_my_username(
    public_key: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let uri = NAME_SERVER;

    let body = format!("{{\"publickey\": \"{}\"}}", public_key);

    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body))?;

    let response = client.request(req).await?;

    let response_bytes = hyper::body::to_bytes(response.into_body()).await?;

    let json_string = String::from_utf8(response_bytes[..].to_vec()).unwrap();

    let response_struct: UsernameResponse = serde_json::from_str(&json_string).unwrap();

    Ok(response_struct.username)
}

async fn get_public_key(
    username: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let uri = format!("{}/{}", NAME_SERVER, username).parse()?;

    let response = client.get(uri).await?;

    let response_bytes = hyper::body::to_bytes(response.into_body()).await?;

    let json_string = String::from_utf8(response_bytes[..].to_vec()).unwrap();

    let response_struct: PublicKeyResponse = serde_json::from_str(&json_string).unwrap();

    Ok(response_struct.publickey)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .init()
        .unwrap();
    let tracker_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(149, 129, 129, 226)), 8982);
    let aether = Arc::new(Aether::new(tracker_addr));

    let user_alias = get_my_username(aether.get_uid()).await?;

    println!("Your username: {}", user_alias);

    print!("Enter peer username: ");
    let mut peer_username = String::new();
    stdin().read_line(&mut peer_username).unwrap();

    peer_username = String::from(peer_username.trim());

    let peer_uid = get_public_key(&peer_username).await?;

    aether.start();

    aether.connect(&peer_uid);

    info!("Waiting to connect...");
    aether.wait_connection(&peer_uid).unwrap();

    info!("Connected!");

    thread::scope(|s| {
        let handle_recv = s.spawn(|_| loop {
            let recved = aether.recv_from(&peer_uid).unwrap();
            println!("other: {}", String::from_utf8(recved).unwrap());
        });
        let handle_send = s.spawn(|_| loop {
            let mut message = String::new();
            stdin().read_line(&mut message).unwrap();
            aether.send_to(&peer_uid, message.into_bytes()).unwrap();
        });
        handle_recv.join().unwrap();
        handle_send.join().unwrap();
    })
    .unwrap();

    Ok(())
}
