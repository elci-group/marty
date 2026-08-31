mod curly_expand;
// build.rs

use clap::CommandFactory;
use clap_mangen::Man;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

// Import the CLI definition from src/cli.rs
include!("src/cli.rs");

fn __curly_original_main() -> std::io::Result<()> {
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

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    let mut positions: Vec<usize> = Vec::new();
    let mut fields: Vec<Vec<String>> = Vec::new();
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--state" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--state=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--state={}", v))
                    .collect(),
            );
            break;
        }
    }

    if fields.is_empty() || fields.iter().all(|f| f.len() <= 1) {
        __curly_original_main();
        return;
    }

    let combos = curly_expand::cartesian(&fields);
    let exe = std::env::current_exe().expect("resolve current exe");
    let mut had_failure = false;
    for combo in &combos {
        let mut new_args = raw_args.clone();
        for (slot, value) in positions.iter().zip(combo.iter()) {
            new_args[*slot] = value.clone();
        }
        let status = std::process::Command::new(&exe)
            .args(&new_args[1..])
            .status()
            .expect("failed to re-exec self");
        if !status.success() {
            had_failure = true;
        }
    }
    if had_failure {
        std::process::exit(1);
    }
}
