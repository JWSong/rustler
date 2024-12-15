use std::{collections::HashMap, net::SocketAddr, time::Duration};

use rand::seq::IteratorRandom as _;
use tokio::time::Instant;

use crate::{
    state::{State, StateContext, SuspectState},
    transport::NetworkTransport,
    GossipMessage,
};

struct ManagedNode {
    state: Box<dyn State>,
    context: StateContext,
}

pub struct FailureDetector {
    self_addr: SocketAddr,
    nodes: HashMap<SocketAddr, ManagedNode>,
    gossip_interval: Duration,
    suspect_duration: Duration,
    fanout: usize,
    gossip_threshold: u64,
}

impl FailureDetector {
    pub fn builder(bind_addr: SocketAddr) -> FailureDetectorBuilder {
        FailureDetectorBuilder::new(bind_addr)
    }

    pub async fn add_node(&mut self, addr: SocketAddr) {
        self.nodes.entry(addr).or_insert_with(|| {
            let context = StateContext {
                gossip: 0,
                gossip_ts: Instant::now(),
                failover: false,
                is_starting: true,
                last_state_change: Instant::now(),
                num_failures: 0,
                suspect_duration: self.suspect_duration,
                gossip_threshold: self.gossip_threshold,
            };

            ManagedNode {
                state: Box::new(SuspectState::new(Instant::now())),
                context,
            }
        });
    }

    async fn handle_gossip(&mut self, msg: GossipMessage) {
        let now = Instant::now();

        // Add any newly discovered nodes
        for addr in msg.gossip_counts.keys() {
            if !self.nodes.contains_key(addr) {
                let context = StateContext {
                    gossip: 0,
                    gossip_ts: now,
                    failover: false,
                    is_starting: true,
                    last_state_change: now,
                    num_failures: 0,
                    suspect_duration: self.suspect_duration,
                    gossip_threshold: self.gossip_threshold,
                };

                let node = ManagedNode {
                    state: Box::new(SuspectState::new(now)),
                    context,
                };

                self.nodes.insert(*addr, node);
            }
        }

        // Update gossip counts and state for known nodes
        for (addr, gossip_count) in msg.gossip_counts {
            if let Some(node) = self.nodes.get_mut(&addr) {
                if gossip_count > node.context.gossip {
                    node.context.gossip = gossip_count;
                    node.context.gossip_ts = now;

                    // Update state using FSM
                    if let Some(new_state) = node.state.update(&mut node.context).await {
                        node.state = new_state;
                    }
                }
            }
        }
    }

    fn select_gossip_targets(&self) -> Vec<SocketAddr> {
        let mut rng = rand::thread_rng();

        self.nodes
            .keys()
            .filter(|&&addr| addr != self.self_addr) // Don't gossip with self
            .choose_multiple(&mut rng, self.fanout) // Random subset of size fanout
            .into_iter()
            .copied()
            .collect()
    }

    pub async fn run(mut self, mut transport: impl NetworkTransport) {
        let mut gossip_interval = tokio::time::interval(self.gossip_interval);

        loop {
            tokio::select! {
                result = transport.receive() => {
                    match result {
                        Ok((from, msg)) => {
                            tracing::info!("Received gossip message from {}", from);
                            self.handle_gossip(msg).await;
                        }
                        Err(e) => {
                            tracing::error!("Failed to receive gossip message: {}", e);
                        }
                    }
                }

                _ = gossip_interval.tick() => {
                    let gossip = GossipMessage {
                        from: self.self_addr,
                        gossip_counts: self.nodes.iter().map(|(addr, node)| (*addr, node.context.gossip)).collect(),
                        failover_list: Vec::new(),
                        starting_nodes: Vec::new(),
                    };

                    for target in self.select_gossip_targets() {
                        if let Err(e) = transport.send(target, gossip.clone()).await {
                            tracing::warn!("Failed to send gossip message: {}", e);
                        }
                    }
                }
            }
        }
    }
}

pub struct FailureDetectorBuilder {
    bind_addr: SocketAddr,
    seed_nodes: Vec<SocketAddr>,
    gossip_interval: Duration,
    suspect_timeout: Duration,
    fanout: usize,
}

impl FailureDetectorBuilder {
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            seed_nodes: Vec::new(),
            gossip_interval: Duration::from_secs(1),
            suspect_timeout: Duration::from_secs(5),
            fanout: 3,
        }
    }

    pub fn with_seed_nodes(mut self, nodes: Vec<SocketAddr>) -> Self {
        self.seed_nodes = nodes;
        self
    }

    pub fn with_gossip_interval(mut self, interval: Duration) -> Self {
        self.gossip_interval = interval;
        self
    }

    pub fn with_suspect_timeout(mut self, timeout: Duration) -> Self {
        self.suspect_timeout = timeout;
        self
    }

    pub fn with_fanout(mut self, fanout: usize) -> Self {
        self.fanout = fanout;
        self
    }

    pub fn build(self) -> FailureDetector {
        FailureDetector {
            self_addr: self.bind_addr,
            nodes: HashMap::new(),
            gossip_interval: self.gossip_interval,
            suspect_duration: self.suspect_timeout,
            fanout: self.fanout,
            gossip_threshold: self.fanout as u64,
        }
    }
}
