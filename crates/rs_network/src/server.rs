use crate::client::Client;
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct Connection {
    pub peer_addr: SocketAddr,
    pub local_addr: SocketAddr,
}

pub struct Server {
    clients: Vec<Client>,
    recciver: std::sync::mpsc::Receiver<TcpStream>,
    shutdown: Arc<AtomicBool>,
    addr: SocketAddr,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shutdown_internal();
    }
}

impl Server {
    pub fn bind(addr: SocketAddr) -> crate::error::Result<Server> {
        let (sender, recciver) = std::sync::mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let listener = TcpListener::bind(addr).map_err(|err| {
            crate::error::Error::IO(err, Some(format!("Failed to bind to: {}", addr)))
        })?;
        let _ = std::thread::Builder::new()
            .name(format!("Network"))
            .spawn({
                let sender = sender.clone();
                let shutdown = shutdown.clone();
                move || {
                    for stream in listener.incoming() {
                        if shutdown.load(Ordering::Relaxed) {
                            break;
                        }
                        match stream {
                            Ok(stream) => {
                                let _ = sender.send(stream);
                            }
                            Err(err) => {
                                log::warn!("Connection failed: {}", err);
                            }
                        }
                    }
                    log::trace!("Shutdown server: {}", addr);
                }
            })
            .map_err(|err| crate::error::Error::IO(err, None))?;

        Ok(Server {
            clients: Vec::new(),
            recciver,
            shutdown: shutdown,
            addr,
        })
    }

    pub fn process_incoming(&mut self) -> Vec<Connection> {
        let mut connections = vec![];
        for stream in self.recciver.try_iter() {
            log::trace!("New stream: {:?}", stream.peer_addr(),);
            match stream.set_read_timeout(Some(Duration::from_millis(1))) {
                Ok(_) => {}
                Err(err) => {
                    log::warn!("{err}")
                }
            }
            match Client::from_stream(stream, Some("Server".to_string())) {
                Ok(client) => {
                    connections.push(Connection {
                        peer_addr: client.peer_addr,
                        local_addr: client.local_addr,
                    });
                    self.clients.push(client);
                }
                Err(err) => {
                    log::warn!("{}", err);
                }
            }
        }
        connections
    }

    pub fn broadcast(&mut self, data: &[u8]) {
        for client in &mut self.clients {
            client.write(data.to_vec());
        }
    }

    pub fn shutdown_all_streams(&mut self) {
        self.clients.clear();
    }

    pub fn shutdown(mut self) {
        self.shutdown_internal();
    }

    fn shutdown_internal(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.shutdown_all_streams();
        let timeout = std::time::Duration::from_secs_f32(3.0);
        if let Err(err) = TcpStream::connect_timeout(&self.addr, timeout) {
            log::warn!("{}", err);
        }
    }

    pub fn clients_mut(&mut self) -> &mut Vec<Client> {
        &mut self.clients
    }

    pub fn shutdown_stream(&mut self, peer_addr: SocketAddr) {
        self.clients.retain_mut(|x| x.peer_addr != peer_addr);
    }
}

#[cfg(test)]
mod test {
    use super::Server;
    use std::{
        net::{Ipv4Addr, SocketAddrV4},
        time::Duration,
    };

    #[test]
    fn test_case() {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8888);
        if let Ok(mut server) = Server::bind(std::net::SocketAddr::V4(addr)) {
            let mut count = 0;
            while count < 10 {
                std::thread::sleep(Duration::from_millis(500));
                let data: Vec<u8> = vec![0; 1024];
                server.process_incoming();
                server.broadcast(&data);
                count += 1;
            }
            drop(server);
        }
    }
}
