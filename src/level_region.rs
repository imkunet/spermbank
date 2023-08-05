use std::{
    fs::{File, OpenOptions},
    io::{self, Cursor, Write},
    path::PathBuf,
};

use anvil_region::{
    position::RegionPosition,
    provider::{FolderRegionProvider, RegionProvider},
    region::Region,
};
use anyhow::{Context, Result};
use tracing::warn;

use crate::{
    blocks::BLOCKS, cli::SpermOpts, pipes::MeteredPipe, region_iterator::IntoRegionIterator,
};

pub(crate) fn process_level_regions(
    target_file: &PathBuf,
    options: SpermOpts,
    sub_path: &str,
) -> Result<()> {
    let mut source = Vec::new();
    let mut pipe = MeteredPipe::new("uncompressed", &mut source);

    let path = options.world.join(sub_path);

    let provider = FolderRegionProvider::new(
        path.to_str()
            .context("could not locate region folder path")?,
    );

    let region_positions: Vec<RegionPosition> = provider
        .iter_positions()
        .context("could not fetch all regions")?
        .collect();

    for region_position in region_positions {
        let Ok(region) = provider.get_region(region_position) else {
            panic!(
                "error in reading region {} {}",
                region_position.x, region_position.z
            );
        };

        process_region(&region_position, region, &mut pipe)?;
    }

    pipe.flush()?;

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target_file)
        .context("could not open output file")?;
    let sink = MeteredPipe::new("zstd", &mut file);
    let mut encoder = zstd::stream::Encoder::new(sink, 0)?;
    encoder.set_pledged_src_size(Some(source.len() as u64))?;
    encoder.include_contentsize(true)?;
    let mut final_boss = encoder.auto_finish();

    io::copy(&mut Cursor::new(source), &mut final_boss)?;
    final_boss.flush()?;

    Ok(())
}

fn process_region(
    region_position: &RegionPosition,
    region: Region<File>,
    encoder: &mut impl Write,
) -> Result<()> {
    for (chunk, region_chunk_pos) in region.into_ext_iter() {
        let Ok(level) = chunk.get_compound_tag("Level") else {
            warn!(
                "invalid chunk (no level tag) r:{:?} p:{:?}",
                region_position, region_chunk_pos
            );
            continue;
        };

        let Ok(mut sections) = level.get_compound_tag_vec("Sections") else {
            warn!("invalid chunk (no sections) r:{:?} p:{:?}", region_position, region_chunk_pos);
            continue;
        };

        sections.retain(|section| {
            let Ok(blocks) = section.get_i8_vec("Blocks") else {
                warn!("invalid chunk section; blocks does not exist in section!");
                return false;
            };

            if section.get_i8_vec("Data").is_err() {
                warn!("invalid chunk section; no data...");
                return false;
            }

            if section.get_i8("Y").is_err() {
                warn!("invalid chunk section; no Y");
                return false;
            }

            blocks.iter().any(|block| *block != 0)
        });

        if sections.is_empty() {
            continue;
        }

        let mut section_mask: u16 = 0;
        for section in &sections {
            let section_y = section.get_i8("Y").unwrap();
            section_mask |= 1 << section_y;
        }

        let chunk_x = region_position.x * 32 + region_chunk_pos.x as i32;
        let chunk_z = region_position.z * 32 + region_chunk_pos.z as i32;

        let mut hunk = Vec::new();

        hunk.extend_from_slice(&chunk_x.to_be_bytes());
        hunk.extend_from_slice(&chunk_z.to_be_bytes());
        hunk.extend_from_slice(&section_mask.to_be_bytes());

        for section in &sections {
            let blocks = section.get_i8_vec("Blocks").unwrap();
            let non_zero_count = blocks.iter().filter(|block| **block != 0).count() as u16;
            hunk.extend_from_slice(&non_zero_count.to_be_bytes());
            let data = section.get_i8_vec("Data").unwrap();
            for (i, block) in blocks.iter().enumerate() {
                let j = i / 2;
                let d = if i % 2 == 0 {
                    data[j] & 15
                } else {
                    data[j] >> 4 & 15
                };

                let types = BLOCKS[*block as u8 as usize];
                let mut data = types[d as u8 as usize];
                if data == -1 {
                    warn!("invalid block {}:{}", block, data);
                    data = 0;
                }

                hunk.extend_from_slice(&data.to_be_bytes());
            }
        }

        encoder.write_all(&hunk)?;
    }

    Ok(())
}
