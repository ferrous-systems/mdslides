//! Our Syntax Highlighting code,
//!
//! A comrak plugin for adding syntax highlighting classes to code blocks.

pub struct SyntaxHighlighter();

impl SyntaxHighlighter {
    pub fn new() -> SyntaxHighlighter {
        SyntaxHighlighter()
    }
}

impl comrak::adapters::SyntaxHighlighterAdapter for SyntaxHighlighter {
    fn write_pre_tag(
        &self,
        output: &mut dyn std::io::Write,
        attributes: std::collections::HashMap<String, String>,
    ) -> std::io::Result<()> {
        println!("Pre Attr: {:?}", attributes);
        writeln!(output, "<pre>")?;
        Ok(())
    }

    fn write_code_tag(
        &self,
        output: &mut dyn std::io::Write,
        attributes: std::collections::HashMap<String, String>,
    ) -> std::io::Result<()> {
        println!("Code Attr: {:?}", attributes);
        let mut lang_class = "";
        if let Some(class) = attributes.get("class") {
            if let Some(lang) = class.strip_prefix("language-") {
                lang_class = lang;
            }
        }
        writeln!(
            output,
            r#"<code data-trim data-noescape class="{lang_class}">"#
        )?;
        Ok(())
    }

    fn write_highlighted(
        &self,
        output: &mut dyn std::io::Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::io::Result<()> {
        println!("Got lang {:?}", lang);
        writeln!(output, "{}", code)?;
        Ok(())
    }
}
