use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use renderware_format as rwf;
use rwf::{dff::Model, raw::BinaryStreamFile, txd::Texture};
use vice_city_formats::{dat::GtaVcDat, Ide};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Where to look for assets
    #[clap(short, long)]
    assets: PathBuf,

    /// Where to deposit the converted files
    #[clap(short, long)]
    output: PathBuf,

    /// Path of the file to repack
    #[clap()]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let data_folder = args.assets.join("data");
    let vc_dat = GtaVcDat::parse(&fs::read_to_string(data_folder.join("gta_vc.dat"))?);

    let models_to_textures: HashMap<_, _> = vc_dat
        .ides
        .iter()
        .map(|filename| data_folder.join(filename.strip_prefix("data/").unwrap()))
        .map(|path| fs::read_to_string(path).unwrap())
        .map(|data| Ide::parse(&data))
        .flat_map(|ide| ide.model_to_texture_map().collect::<Vec<_>>())
        .collect();

    let model_path = args.path;
    let file_stem = model_path.file_stem().unwrap().to_string_lossy();

    let texture_name = models_to_textures
        .get(&file_stem.to_string())
        .context("failed to find corresponding texture")?;
    let texture_path = model_path.with_file_name(&format!("{}.txd", texture_name));

    let models = Model::from_raw(&BinaryStreamFile::open(&model_path)?);
    let textures = Texture::from_raw(&BinaryStreamFile::open(&texture_path)?);

    fs::create_dir_all(&args.output)?;
    for (index, (_, model)) in models.iter().enumerate() {
        let texture = rwf::packer::repack_model_textures(
            &model.materials,
            &model.material_indices,
            &textures,
        );

        let texture_output_path = args.output.join(format!("{}_{}.png", file_stem, index));
        image::save_buffer(
            texture_output_path,
            &texture.data,
            texture.width as _,
            texture.height as _,
            image::ColorType::Rgba8,
        )?;
    }

    Ok(())
}
