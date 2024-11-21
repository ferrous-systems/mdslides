//! # mdslides - a simple Markdown to Slide converter
//!
//! Written by Jonathan Pallant at Ferrous Systems

use std::io::prelude::*;
use std::path::Path;

/// Describes the ways in which this library can fail.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Found a # heading with no text after")]
    MissingHeading,
    #[error("Found a ## heading with no text after")]
    MissingSubheading,
    #[error("I/O Error {0}")]
    Io(#[from] std::io::Error),
    #[error("No src field in book")]
    NoSrcField,
    #[error("No title field in book")]
    NoTitleField,
    #[error("No book table in book config")]
    NoBookTable,
    #[error("Invalid toml input file")]
    FormatError(#[from] toml::de::Error),
}

/// Represents an entry in the index page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexEntry {
    /// A `#` or `<h1>` heading
    Heading(String),
    /// A `##` or `<h2>` heading
    SubHeading(String),
    /// A chapter, with a link title and a file path
    Chapter { title: String, path: String },
}

/// Generate a slide deck from an mdbook.
///
/// * `mdbook_path` - the location of the mdbook's `book.toml`. Assumed to be
///   the current directory if not given.
/// * `output_dir` - where to write the HTML slides
/// * `slide_template` - a template for the slide decks, containing `$VARIABLES` for
///   substitution
/// * `index_template` - a template for the index file, containing `$VARIABLES`
///   for substitution
pub fn run(
    mdbook_path: Option<&Path>,
    output_dir: &Path,
    slide_template: &str,
    index_template: Option<&str>,
) -> Result<(), Error> {
    let mdbook_path = mdbook_path.unwrap_or_else(|| Path::new("."));

    let mdbook_toml_path = {
        let mut path = mdbook_path.to_owned();
        path.push("book.toml");
        path
    };

    log::info!("Loading book: {}", mdbook_toml_path.display());
    let book_config_src = std::fs::read_to_string(&mdbook_toml_path)?;
    let book_config: toml::Table = toml::from_str(&book_config_src)?;
    let book_config = book_config
        .get("book")
        .and_then(|t| t.as_table())
        .ok_or(Error::NoBookTable)?;
    log::debug!("Book config:\n{:?}", book_config);

    let book_title = book_config
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or(Error::NoTitleField)?;
    let book_src = book_config
        .get("src")
        .and_then(|v| v.as_str())
        .ok_or(Error::NoSrcField)?;
    log::info!("Book title: {:?}", book_title);
    log::info!("Book src: {:?}", book_src);

    let mdbook_summary_path = {
        let mut path = mdbook_path.join(book_src);
        path.push("SUMMARY.md");
        path
    };

    log::info!("Loading book summary: {}", mdbook_summary_path.display());
    let summary_src = std::fs::read_to_string(&mdbook_summary_path)?;
    let index_entries = load_book(&summary_src)?;

    std::fs::create_dir_all(output_dir)?;

    // Process each chapter
    for entry in index_entries.iter() {
        match entry {
            IndexEntry::Heading(_heading) => {
                // Ignore
            }
            IndexEntry::SubHeading(_heading) => {
                // Ignore
            }
            IndexEntry::Chapter { title, path } if path.is_empty() => {
                log::info!("Processing placeholder: {:?}", title);
            }
            IndexEntry::Chapter { title, path } => {
                log::info!("Processing {}: {:?}", path, title);
                let in_path = {
                    let mut temp_path = mdbook_path.join(book_src);
                    temp_path.push(path);
                    temp_path
                };
                let out_path = {
                    let mut temp_path = output_dir.to_owned();
                    let new_filename = path.replace("md", "html");
                    temp_path.push(new_filename);
                    temp_path
                };
                generate_deck(&in_path, &out_path, slide_template, title)?;
            }
        }
    }

    // Generate index page
    if let Some(index_template) = index_template {
        log::info!("Processing index");
        let out_path = {
            let mut path = output_dir.to_owned();
            path.push("index.html");
            path
        };
        let mut output = std::fs::File::create(out_path)?;
        generate_index(&index_entries, &mut output, index_template, book_title)?;
    }

    log::info!("Done!");

    Ok(())
}

/// Load an mdbook summary file into a list of index entries.
pub fn load_book(summary_src: &str) -> Result<Vec<IndexEntry>, Error> {
    let mut parser = pulldown_cmark::Parser::new(summary_src);
    let mut in_item = false;
    let mut index_entries = Vec::new();
    let mut last_link = None;
    loop {
        let Some(event) = parser.next() else {
            break;
        };
        log::trace!("Got event: {:?}", event);
        // Every link in the book looks like:
        // Got event: Start(Item)
        // Got event: Start(Link(Inline, Borrowed("./intro.md"), Borrowed("")))
        // Got event: Text(Borrowed("Introduction"))
        // Got event: End(Link(Inline, Borrowed("./intro.md"), Borrowed("")))
        // Got event: End(Item)
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading(
                pulldown_cmark::HeadingLevel::H1,
                _fragment,
                _classes,
            )) => {
                let Some(pulldown_cmark::Event::Text(content)) = parser.next() else {
                    panic!("Found heading with no content");
                };
                // Don't push two headings in a row.
                if let Some(IndexEntry::Heading(_)) = index_entries.last() {
                    index_entries.pop();
                }
                index_entries.push(IndexEntry::Heading(content.to_string()));
            }
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading(
                pulldown_cmark::HeadingLevel::H2,
                _fragment,
                _classes,
            )) => {
                let Some(pulldown_cmark::Event::Text(content)) = parser.next() else {
                    panic!("Found subheading with no content");
                };
                index_entries.push(IndexEntry::SubHeading(content.to_string()));
            }
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item) => {
                in_item = true;
            }
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link(_kind, path, _title))
                if in_item =>
            {
                last_link = Some(path.to_string());
            }
            pulldown_cmark::Event::Text(title) if last_link.is_some() => {
                let index_entry = IndexEntry::Chapter {
                    title: title.to_string(),
                    path: last_link.take().unwrap(),
                };
                index_entries.push(index_entry);
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
    Ok(index_entries)
}

/// Processes a markdown file into an HTML document, using the given template.
///
/// The template should contain the string `$TITLE`, which is the title of the
/// chapter, and `$CONTENT` which will be the Markdown slide contents. We assume
/// your template has an integrated Markdown-to-HTML convertor, like reveal.js
/// does.
pub fn generate_deck(
    in_path: &Path,
    out_path: &Path,
    template: &str,
    title: &str,
) -> std::io::Result<()> {
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

    let mut collecting_diagram: Option<String> = None;
    let mut first = true;
    for line in generated.lines() {
        // Find end-of-block, in case it's the end of a diagram.
        if line == "```" {
            if let Some(diagram_str) = collecting_diagram.take() {
                // This is the end of a dot block
                log::debug!("Got graph: {:?}", diagram_str);
                log::info!(
                    "Calling graphviz to render diagram in {}",
                    in_path.display()
                );
                let diagram = graphviz_rust::exec_dot(
                    diagram_str,
                    vec![graphviz_rust::cmd::CommandArg::Format(
                        graphviz_rust::cmd::Format::Svg,
                    )],
                )
                .expect("Failed to generate graph");
                // insert the SVG in-line
                writeln!(output_file, "<figure>")?;
                output_file.write_all(&diagram)?;
                writeln!(output_file, "</figure>")?;
                // Don't emit the code fence
                continue;
            }
        }

        // Are we in a diagram?
        if let Some(graph_str) = collecting_diagram.as_mut() {
            graph_str.push_str(line);
            graph_str.push('\n');
            // Don't emit the dot code
            continue;
        }

        // starting a new diagram
        if line.starts_with("```dot") && line.contains("process") {
            collecting_diagram = Some(String::new());
            // Don't emit the code fence
            continue;
        }

        // Fixup headings into slide breaks
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

/// Processes a list of chapters into an HTML document, using the given template.
///
/// The template should contain the string `$INDEX` which is replaced with
/// a simple HTML unordered list of all the chapter headings as links.
pub fn generate_index(
    chapters: &[IndexEntry],
    output: &mut dyn std::io::Write,
    template: &str,
    title: &str,
) -> std::io::Result<()> {
    // Build chapter list as HTML
    let mut generated_html = String::new();
    let mut heading_is_open = false;
    for entry in chapters {
        match entry {
            IndexEntry::Chapter { title, path } if path.is_empty() => {
                if !heading_is_open {
                    generated_html.push_str("<ul>\n");
                    heading_is_open = true;
                }
                generated_html.push_str(&format!("<li>{}</li>\n", title));
            }
            IndexEntry::Chapter { title, path } => {
                if !heading_is_open {
                    generated_html.push_str("<ul>\n");
                    heading_is_open = true;
                }
                let new_filename = path.replace("md", "html");
                generated_html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>\n",
                    new_filename, title
                ));
            }
            IndexEntry::Heading(heading) => {
                if heading_is_open {
                    generated_html.push_str("</ul>\n");
                    heading_is_open = false;
                }
                generated_html.push_str("<h1>");
                generated_html.push_str(heading);
                generated_html.push_str("</h1>\n");
            }
            IndexEntry::SubHeading(heading) => {
                if heading_is_open {
                    generated_html.push_str("</ul>\n");
                    heading_is_open = false;
                }
                generated_html.push_str("<h2>");
                generated_html.push_str(heading);
                generated_html.push_str("</h2>\n");
            }
        }
    }
    if heading_is_open {
        generated_html.push_str("</ul>\n");
    }

    let generated = template.replace("$INDEX", &generated_html);

    let generated = generated.replace("$TITLE", title);

    output.write_all(generated.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_index() {
        let index_contents = [
            IndexEntry::Heading("Heading".to_owned()),
            IndexEntry::SubHeading("SubHeading".to_owned()),
            IndexEntry::Chapter {
                title: "Link Title".to_owned(),
                path: "./test.md".to_owned(),
            },
            IndexEntry::SubHeading("SubHeading 2".to_owned()),
            IndexEntry::Chapter {
                title: "Link Title 2".to_owned(),
                path: "./test2.md".to_owned(),
            },
            IndexEntry::Chapter {
                title: "Link Title 3".to_owned(),
                path: String::new(),
            },
        ];
        let title = "My Title";
        let template = "<title>$TITLE</title>\n$INDEX";
        let mut output = Vec::new();
        generate_index(&index_contents, &mut output, template, title).unwrap();
        let output: &str = std::str::from_utf8(&output).unwrap();
        assert_eq!(
            output,
            "<title>My Title</title>\n\
            <h1>Heading</h1>\n\
            <h2>SubHeading</h2>\n\
            <ul>\n\
            <li><a href=\"./test.html\">Link Title</a></li>\n\
            </ul>\n\
            <h2>SubHeading 2</h2>\n\
            <ul>\n\
            <li><a href=\"./test2.html\">Link Title 2</a></li>\n\
            <li>Link Title 3</li>\n\
            </ul>\n"
        );
    }

    #[test]
    fn check_book() {
        let summary_src = "\
        # Heading 0\n\
        \n\
        # Heading 1\n\
        \n\
        ## Subheading 1.1\n\
        \n\
        -   [Link 1](./link1.md)\n\
        -   [Link 2](./link2.md)\n\
        \n\
        ## Subheading 1.2\n\
        \n\
        -   [Link 3](./link3.md)\n\
        -   [Link 4](./link4.md)\n\
        \n\
        # Heading 2\n\
        \n\
        ## Subheading 2.1\n\
        \n\
        -   [Link 5](./link5.md)\n\
        -   [Link 6]()\n\
        ";
        let index_entries = load_book(summary_src).unwrap();
        assert_eq!(
            &index_entries,
            &[
                IndexEntry::Heading("Heading 1".to_string()),
                IndexEntry::SubHeading("Subheading 1.1".to_string()),
                IndexEntry::Chapter {
                    title: "Link 1".to_string(),
                    path: "./link1.md".to_string()
                },
                IndexEntry::Chapter {
                    title: "Link 2".to_string(),
                    path: "./link2.md".to_string()
                },
                IndexEntry::SubHeading("Subheading 1.2".to_string()),
                IndexEntry::Chapter {
                    title: "Link 3".to_string(),
                    path: "./link3.md".to_string()
                },
                IndexEntry::Chapter {
                    title: "Link 4".to_string(),
                    path: "./link4.md".to_string()
                },
                IndexEntry::Heading("Heading 2".to_string()),
                IndexEntry::SubHeading("Subheading 2.1".to_string()),
                IndexEntry::Chapter {
                    title: "Link 5".to_string(),
                    path: "./link5.md".to_string()
                },
                IndexEntry::Chapter {
                    title: "Link 6".to_string(),
                    path: String::new(),
                },
            ]
        );
    }
}
