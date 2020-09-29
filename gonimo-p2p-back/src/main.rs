use libp2p::{
    core::PeerId,
    identity::{ed25519, PublicKey},
};
use libp2p_mdns::service::{
    build_query_response, build_service_discovery_response, MdnsPacket, MdnsService,
};

use tokio::{prelude::*, stream::Stream, stream::StreamExt};

use parity_multiaddr::{Multiaddr, Protocol};
use pnet::datalink;
use std::io::Error;
use std::net::IpAddr;
use std::time::Duration;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let keypair = ed25519::Keypair::generate();
    let my_peer_id = PeerId::from_public_key(PublicKey::Ed25519(keypair.public()));
    println!("Our peer id: {:?}", my_peer_id);
    let service = MdnsService::new().expect("Another mdns service already running?");
    async {
        println!("Before asking next");
        let mut next = service.next().await;
        println!("Before loop");
        loop {
            let (mut service, packet) = next;
            match packet {
                MdnsPacket::Query(query) => {
                    // Addresses might change over time - so re-query them everytime.
                    let addrs = get_peer_addrs();
                    println!("Query from {:?}", query.remote_addr());
                    let resp = build_query_response(
                        query.query_id(),
                        my_peer_id.clone(),
                        addrs.into_iter(),
                        Duration::from_secs(120),
                    )
                    .unwrap();
                    service.enqueue_response(resp);
                }
                MdnsPacket::Response(response) => {
                    for peer in response.discovered_peers() {
                        println!("Discovered peer {:?}", peer.id());
                        for addr in peer.addresses() {
                            println!("Address = {:?}", addr);
                        }
                    }
                }
                MdnsPacket::ServiceDiscovery(disc) => {
                    println!("Service discovery");
                    let resp =
                        build_service_discovery_response(disc.query_id(), Duration::from_secs(120));
                    service.enqueue_response(resp);
                }
            }
            next = service.next().await;
        }
    }
    .await;
    Ok(())
}

/// Get all available addresses of our endpoint.
fn get_peer_addrs() -> Vec<Multiaddr> {
    let raw_addrs = datalink::interfaces()
        .into_iter()
        .filter(|i| !i.is_loopback())
        .flat_map(|i| i.ips);
    raw_addrs.map(|a| multi_addr_from_ip(a.ip())).collect()
}

/// Convert an `IpAddr` to a `MultiAddr` with a transport suitable for Gonimo signalling.
fn multi_addr_from_ip(addr: IpAddr) -> Multiaddr {
    Multiaddr::empty()
        .with(Protocol::from(addr))
        .with(Protocol::Udp(43286))
}
