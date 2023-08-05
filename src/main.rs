use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_panic::panic_hook;

use crate::cli::SpermOpts;

mod blocks;
mod cli;
mod level;
mod level_region;
mod pipes;
mod region_iterator;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    std::panic::set_hook(Box::new(panic_hook));
    info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let options = SpermOpts::parse();
    if !options.world.is_dir() {
        panic!("the specified world is not a directory");
    }

    let now = Instant::now();
    level::process_level(options)?;
    let duration = Instant::now().duration_since(now);
    info!("finished processing in {:?}", duration);

    Ok(())
}
