use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};

use crate::state::NodeState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    GossipReceived {
        from: SocketAddr,
        message: GossipMessage,
    },
    NodeStateChanged {
        node: SocketAddr,
        state: NodeState,
    },
    NodeAdded {
        addr: SocketAddr,
    },
    NodeRemoved {
        addr: SocketAddr,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GossipMessage {
    pub from: SocketAddr,
    pub gossip_counts: HashMap<SocketAddr, u64>,
    pub failover_list: Vec<SocketAddr>,
    pub starting_nodes: Vec<SocketAddr>,
}
