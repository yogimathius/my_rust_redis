use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "my_redis_server",
    about = "A Redis-compatible server implementation"
)]
pub struct Args {
    #[structopt(short, long, default_value = "6379", help = "Port to listen on")]
    pub port: u16,

    #[structopt(long = "replicaof", number_of_values = 2, help = "Set up replication")]
    pub replicaof: Option<Vec<String>>,
}
