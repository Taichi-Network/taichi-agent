use futures::StreamExt;
use log::*;
use reqwest::Error as HttpError;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use std::ops::Sub;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 轮询http rpc接口，获取目标节点收到的交易数量情况
pub struct PushedResult {
    http_url: String,
    name: String,
    client: reqwest::Client,
    req_id: Arc<AtomicU64>,
    taichi_id: String, // taichi peer id, not enode id
    poll_interval: Duration,
}

impl PushedResult {
    pub fn new(name: String, http_url: String, taichi_id: String, poll_interval: Duration) -> Self {
        let client = reqwest::Client::new();
        Self {
            name,
            http_url,
            client,
            req_id: Arc::new(AtomicU64::new(0)),
            taichi_id,
            poll_interval,
        }
    }
    async fn set_taichi_peer(&self) -> Result<bool, HttpError> {
        let req_id = self.req_id.clone();
        let id = req_id.fetch_add(1, Ordering::Relaxed);

        let r = SetTaichiPeerReq::new(id, self.taichi_id.clone());
        let resp = self
            .client
            .post(&self.http_url)
            .json(&r)
            .send()
            .await?
            .json::<SetTaichiPeerResp>()
            .await?;
        Ok(resp.result)
    }

    async fn get_taichi_peer(&self) -> Result<String, HttpError> {
        let req_id = self.req_id.clone();
        let id = req_id.fetch_add(1, Ordering::Relaxed);

        let r = GetTaichiPeerReq::new(id);
        let resp = self
            .client
            .post(&self.http_url)
            .json(&r)
            .send()
            .await?
            .json::<GetTaichiPeerResp>()
            .await?;
        Ok(resp.result)
    }

    async fn get_taichi_stats(&self) -> Result<[u64; 5], HttpError> {
        let req_id = self.req_id.clone();
        let id = req_id.fetch_add(1, Ordering::Relaxed);
        let r = GetTaichiStatsReq::new(id);
        let resp = self
            .client
            .post(&self.http_url)
            .json(&r)
            .send()
            .await?
            .json::<GetTaichiStatsResp>()
            .await?;
        Ok(resp.result)
    }

    pub fn run(this: Arc<Self>) -> UnboundedReceiver<StatsResult> {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let poll_interval = this.poll_interval;
            let name = &this.name;
            let tx = tx;

            // first must set_taichi_peered
            let mut fitler_req_itv = tokio::time::interval(Duration::from_secs(5));
            loop {
                match this.set_taichi_peer().await {
                    Ok(true) => break,
                    Ok(false) => (),
                    Err(e) => error!("set_taichi_peer error: {}", e),
                }
                fitler_req_itv.next().await;
            }

            let mut itv = tokio::time::interval(poll_interval);
            let mut stats: Option<StatsResult> = None;
            while let Some(_i) = itv.next().await {
                match this.get_taichi_peer().await {
                    Ok(peer_id) => {
                        if peer_id.is_empty() || (&peer_id[0..16] != &this.taichi_id[0..16]) {
                            warn!("taichi peer id was modified to {}, not taichi id: {}", peer_id, this.taichi_id);
                            match this.set_taichi_peer().await {
                                Ok(true) => info!("recovery set_taichi_peer id success"),
                                Ok(false) => warn!("recovery set_taichi_peer id failed"),
                                Err(e) => error!("recovery set_taichi_peer id error: {}", e),
                            }
                            continue;
                        }
                    }
                    Err(e) => error!("get taichi peer error: {}", e),
                }

                match this.get_taichi_stats().await {
                    Ok(data) => {
                        let new_cnt = &StatsResult::new(data);
                        match stats.as_mut() {
                            None => stats = Some(*new_cnt),
                            Some(cnt) => {
                                let delta = new_cnt - cnt;
                                tx.send(delta).unwrap_or_else(|e| {
                                    error!("send StatsResult to ch error: {}", e)
                                });

                                *cnt = *new_cnt;
                                info!(
                                    "target node: {}, tx cnt: {}, ratio: {:.4}",
                                    name,
                                    delta,
                                    if delta.accept == 0 {
                                        0.0
                                    } else {
                                        delta.pushed as f64 / delta.accept as f64
                                    }
                                );
                            }
                        }
                    }
                    Err(e) => error!("get stats error: {}", e),
                }
            }
        });
        rx
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SetTaichiPeerReq {
    id: u64,
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<String>,
}

impl SetTaichiPeerReq {
    fn new(id: u64, peer_id: String) -> Self {
        Self {
            id,
            method: "eth_setTaichiPeer",
            jsonrpc: "2.0",
            params: vec![peer_id],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GetTaichiPeerReq {
    id: u64,
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<String>,
}

impl GetTaichiPeerReq {
    fn new(id: u64) -> Self {
        Self {
            id,
            method: "eth_getTaichiPeer",
            jsonrpc: "2.0",
            params: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GetTaichiStatsReq {
    id: u64,
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<String>,
}

impl GetTaichiStatsReq {
    fn new(id: u64) -> Self {
        Self {
            id,
            method: "eth_getTaichiStats",
            jsonrpc: "2.0",
            params: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SetTaichiPeerResp {
    id: u64,
    jsonrpc: String,
    result: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetTaichiPeerResp {
    id: u64,
    jsonrpc: String,
    result: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetTaichiStatsResp {
    id: u64,
    jsonrpc: String,
    result: [u64; 5],
}
#[derive(Serialize, Deserialize, Debug)]
struct ErrorResp {
    id: u64,
    jsonrpc: String,
    error: ErrorMsg,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorMsg {
    code: i64,
    message: String,
}

#[derive(Clone)]
pub struct StatsResult {
    pub accept: u64,
    pub pushed: u64,
    pub dup: u64,
    pub under_price: u64,
    pub other: u64,
}
impl Copy for StatsResult {}

impl StatsResult {
    pub fn new(data: [u64; 5]) -> Self {
        StatsResult {
            accept: data[0],
            pushed: data[1],
            dup: data[2],
            under_price: data[3],
            other: data[4],
        }
    }
}

impl Sub<&mut StatsResult> for &StatsResult {
    type Output = StatsResult;

    fn sub(self, rhs: &mut StatsResult) -> Self::Output {
        StatsResult::new([
            self.accept.checked_sub(rhs.accept).unwrap_or(0),
            self.pushed.checked_sub(rhs.pushed).unwrap_or(0),
            self.dup.checked_sub(rhs.dup).unwrap_or(0),
            self.under_price.checked_sub(rhs.under_price).unwrap_or(0),
            self.other.checked_sub(rhs.other).unwrap_or(0),
        ])
    }
}

use std::fmt;
impl fmt::Display for StatsResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TxCountResult: accept {}, pushed {}, duplicate {}, under price {}, other {}",
            self.accept, self.pushed, self.dup, self.under_price, self.other
        )
    }
}
