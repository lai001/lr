use crate::{
    codec::{Decoder, Encoder, Message},
    length_prefix_decoder::LengthPrefixDecoder,
    length_prefix_encoder::LengthPrefixEncoder,
};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

pub struct Client {
    recciver: std::sync::mpsc::Receiver<Vec<u8>>,
    sender: std::sync::mpsc::Sender<Vec<u8>>,
    encoder: LengthPrefixEncoder,
    decoder: LengthPrefixDecoder,
    shutdown: Arc<AtomicBool>,
    pub peer_addr: SocketAddr,
    pub local_addr: SocketAddr,
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Client {
    pub fn bind(addr: SocketAddr, debug_label: Option<String>) -> crate::error::Result<Client> {
        let timeout = Duration::from_secs_f32(30.0);
        let tcp_stream = TcpStream::connect_timeout(&addr, timeout).map_err(|err| {
            crate::error::Error::IO(
                err,
                Some(format!("Failed to connect to remote address: {}", addr)),
            )
        })?;

        match tcp_stream.set_read_timeout(Some(Duration::from_millis(1))) {
            Ok(_) => {}
            Err(err) => {
                log::warn!("{err}")
            }
        }

        Self::from_stream(tcp_stream, debug_label)
    }

    pub fn from_stream(
        mut tcp_stream: TcpStream,
        debug_label: Option<String>,
    ) -> crate::error::Result<Client> {
        let local_addr = tcp_stream
            .local_addr()
            .map_err(|err| crate::error::Error::IO(err, None))?;
        let peer_addr = tcp_stream
            .peer_addr()
            .map_err(|err| crate::error::Error::IO(err, None))?;
        log::trace!("local_addr: {}, peer_addr: {}", local_addr, peer_addr);

        let shutdown = Arc::new(AtomicBool::new(false));
        let (sender, recciver) = std::sync::mpsc::channel::<Vec<u8>>();
        let (sender1, recciver1) = std::sync::mpsc::channel::<Vec<u8>>();
        let _ = std::thread::Builder::new()
            .name(format!("Network"))
            .spawn({
                let sender = sender.clone();
                let shutdown = shutdown.clone();
                #[cfg(feature="network_debug_trace")]
                let debug_label = debug_label.clone();
                move || {
                    let mut buffer: Vec<u8> = vec![0; 512 * 10];
                    let mut write_buffer: Vec<u8> = vec![];
                    loop {
                        if shutdown.load(Ordering::Relaxed) {
                            break;
                        }
                        match tcp_stream.read(&mut buffer) {
                            Ok(size) => {
                                if size != 0 {
                                    if buffer.len() < size {
                                        buffer.resize(size, 0);
                                    }
                                    #[cfg(feature="network_debug_trace")]
                                    match &debug_label {
                                        Some(debug_label) => {
                                            log::trace!("[{debug_label}] Receive data. {size}");
                                        }
                                        None => {
                                            log::trace!("Receive data. {size}");
                                        }
                                    }
                                    let _ = sender.send(buffer[0..size].to_vec());
                                }
                            }
                            Err(err) => {
                                if !matches!(err.kind(), std::io::ErrorKind::TimedOut) {
                                    match &debug_label {
                                        Some(debug_label) => {
                                            log::warn!("[{debug_label}] Failed to read from: {peer_addr}, {err}");
                                        }
                                        None => {
                                            log::warn!("Failed to read from: {peer_addr}, {err}");
                                        }
                                    }
                                }
                            }
                        }
                        match recciver1.try_recv() {
                            Ok(mut data) => {
                                write_buffer.append(&mut data);
                                match tcp_stream.write(write_buffer.as_slice()) {
                                    Ok(bytes) => {
                                        // let _ = tcp_stream.flush();
                                        if bytes != 0 {
                                            #[cfg(feature="network_debug_trace")]
                                            match &debug_label {
                                                Some(debug_label) => {
                                                    log::trace!(
                                                        "[{debug_label}] Write to: {peer_addr}, {bytes}"
                                                    );
                                                }
                                                None => {
                                                    log::trace!("Write to: {peer_addr}, {bytes}");
                                                }
                                            }
                                            write_buffer.drain(0..bytes);
                                        }
                                    }
                                    Err(err) => {
                                        if !matches!(err.kind(), std::io::ErrorKind::TimedOut) {
                                            match &debug_label {
                                                Some(debug_label) => {
                                                    log::warn!("[{debug_label}] Failed to write to: {peer_addr}, {err}");
                                                }
                                                None => {
                                                    log::warn!("Failed to write to: {peer_addr}, {err}");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    let _ = tcp_stream.shutdown(std::net::Shutdown::Both);
                    log::trace!(
                        "Shutdown stream:, local_addr: {local_addr}, peer_addr: {peer_addr}"
                    );
                }
            })
            .map_err(|err| crate::error::Error::IO(err, None))?;

        let decoder = LengthPrefixDecoder::new();
        let encoder = LengthPrefixEncoder::new(rs_artifact::EEndianType::Little);

        Ok(Client {
            recciver,
            sender: sender1,
            encoder,
            decoder,
            shutdown,
            peer_addr,
            local_addr,
        })
    }

    pub fn take_messages(&mut self) -> Vec<Message> {
        while let Ok(data) = self.recciver.try_recv() {
            let result = self.decoder.decode(data);
            if let Err(err) = result {
                log::warn!("{}", err);
            }
        }
        self.decoder.take_messages()
    }

    pub fn write(&mut self, buf: Vec<u8>) {
        let encoded = self.encoder.encode(&buf).unwrap();
        match self.sender.send(encoded) {
            Ok(_) => {}
            Err(err) => {
                log::warn!("Write, {err}");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Client;
    use std::{
        net::{Ipv4Addr, SocketAddrV4},
        time::Duration,
    };

    #[test]
    fn test_case() {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8888);
        match Client::bind(std::net::SocketAddr::V4(addr), None) {
            Ok(mut client) => {
                let mut count = 0;
                while count < 10 {
                    std::thread::sleep(Duration::from_millis(500));
                    let messages = client.take_messages();
                    for message in messages {
                        assert_eq!(message.data.len(), 1024);
                    }
                    count += 1;
                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }
}
