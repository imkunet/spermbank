use anyhow::{Context, Result};
use std::{fs, path::PathBuf};
use tracing::warn;

use crate::{cli::SpermOpts, level_region};

fn ensure_target(options: SpermOpts) -> Result<PathBuf> {
    let parent_dir = options
        .world
        .parent()
        .context("could not find parent directory of world to find a place to put the new world")?;

    let world_file_name = options
        .world
        .file_name()
        .context("the world does not have a name for some reason")?;

    let target_file = parent_dir.join(format!(
        "{}.cum",
        world_file_name
            .to_str()
            .context("could not convert the name of the original world file to a string")?
    ));

    if target_file.exists() {
        warn!("existing file found, deleting...");
        fs::remove_file(&target_file).context("couldn't delete the old .cum")?;
    }

    Ok(target_file)
}

pub(crate) fn process_level(options: SpermOpts) -> Result<()> {
    let target_file = ensure_target(options.clone())?;

    level_region::process_level_regions(&target_file, options, "region")?;

    Ok(())
}
