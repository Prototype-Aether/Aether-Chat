use aether_lib::peer::Aether;
use std::io::stdin;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn main() {
    println!("Enter username: ");
    let mut username = String::new();
    stdin().read_line(&mut username).unwrap();

    username = String::from(username.trim());

    let tracker_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(149, 129, 129, 226)), 8982);
    let aether = Aether::new(username, tracker_addr);

    println!("Enter peer username: ");
    let mut peer_username = String::new();
    stdin().read_line(&mut peer_username).unwrap();

    peer_username = String::from(peer_username.trim());

    aether.start();

    aether.connect(peer_username.clone());

    aether.wait_connection(&peer_username).unwrap();

    loop {
        let mut message = String::new();
        stdin().read_line(&mut message).unwrap();
        aether
            .send_to(&peer_username, message.into_bytes())
            .unwrap();
        let recved = aether.recv_from(&peer_username).unwrap();
        println!("{}: {}", peer_username, String::from_utf8(recved).unwrap());
    }
}
