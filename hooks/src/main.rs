mod parser;
mod sender;

use std::io::Read;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "agentos-hook",
    version,
    about = "AgentOS hook CLI - forwards agent events to daemon"
)]
struct Cli {
    #[arg(
        short = 's',
        long = "source",
        help = "Agent source (opencode, claude, codex, etc.)"
    )]
    source: Option<String>,

    #[arg(short = 'e', long = "event", help = "Inline JSON event payload")]
    event: Option<String>,

    #[arg(long = "from-stdin", help = "Read payload from stdin (default)")]
    from_stdin: bool,
}

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::WARN)
        .try_init();

    let cli = Cli::parse();

    let payload: String;
    let source: String;

    if let Some(event_str) = cli.event {
        payload = event_str;
        source = cli.source.unwrap_or_else(|| "unknown".to_string());
    } else {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .unwrap_or_default();

        if input.trim().is_empty() {
            std::process::exit(0);
        }

        let hook_input = match parser::parse_input(&input) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[agentos-hook] parse error: {}", e);
                std::process::exit(0);
            }
        };

        source = match parser::resolve_source(cli.source, hook_input.source) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[agentos-hook] {}", e);
                std::process::exit(0);
            }
        };

        payload = hook_input
            .event
            .map(|e| e.to_string())
            .unwrap_or_else(|| input.trim().to_string());
    }

    let event_value: serde_json::Value = match serde_json::from_str(&payload) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[agentos-hook] invalid event JSON: {}", e);
            std::process::exit(0);
        }
    };

    if let Err(e) = sender::send_event(&source, event_value) {
        eprintln!("[agentos-hook] {}", e);
    }

    std::process::exit(0);
}
