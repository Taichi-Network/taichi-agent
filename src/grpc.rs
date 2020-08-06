use futures::StreamExt;
use log::*;
use tokio::sync::mpsc::UnboundedReceiver;
use tonic::{transport::Channel, Request};

use crate::http::StatsResult;
use crate::utils::now_milis;

use std::time::Duration;

mod agent {
    tonic::include_proto!("scout");
}

pub use agent::{
    scout_client::ScoutClient,
    scout_server::{Scout, ScoutServer},
    Ack, TxCnt,
};

type GrpcClient = ScoutClient<Channel>;

impl TxCnt {
    pub fn new(name: String, self_enode: String, ts: u128, stats: StatsResult) -> Self {
        Self {
            name,
            enode: self_enode,
            timestamp: ts.to_le_bytes().to_vec(),
            all: stats.accept,
            pushed: stats.pushed,
            dup: stats.dup,
            under_price: stats.under_price,
            other: stats.other,
        }
    }
}

pub async fn client(
    name: String,
    enode: String,
    url: String,
    mut stats_receiver: UnboundedReceiver<StatsResult>,
) {
    let mut client: GrpcClient = {
        match connect(url.clone()).await {
            Some(c) => c,
            None => {
                error!("connect to taichi grpc failed");
                return;
            }
        }
    };
    while let Some(stats) = stats_receiver.next().await {
        let tx_cnt = TxCnt::new(name.clone(), enode.clone(), now_milis(), stats);
        match client.send_txs_cnt(Request::new(tx_cnt)).await {
            Ok(resp) => {
                let resp = resp.into_inner();
                info!(
                    "send tx count to taichi success: {} {}",
                    resp.code, resp.msg
                );
            }
            Err(status) => {
                error!("grpc send tx count error: {}", status);
                match status.code() {
                    tonic::Code::Unavailable => {
                        if let Some(new_client) = connect(url.clone()).await {
                            client = new_client;
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

async fn connect(url: String) -> Option<GrpcClient> {
    let itv = Duration::from_secs(3);
    for i in 0..3_u8 {
        match GrpcClient::connect(url.clone()).await {
            Ok(client) => {
                return Some(client);
            }
            Err(e) => {
                error!("grpc connect error: {}, retry {} ...", e, i);
                tokio::time::delay_for(itv).await;
            }
        }
    }
    None
}
