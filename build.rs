#[path = "src/cli.rs"]
mod cli;

use std::fs;

fn main() {
    println!("cargo::rerun-if-changed=src/cli.rs");
    let markdown: String = clap_markdown::help_markdown::<cli::Cli>();
    fs::write("./Readme.md", markdown).expect("Unable to write Readme.md");
}
