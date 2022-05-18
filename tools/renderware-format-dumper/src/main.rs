use std::path::PathBuf;

use clap::{ArgEnum, Parser};
use renderware_format as rwf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Mode {
    Raw,
    Processed,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// What mode to run the program in
    #[clap(arg_enum, short, long)]
    mode: Mode,

    /// Path of the file to inspect
    #[clap()]
    path: PathBuf,
}

fn print_section(section: &rwf::raw::Section, depth: i32) {
    print!("{}", "  ".repeat(depth as usize));
    println!(
        "{:?}({:X}): {:?}",
        section.section_type, section.version, section.data
    );
    for child in &section.children {
        print_section(child, depth + 1);
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let paths = if args.path.is_dir() {
        std::fs::read_dir(&args.path)?
            .filter_map(Result::ok)
            .map(|d| d.path())
            .filter(|p| {
                let extension = p.extension().unwrap_or_default();
                extension == "dff" || extension == "txd"
            })
            .collect()
    } else {
        vec![args.path]
    };

    for path in &paths {
        if paths.len() > 1 {
            println!("{:?}", path);
        }

        let file = rwf::raw::BinaryStreamFile::open(path)?;
        match args.mode {
            Mode::Raw => print_section(&file.sections[0], 0),
            Mode::Processed => {
                let extension = path.extension().unwrap_or_default();

                if extension == "dff" {
                    println!("{:?}", rwf::dff::Model::from_raw(&file));
                } else if extension == "txd" {
                    println!("{:?}", rwf::txd::Texture::from_raw(&file));
                }
            }
        }
    }
    Ok(())
}
