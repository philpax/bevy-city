use std::path::PathBuf;

use clap::{ArgEnum, Parser};
use renderware_format as rwf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Format {
    Png,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The format to convert to
    #[clap(arg_enum, short, long)]
    format: Format,

    /// Path of the file to convert
    #[clap()]
    path: PathBuf,

    /// Where to deposit the converted files
    #[clap()]
    output_directory: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    std::fs::create_dir_all(&args.output_directory)?;

    if args.path.extension().unwrap_or_default() != "txd" {
        unimplemented!(
            "renderware-format-converter does not support anything other than txd at this time"
        );
    }

    let extension = match args.format {
        Format::Png => "png",
    };

    let textures = rwf::txd::Texture::from_raw(&rwf::raw::BinaryStreamFile::open(&args.path)?);
    for texture in &textures {
        image::save_buffer(
            args.output_directory
                .join(&texture.name)
                .with_extension(extension),
            &texture.data,
            texture.width as u32,
            texture.height as u32,
            image::ColorType::Rgba8,
        )?;
    }

    Ok(())
}
