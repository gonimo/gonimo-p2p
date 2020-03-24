use libp2p_mdns:: {
    service::{
        MdnsService,
        MdnsPacket,
        build_query_response,
        build_service_discovery_response
    }
};
use libp2p::{
    identity::{ed25519, PublicKey},
    core::{
        PeerId
    }
};

use tokio:: {
    prelude::*,
    stream::Stream,
    stream::StreamExt,
};

use std::io::Error;
use std::time::{Duration};
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
                    println!("Query from {:?}", query.remote_addr());
                    let resp = build_query_response(
                        query.query_id(),
                        my_peer_id.clone(),
                        vec![].into_iter(),
                        Duration::from_secs(120),
                    ).unwrap();
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
                    let resp = build_service_discovery_response(
                        disc.query_id(),
                        Duration::from_secs(120),
                    );
                    service.enqueue_response(resp);
                }
            }
            next = service.next().await;
        }
	}.await;
	Ok(())
}
