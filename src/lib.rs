pub mod error;
pub mod events;
pub mod failure_detector;
pub mod state;
pub mod transport;

pub use error::Error;
pub use events::{Event, GossipMessage};

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use failure_detector::FailureDetector;
    use transport::{TcpTransport, UdpTransport};

    use crate::*;

    #[tokio::test]
    async fn usage_udp_example() {
        let transport = UdpTransport::new(SocketAddr::from(([127, 0, 0, 1], 8080)))
            .await
            .unwrap();
        let failure_detector =
            FailureDetector::builder(SocketAddr::from(([127, 0, 0, 1], 8080))).build();
        failure_detector.run(transport).await;
    }

    #[tokio::test]
    async fn usage_tcp_example() {
        let transport = TcpTransport::new(SocketAddr::from(([127, 0, 0, 1], 8080)))
            .await
            .unwrap();
        let failure_detector =
            FailureDetector::builder(SocketAddr::from(([127, 0, 0, 1], 8080))).build();
        failure_detector.run(transport).await;
    }
}
