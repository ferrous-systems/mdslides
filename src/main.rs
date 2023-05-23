//! # mdslides - a simple Markdown to Slide converter
//!
//! Written by Jonathan Pallant at Ferrous Systems

use std::path::PathBuf;

use clap::Parser;

static ABOUT_TEXT: &str = concat!(
    env!("CARGO_PKG_DESCRIPTION"),
    r#"

This program is licensed as "#,
    env!("CARGO_PKG_LICENSE"),
    r#", at your option.

For a list of dependencies included in this binary, refer to the source code.
The canonical source code location is "#,
    env!("CARGO_PKG_REPOSITORY"),
    r#"

You are running version "#,
    env!("CARGO_PKG_VERSION"),
    "."
);

/// Command line arguments for this program.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = ABOUT_TEXT)]
struct Args {
    /// The mdbook to process
    #[arg(long)]
    mdbook_path: Option<PathBuf>,

    /// The output directory
    #[arg(long)]
    output_dir: PathBuf,

    /// The HTML Template for the slides.
    #[arg(long)]
    template: PathBuf,

    /// The HTML Template for the index.
    #[arg(long)]
    index_template: Option<PathBuf>,
}

fn main() -> Result<(), mdslides::Error> {
    env_logger::init();
    let args = Args::parse();
    log::debug!("Args: {:?}", args);

    log::info!("Loading slide template: {}", args.template.display());
    let slide_template_string = std::fs::read_to_string(&args.template)?;

    let mut index_template_string = None;
    if let Some(index_template_path) = args.index_template {
        log::info!("Using index template: {}", index_template_path.display());
        index_template_string = Some(std::fs::read_to_string(index_template_path)?);
    }

    mdslides::run(
        args.mdbook_path.as_deref(),
        &args.output_dir,
        &slide_template_string,
        index_template_string.as_deref(),
    )
}
