//! Our Slide Adapter
//!
//! A comrak plugin for splitting up plain markdown text into reveal.js slides
//! by adding `<section>` tags around each slide (as delimited by `h1` or `h3`
//! heading).

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// Adds `<section>` around <h1> and <h2> slides.
pub struct SlideAdapter {
    first: AtomicBool,
    no_heading_slide: AtomicBool,
    in_speaker_notes: AtomicBool,
}

impl SlideAdapter {
    pub fn new() -> SlideAdapter {
        SlideAdapter {
            first: AtomicBool::new(true),
            no_heading_slide: AtomicBool::new(false),
            in_speaker_notes: AtomicBool::new(false),
        }
    }
}

impl comrak::adapters::HeadingAdapter for SlideAdapter {
    fn enter(
        &self,
        output: &mut dyn std::io::Write,
        heading: &comrak::adapters::HeadingMeta,
        _sourcepos: Option<comrak::nodes::Sourcepos>,
    ) -> std::io::Result<()> {
        if heading.level == 3 && heading.content == "Notes" {
            // the start of speaker notes.
            writeln!(output, r#"<aside class="notes">"#)?;
            self.in_speaker_notes.store(true, Relaxed);
        } else if heading.level <= 2 {
            // We break slides on # and ##
            if !self.first.load(Relaxed) {
                // we have a previous slide open. Close it

                // but first close any speaker notes
                if self.in_speaker_notes.load(Relaxed) {
                    self.in_speaker_notes.store(false, Relaxed);
                    writeln!(output, "</aside>")?;
                }
                // now close the slide
                writeln!(output, "</section>")?;
            } else {
                self.first.store(false, Relaxed);
            }
            // open a new slide
            writeln!(output, "<section>")?;
        }

        if heading.content == "NO_HEADING" {
            // this is a slide with no heading - comment out the fake heading
            self.no_heading_slide.store(true, Relaxed);
            writeln!(output, "<!-- ")?;
        } else {
            // write out the heading
            self.no_heading_slide.store(false, Relaxed);
            writeln!(output, "<h{}>", heading.level)?;
        }

        Ok(())
    }

    fn exit(
        &self,
        output: &mut dyn std::io::Write,
        heading: &comrak::adapters::HeadingMeta,
    ) -> std::io::Result<()> {
        if self.no_heading_slide.load(Relaxed) {
            // stop hiding the fake heading
            writeln!(output, " -->")?;
            self.no_heading_slide.store(false, Relaxed);
        } else {
            // close the heading
            writeln!(output, "</h{}>", heading.level)?;
        }
        Ok(())
    }
}
