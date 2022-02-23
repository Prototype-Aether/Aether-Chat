use aether_lib::peer::Aether;
use crossbeam::thread;
use log::info;
use std::io::stdin;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let tracker_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(149, 129, 129, 226)), 8982);
    let aether = Arc::new(Aether::new(tracker_addr));

    println!("Your id:\n{}", aether.get_uid());

    println!("Enter peer username: ");
    let mut peer_username = String::new();
    stdin().read_line(&mut peer_username).unwrap();

    peer_username = String::from(peer_username.trim());

    aether.start();

    aether.connect(&peer_username);

    info!("waiting to connect...");
    aether.wait_connection(&peer_username).unwrap();

    info!("connected!");

    thread::scope(|s| {
        let handle_recv = s.spawn(|_| loop {
            let recved = aether.recv_from(&peer_username).unwrap();
            println!("other: {}", String::from_utf8(recved).unwrap());
        });
        let handle_send = s.spawn(|_| loop {
            let mut message = String::new();
            stdin().read_line(&mut message).unwrap();
            aether
                .send_to(&peer_username, message.into_bytes())
                .unwrap();
        });
        handle_recv.join().unwrap();
        handle_send.join().unwrap();
    })
    .unwrap();
}
