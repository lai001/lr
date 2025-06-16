use std::net::{Ipv4Addr, SocketAddrV4};

pub static DEFAULT_SERVER_ADDR: std::net::SocketAddr =
    std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8700));

#[derive(Clone)]
pub struct MultiplePlayerOptions {
    pub server_socket_addr: std::net::SocketAddr,
    pub is_server: bool,
    pub players: u32,
}

impl Default for MultiplePlayerOptions {
    fn default() -> Self {
        Self {
            server_socket_addr: DEFAULT_SERVER_ADDR,
            is_server: true,
            players: 2,
        }
    }
}

#[derive(Clone)]
pub enum StandaloneSimulationType {
    Single,
    MultiplePlayer(MultiplePlayerOptions),
}
