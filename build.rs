// build.rs

use clap::CommandFactory;
use clap_mangen::Man;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

// Import the CLI definition from src/cli.rs
include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let man_dir = Path::new(&out_dir).join("man-pages");
    fs::create_dir_all(&man_dir)?;

    let cmd = Cli::command();

    let man = Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let mut man_file = File::create(man_dir.join("marty.1"))?;
    man_file.write_all(&buffer)?;

    println!("cargo:info=Man page generated at {}", man_dir.display());

    Ok(())
}
