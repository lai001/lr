use crate::{codec::Encoder, length_prefix_encoder::LengthPrefixEncoder};
use std::{
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub struct Server {
    streams: Vec<TcpStream>,
    encoder: LengthPrefixEncoder,
    recciver: std::sync::mpsc::Receiver<TcpStream>,
    shutdown: Arc<AtomicBool>,
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
        let _ = std::thread::Builder::new()
            .name(format!("Network"))
            .spawn({
                let sender = sender.clone();
                let shutdown = shutdown.clone();
                move || {
                    let listener =
                        TcpListener::bind(addr).map_err(|err| crate::error::Error::IO(err, None));
                    match listener {
                        Ok(listener) => {
                            for stream in listener.incoming() {
                                if shutdown.load(Ordering::Relaxed) {
                                    break;
                                }
                                match stream {
                                    Ok(stream) => {
                                        let _ = sender.send(stream);
                                    }
                                    Err(err) => {
                                        log::warn!("{}", err);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("{}", err);
                        }
                    }
                }
            })
            .map_err(|err| crate::error::Error::IO(err, None))?;

        let encoder = LengthPrefixEncoder::new(rs_artifact::EEndianType::Little);
        Ok(Server {
            encoder,
            streams: Vec::new(),
            recciver,
            shutdown: shutdown,
        })
    }

    pub fn process_incoming(&mut self) {
        for stream in self.recciver.try_iter() {
            log::trace!("New stream {:?}", stream.peer_addr(),);
            self.streams.push(stream);
        }
    }

    pub fn broadcast(&mut self, data: &[u8]) {
        let encoded = self.encoder.encode(data).unwrap();
        for stream in &mut self.streams {
            match stream.write(&encoded) {
                Ok(size) => {
                    log::trace!("Broadcast {:?}, {}", stream.peer_addr(), size);
                    let _ = stream.flush();
                }
                Err(err) => {
                    log::warn!("{:?}, {}", stream.peer_addr(), err);
                }
            }
        }
    }

    pub fn shutdown_all_streams(&mut self) {
        for stream in &mut self.streams {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        self.streams.clear();
    }

    pub fn shutdown(mut self) {
        self.shutdown_internal();
    }

    fn shutdown_internal(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.shutdown_all_streams();
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
