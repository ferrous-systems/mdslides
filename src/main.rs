//! # mdslides - a simple Markdown to Slide converter
//!
//! Written by Jonathan Pallant at Ferrous Systems

use std::io::prelude::*;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

/// Simple program convert an mdbook to a slide deck
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The mdbook to process
    #[arg(long)]
    mdbook_path: Option<PathBuf>,

    /// The output directory
    #[arg(long)]
    output_dir: PathBuf,

    /// The HTML Template
    #[arg(long)]
    template: PathBuf,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    log::debug!("Args: {:?}", args);

    let root_path = args.mdbook_path.unwrap_or_default();

    let mdbook_toml_path = {
        let mut path = root_path.clone();
        path.push("book.toml");
        path
    };

    log::info!("Loading book: {}", mdbook_toml_path.display());
    let book_config_src = std::fs::read_to_string(&mdbook_toml_path).with_context(|| {
        format!(
            "Failed to read book config from {}",
            mdbook_toml_path.display()
        )
    })?;
    let book_config: toml::Table = toml::from_str(&book_config_src)?;
    let book_config = book_config
        .get("book")
        .and_then(|t| t.as_table())
        .ok_or(anyhow::anyhow!("No book table in book config"))?;
    log::debug!("Book config:\n{:?}", book_config);

    let book_title = book_config
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or(anyhow::anyhow!("No title field in book"))?;
    let book_src = book_config
        .get("src")
        .and_then(|v| v.as_str())
        .ok_or(anyhow::anyhow!("No src field in book"))?;
    log::info!("Book title: {:?}", book_title);
    log::info!("Book src: {:?}", book_src);

    let mdbook_summary_path = {
        let mut path = root_path.join(book_src);
        path.push("SUMMARY.md");
        path
    };

    log::info!("Loading book summary: {}", mdbook_summary_path.display());
    let summary_src = std::fs::read_to_string(&mdbook_summary_path)?;
    let parser = pulldown_cmark::Parser::new(&summary_src);
    let mut in_item = false;
    let mut chapters = Vec::new();
    let mut last_link = None;
    for event in parser {
        log::trace!("Got event: {:?}", event);
        // Every link in the book looks like:
        // Got event: Start(Item)
        // Got event: Start(Link(Inline, Borrowed("./intro.md"), Borrowed("")))
        // Got event: Text(Borrowed("Introduction"))
        // Got event: End(Link(Inline, Borrowed("./intro.md"), Borrowed("")))
        // Got event: End(Item)
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item) => {
                in_item = true;
            }
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link(_kind, path, _title))
                if in_item =>
            {
                last_link = Some(path.to_string());
            }
            pulldown_cmark::Event::Text(title) if last_link.is_some() => {
                let chapter = (last_link.take().unwrap(), title.to_string());
                chapters.push(chapter);
            }
            pulldown_cmark::Event::End(pulldown_cmark::Tag::Link(_kind, _path, _title))
                if in_item =>
            {
                last_link = None;
            }
            pulldown_cmark::Event::End(pulldown_cmark::Tag::Item) => {
                in_item = false;
            }
            _ => {
                // ignore everything else
            }
        }
    }
    log::info!("Loading template: {}", args.template.display());
    let template = std::fs::read_to_string(&args.template)?;

    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Creating {}", args.output_dir.display()))?;

    for (chapter_path, title) in chapters.iter() {
        log::info!("Processing {}: {:?}", chapter_path, title);
        let in_path = {
            let mut path = root_path.join(book_src);
            path.push(chapter_path);
            path
        };
        let out_path = {
            let mut path = args.output_dir.clone();
            let new_filename = chapter_path.replace("md", "html");
            path.push(new_filename);
            path
        };
        process(&in_path, &out_path, &template, title)?;
    }

    log::info!("Done!");

    Ok(())
}

/// Processes a markdown file into an HTML document, using the given template.
///
/// The template should contain the string `$TITLE`, which is the title of the
/// chapter, and `$CONTENT` which will be the Markdown slide contents.
fn process(in_path: &Path, out_path: &Path, template: &str, title: &str) -> std::io::Result<()> {
    log::debug!(
        "in_path: {:?}, out_path: {:?}, title: {:?}",
        in_path,
        out_path,
        title
    );

    let generated = template.replace("$TITLE", title);

    let content = std::fs::read_to_string(in_path)?;

    let generated = generated.replace("$CONTENT", &content);

    let mut output_file = std::fs::File::create(out_path)?;

    let mut first = true;
    for line in generated.lines() {
        if line.starts_with("## ") || line.starts_with("# ") {
            // Don't put a --- before the first heading, as it's our first slide
            if !first {
                writeln!(output_file, "---")?;
            } else {
                first = false;
            }
        }
        writeln!(output_file, "{}", line)?;
    }

    Ok(())
}
