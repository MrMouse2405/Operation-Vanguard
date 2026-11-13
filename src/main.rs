use std::{
    error::Error, fs, path::PathBuf,
};

use clap::Parser;

mod web;
mod sim;
mod runtime;
mod server;
mod memory;
mod network;
mod scoring;
mod debug;
mod projection;

#[derive(Parser)]
#[command(name = "vanguard", version, about = "Run one supplied Lua agent in a private VM on every router")]
struct Args {
    /// Agent entrypoint; the same code runs independently on every router
    #[arg(long)]
    script: PathBuf,

    /// Explicit TOML topology and optional simulation settings
    #[arg(long)]
    network: PathBuf,

    /// Port for the local web interface (0 selects an available port)
    #[arg(long, default_value_t = 8000)]
    port: u16,

    /// Run deterministic committed barriers without a browser/server
    #[arg(long)]
    headless: bool,

    /// Number of barriers in headless mode
    #[arg(long, default_value_t = 100)]
    ticks: u64,

    /// Write per-tick semantic hashes and final snapshot as JSON
    #[arg(long, requires = "headless")]
    replay: Option<PathBuf>,

    /// Print effective network settings without starting the run
    #[arg(long)]
    print_config: bool,

    /// Enable agent-authored dbgguard visualizations (uses normal fuel)
    #[arg(long)]
    dbg: bool,
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let files = network::RunFiles { script: args.script, network: args.network, dbg: args.dbg };
    let config = files.load().map_err(std::io::Error::other)?;
    if args.print_config {
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }
    if args.headless {
        let mut runtime = runtime::Runtime::new(config, "headless".into()).map_err(std::io::Error::other)?;
        let mut hashes = Vec::new();
        if args.replay.is_some(){hashes.push(serde_json::json!({"tick":0,"hash":runtime.state_hash()}));}
        for _ in 0..args.ticks {
            runtime.step().map_err(std::io::Error::other)?;
            if args.replay.is_some(){hashes.push(serde_json::json!({"tick":runtime.core.borrow().tick,"hash":runtime.state_hash()}));}
            if runtime.control == "FINISHED" { break; }
        }
        let final_hash=hashes.last().and_then(|entry|entry["hash"].as_str()).map(str::to_owned).unwrap_or_else(||runtime.state_hash());
        if let Some(path) = args.replay {
            let result = serde_json::json!({"hashes":hashes,"snapshot":runtime.snapshot()});
            let mut writer=std::io::BufWriter::new(fs::File::create(path)?);
            serde_json::to_writer_pretty(&mut writer, &result)?;
            std::io::Write::flush(&mut writer)?;
        }
        println!("Tick {} · semantic hash {}", runtime.core.borrow().tick, final_hash);
        return Ok(());
    }
    web::serve(files, args.port).await?;
    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    #[test]
    fn script_and_network_are_required_named_arguments() {
        assert!(Args::try_parse_from(["vanguard"]).is_err());
        assert!(Args::try_parse_from(["vanguard", "--script", "agent.lua"]).is_err());
        assert!(Args::try_parse_from(["vanguard", "--network", "network.toml"]).is_err());
        assert!(Args::try_parse_from(["vanguard", "agent.lua", "--network", "network.toml"]).is_err());
        assert!(Args::try_parse_from(["vanguard", "--script", "agent.lua", "--network", "network.toml"]).is_ok());
    }
}
