use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use parity_crypto::Keccak256;

pub fn now_milis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("get timestamp failed")
        .as_millis()
}

pub fn setup_logger(verbose: u8) -> Result<(), fern::InitError> {
    let level = log_level(verbose);

    fern::Dispatch::new()
        .format(|out, message, record| {
            let colors = ColoredLevelConfig::new().info(Color::Green);
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S%.3f%:z]"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn log_level(verbose: u8) -> LevelFilter {
    if verbose == 1 {
        LevelFilter::Info
    } else if verbose == 2 {
        LevelFilter::Debug
    } else if verbose == 3 {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    }
}

use ethereum_types::H512;
use std::str::FromStr;

pub fn parse_enode(enode: &str) -> H512 {
    if &enode[0..5] == "enode" {
        H512::from_str(&enode[8..136]).expect("enode parse failed")
    } else {
        H512::from_str(&enode[0..128]).expect("enode parse failed")
    }
}
pub fn enode_id_to_peer_id(node_id: H512) -> [u8; 32] {
    let s: [u8; 64] = node_id.into();
    let s = s.to_vec();
    Keccak256::keccak256(&s)
}
