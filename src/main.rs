mod config;
mod grpc;
mod http;
mod utils;

use log::*;
use structopt::StructOpt;

use std::sync::Arc;
use std::time::Duration;

use utils::{enode_id_to_peer_id, parse_enode, setup_logger};

fn main() {
    let ops = config::Opts::from_args();
    setup_logger(ops.verbose).expect("setup logger failed");

    // Create the runtime
    let core_nums = num_cpus::get();
    warn!("cpu nums: {}", core_nums);
    let mut rt = {
        let mut rt = tokio::runtime::Builder::new();
        if core_nums > 1 {
            rt.threaded_scheduler()
                .core_threads(core_nums)
                .enable_all()
                .build()
                .expect("tokio runtime failed")
        } else {
            rt.basic_scheduler()
                .enable_all()
                .build()
                .expect("tokio runtime failed")
        }
    };

    rt.block_on(async move {
        let ops = ops;
        let pool_enode = ops.pool_enode;
        let pool_name = ops.name;
        let taichi_grpc = ops.taichi_grpc;

        let taichi_node_id = parse_enode(&ops.taichi_enode);
        let taichi_peer_id = hex::encode(enode_id_to_peer_id(taichi_node_id));

        let push_result = http::PushedResult::new(
            pool_name.clone(),
            ops.geth_http,
            taichi_peer_id.clone(),
            Duration::from_secs(60 * ops.poll_interval_minutes),
        );
        let stats_receiver = http::PushedResult::run(Arc::new(push_result));

        grpc::client(pool_name, pool_enode, taichi_grpc, stats_receiver).await;
    });
}
