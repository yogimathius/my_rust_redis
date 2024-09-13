use clap::Parser as ClapParser;

#[derive(ClapParser, Debug, Clone)]
pub struct Args {
    #[arg(short, long, default_value_t = 6379)]
    pub port: u16,
    #[arg(short, long, value_delimiter = ' ', num_args = 1)]
    pub replicaof: Option<Vec<String>>,
}
