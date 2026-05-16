use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "agentosd",
    version,
    about = "AgentOS daemon - AI agent system cockpit"
)]
pub struct CliArgs {
    #[arg(short = 's', long = "socket", default_value = ".agentosd.sock")]
    pub socket_path: PathBuf,

    #[arg(
        short = 'd',
        long = "db-path",
        default_value = "~/.local/share/agentos/agentosd.db"
    )]
    pub db_path: PathBuf,

    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[arg(long = "max-connections", default_value = "32")]
    pub max_connections: u32,

    #[arg(long = "db-in-memory")]
    pub db_in_memory: bool,
}
