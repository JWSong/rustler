use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};

pub struct StateContext {
    pub gossip: u64,
    pub gossip_ts: Instant,
    pub failover: bool,
    pub is_starting: bool,
    pub last_state_change: Instant,
    pub num_failures: u32,
    pub suspect_duration: Duration,
    pub gossip_threshold: u64,
}

#[async_trait]
pub trait State: Send + Sync {
    fn kind(&self) -> NodeState;
    async fn update(&self, ctx: &mut StateContext) -> Option<Box<dyn State>>;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    Dead,
    Suspect,
    Alive,
}

pub struct AliveState;

#[async_trait]
impl State for AliveState {
    fn kind(&self) -> NodeState {
        NodeState::Alive
    }

    async fn update(&self, ctx: &mut StateContext) -> Option<Box<dyn State>> {
        let time_since_gossip = ctx.gossip_ts.elapsed();
        if time_since_gossip > Duration::from_secs(ctx.gossip_threshold) {
            ctx.num_failures += 1;
            Some(Box::new(SuspectState {
                entered_at: Instant::now(),
            }))
        } else {
            None
        }
    }
}
pub struct SuspectState {
    entered_at: Instant,
}

impl SuspectState {
    pub fn new(entered_at: Instant) -> Self {
        Self { entered_at }
    }
}

#[async_trait]
impl State for SuspectState {
    fn kind(&self) -> NodeState {
        NodeState::Suspect
    }

    async fn update(&self, ctx: &mut StateContext) -> Option<Box<dyn State>> {
        let time_since_gossip = ctx.gossip_ts.elapsed();

        if time_since_gossip > ctx.suspect_duration {
            // Suspect 기간 동안 gossip이 없으면 Dead로
            ctx.num_failures += 1;
            Some(Box::new(DeadState))
        } else if time_since_gossip <= Duration::from_secs(1)
            && self.entered_at.elapsed() >= ctx.suspect_duration
        {
            // Suspect 기간이 지나고 gossip이 정상이면 Alive로
            ctx.num_failures = 0;
            Some(Box::new(AliveState))
        } else {
            None
        }
    }
}

pub struct DeadState;

#[async_trait]
impl State for DeadState {
    fn kind(&self) -> NodeState {
        NodeState::Dead
    }

    async fn update(&self, ctx: &mut StateContext) -> Option<Box<dyn State>> {
        let time_since_gossip = ctx.gossip_ts.elapsed();
        if time_since_gossip <= Duration::from_secs(1) {
            Some(Box::new(SuspectState {
                entered_at: Instant::now(),
            }))
        } else {
            None
        }
    }
}
