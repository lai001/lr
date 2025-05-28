use crate::{
    codec::{Decoder, Message},
    length_prefix_decoder::LengthPrefixDecoder,
};
use std::{
    io::Read,
    net::{SocketAddr, TcpStream},
    time::Duration,
};

pub struct Client {
    stream: TcpStream,
    decoder: LengthPrefixDecoder,
    buffer: Vec<u8>,
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}

impl Client {
    pub fn bind(addr: SocketAddr) -> crate::error::Result<Client> {
        let timeout = Duration::from_secs_f32(30.0);
        let tcp_stream = TcpStream::connect_timeout(&addr, timeout)
            .map_err(|err| crate::error::Error::IO(err, None))?;
        match tcp_stream.set_read_timeout(Some(Duration::from_millis(5000))) {
            Ok(_) => {}
            Err(err) => {
                log::warn!("{err}")
            }
        }
        let decoder = LengthPrefixDecoder::new();
        Ok(Client {
            stream: tcp_stream,
            decoder,
            buffer: vec![0; 512 * 10],
        })
    }

    pub fn try_read(&mut self) -> crate::error::Result<()> {
        match self.stream.read(&mut self.buffer) {
            Ok(size) => {
                if self.buffer.len() < size {
                    self.buffer.resize(size, 0);
                }
                let _ = self.decoder.decode((&self.buffer[0..size]).to_vec());
                Ok(())
            }
            Err(err) => Err(crate::error::Error::IO(err, None)),
        }
    }

    pub fn take_messages(&mut self) -> Vec<Message> {
        self.decoder.take_messages()
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
        match Client::bind(std::net::SocketAddr::V4(addr)) {
            Ok(mut client) => loop {
                std::thread::sleep(Duration::from_millis(500));
                if let Err(err) = client.try_read() {
                    eprintln!("{}", err);
                    break;
                }
                let messages = client.take_messages();
                for message in messages {
                    println!("message data len: {:?}", message.data.len());
                }
            },
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }
}
