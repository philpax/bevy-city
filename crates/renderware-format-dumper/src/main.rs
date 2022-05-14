use std::path::PathBuf;

use clap::Parser;
use renderware_format as rwf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
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
    let file = rwf::raw::BinaryStreamFile::open(args.path)?;
    print_section(&file.sections[0], 0);
    Ok(())
}
