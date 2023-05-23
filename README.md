# mdslides

A tool for turning mdbooks into slide shows.

Written by [Ferrous Systems](https://www.ferrous-systems.com). Contact us for Rust training, or help with your next Rust project.

## Installation

The crate is built with cargo-dist. You can grab binaries from the release area on Github: https://github.com/ferrous-systems/mdslides/releases

## Usage

Run the tool, passing the source of your `mdbook` of slides, and a template HTML file:

```console
$ mdslides --mdbook-path ~/Documents/my-slides --output-dir ./html --template ~/Documents/my-slides/template.html
```

It will create a new HTML file for every chapter in your `mdbook`. Each HTML file will be a copy of the template, but with the string `$TITLE` replaced with the title of the chapter, and the string `$CONTENT` replaced with the Markdown source of that chapter. Additionally, each `# Heading` or `## Subheading` in the Markdown will have an `---` divider added before it. The reveal.js framework uses this to indicate when a new page is required, so each heading them forms a new slide.

You can also pass `--index-template ./index-template.html` and a file called `${OUTPUT_DIR}/index.html` will be created using that template, replacing `$INDEX` with a series of HTML headings, subheadings and links to each slide deck.

You can see an example of using this tool at <https://github.com/ferrous-systems/material-templates>.

## License

This crate is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](./LICENSE-APACHE), [LICENSE-MIT](./LICENSE-MIT), and [COPYRIGHT](./COPYRIGHT) for details.


