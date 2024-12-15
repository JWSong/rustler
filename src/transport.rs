use std::{collections::HashMap, net::SocketAddr};

use async_trait::async_trait;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
};

use crate::{Error, GossipMessage};

#[derive(Debug, Clone, Copy)]
pub enum Transport {
    Tcp,
    Udp,
}

#[async_trait]
pub trait NetworkTransport: Send + Sync {
    async fn send(&mut self, to: SocketAddr, message: GossipMessage) -> Result<(), Error>;
    async fn receive(&mut self) -> Result<(SocketAddr, GossipMessage), Error>;
}

pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    pub async fn new(bind_addr: SocketAddr) -> Result<Self, Error> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(Self { socket })
    }
}

#[async_trait]
impl NetworkTransport for UdpTransport {
    async fn send(&mut self, to: SocketAddr, message: GossipMessage) -> Result<(), Error> {
        let bytes = bincode::serialize(&message)?;
        self.socket.send_to(&bytes, to).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<(SocketAddr, GossipMessage), Error> {
        let mut buf = vec![0; 1024];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        let message = bincode::deserialize(&buf[..len])?;
        Ok((addr, message))
    }
}

pub struct TcpTransport {
    listener: TcpListener,
    connections: HashMap<SocketAddr, TcpStream>,
}

impl TcpTransport {
    pub async fn new(bind_addr: SocketAddr) -> Result<Self, Error> {
        let listener = TcpListener::bind(bind_addr).await?;
        Ok(Self {
            listener,
            connections: HashMap::new(),
        })
    }
}

#[async_trait]
impl NetworkTransport for TcpTransport {
    async fn send(&mut self, to: SocketAddr, message: GossipMessage) -> Result<(), Error> {
        let stream: &mut TcpStream = if let Some(stream) = self.connections.get_mut(&to) {
            stream
        } else {
            let stream = TcpStream::connect(to).await?;
            self.connections.insert(to, stream);
            self.connections.get_mut(&to).unwrap()
        };

        let bytes = bincode::serialize(&message)?;
        let len = bytes.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?; // Write length first
        stream.write_all(&bytes).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<(SocketAddr, GossipMessage), Error> {
        let (stream, addr) = self.listener.accept().await?;
        self.connections.insert(addr, stream);
        let stream = self.connections.get_mut(&addr).unwrap();

        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0; len];
        stream.read_exact(&mut buffer).await?;
        let message = bincode::deserialize(&buffer)?;

        Ok((addr, message))
    }
}
