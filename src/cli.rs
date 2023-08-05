use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Parser, Clone)]
#[command(
    about = "Covert your worlds to a frozen 'cum' format",
    author = "KuNet",
    version = env!("CARGO_PKG_VERSION")
)]
pub(crate) struct SpermOpts {
    #[arg()]
    pub(crate) world: PathBuf,
}
