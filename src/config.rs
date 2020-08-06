use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Taichi Agent", about = "An Agent to get tx pushed data.")]
pub struct Opts {
    /// enode of pool node
    #[structopt(long)]
    pub pool_enode: String,
    /// pool name
    #[structopt(short, long)]
    pub name: String,
    /// json-rpc url of pool node
    #[structopt(short, long, default_value = "http://127.0.0.1:8545")]
    pub geth_http: String,
    /// Taichi GRpc url
    #[structopt(long, default_value = "http://grpc.taichi.network:11003")]
    pub taichi_grpc: String,
    /// how long to fetch tx pushed data
    #[structopt(short, long, default_value = "1")]
    pub poll_interval_minutes: u64,
    /// enode of Taichi push node
    #[structopt(
        long,
        default_value = "enode://24a2bdca9fae77873ecedfbb3d418a524601790de393f8fa62620fd2092429b5c40fbc30d4e89049964dcf9167913a1b2198592044ee72c8eeec4c7c3fd29336@47.114.137.69:32303"
    )]
    pub taichi_enode: String,
    /// log verbose, defualt WARN, -v Info, -VV DEBUG
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
