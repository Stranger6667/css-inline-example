use cssparser::{
    AtRuleParser, BasicParseErrorKind, ParseError, ParseErrorKind, Parser, ParserInput,
    QualifiedRuleParser, RuleListParser, SourceLocation,
};
use kuchiki::{parse_html, traits::TendrilSink, NodeRef};
use std::error::Error;
use std::{fmt, io};

/// Shortcut for inlining CSS with default parameters.
pub fn inline(html: &str) -> Result<String, InlineError> {
    CSSInliner::default().inline(html)
}

#[derive(Debug)]
pub enum InlineError {
    ParseError(String),
    /// Input-output error. May happen during writing the resulting HTML.
    IO(io::Error),
}

impl From<io::Error> for InlineError {
    fn from(error: io::Error) -> Self {
        InlineError::IO(error)
    }
}

impl Error for InlineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InlineError::IO(error) => Some(error),
            InlineError::ParseError(_) => None,
        }
    }
}

impl fmt::Display for InlineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InlineError::IO(error) => write!(f, "{}", error),
            InlineError::ParseError(error) => write!(f, "{}", error),
        }
    }
}

impl From<(ParseError<'_, ()>, &str)> for InlineError {
    fn from(error: (ParseError<'_, ()>, &str)) -> Self {
        let message = match error.0.kind {
            ParseErrorKind::Basic(kind) => match kind {
                BasicParseErrorKind::UnexpectedToken(token) => {
                    format!("Unexpected token: {:?}", token)
                }
                BasicParseErrorKind::EndOfInput => "End of input".to_string(),
                BasicParseErrorKind::AtRuleInvalid(value) => format!("Invalid @ rule: {}", value),
                BasicParseErrorKind::AtRuleBodyInvalid => "Invalid @ rule body".to_string(),
                BasicParseErrorKind::QualifiedRuleInvalid => "Invalid qualified rule".to_string(),
            },
            ParseErrorKind::Custom(_) => "Never happens".to_string(),
        };
        InlineError::ParseError(message)
    }
}

#[derive(Debug)]
pub struct InlineOptions {
    pub remove_style_tags: bool,
}

impl Default for InlineOptions {
    fn default() -> Self {
        InlineOptions {
            remove_style_tags: false,
        }
    }
}

impl InlineOptions {
    pub fn remove_style_tags(mut self, remove_style_tags: bool) -> Self {
        self.remove_style_tags = remove_style_tags;
        self
    }

    pub fn build(self) -> CSSInliner {
        CSSInliner::new(self)
    }
}

#[derive(Debug)]
pub struct CSSInliner {
    options: InlineOptions,
}

impl Default for CSSInliner {
    #[inline]
    fn default() -> Self {
        CSSInliner::new(InlineOptions::default())
    }
}

impl CSSInliner {
    pub fn new(options: InlineOptions) -> Self {
        CSSInliner { options }
    }

    /// Return a default `InlineOptions` that can fully configure the CSS inliner.
    ///
    /// # Examples
    ///
    /// Get default `InlineOptions`, then change base url
    ///
    /// ```rust
    /// use css_inline_example::CSSInliner;
    /// # fn run() {
    /// let inliner = CSSInliner::options()
    ///     .remove_style_tags(true)
    ///     .build();
    /// # }
    /// # run();
    /// ```
    pub fn options() -> InlineOptions {
        InlineOptions::default()
    }

    /// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a
    /// string.
    pub fn inline(&self, html: &str) -> Result<String, InlineError> {
        let mut output = Vec::new();
        self.inline_to(html, &mut output)?;
        Ok(String::from_utf8_lossy(&output).to_string())
    }

    pub fn inline_to<W: io::Write>(&self, html: &str, target: &mut W) -> Result<(), InlineError> {
        let document = parse_html().one(html);
        for style_tag in document
            .select("style")
            .map_err(|_| InlineError::ParseError("Unknown error".to_string()))?
        {
            if let Some(first_child) = style_tag.as_node().first_child() {
                if let Some(css_cell) = first_child.as_text() {
                    process_css(&document, css_cell.borrow().as_str())?;
                }
            }
            if self.options.remove_style_tags {
                style_tag.as_node().detach()
            }
        }
        document.serialize(target)?;
        Ok(())
    }
}

fn process_css(document: &NodeRef, css: &str) -> Result<(), InlineError> {
    let mut parse_input = ParserInput::new(css);
    let mut parser = Parser::new(&mut parse_input);
    let rules = RuleListParser::new_for_stylesheet(&mut parser, CSSRuleListParser);
    for rule in rules {
        let (selector, block) = rule?;
        if let Ok(matching_elements) = document.select(selector) {
            for element in matching_elements {
                if let Ok(mut attributes) = element.attributes.try_borrow_mut() {
                    attributes.insert("style", block.to_string());
                }
            }
        }
    }
    Ok(())
}

struct CSSRuleListParser;

type QualifiedRule<'i> = (&'i str, &'i str);

fn exhaust<'i>(input: &mut Parser<'i, '_>) -> &'i str {
    let start = input.position();
    while input.next().is_ok() {}
    input.slice_from(start)
}

impl<'i> QualifiedRuleParser<'i> for CSSRuleListParser {
    type Prelude = &'i str;
    type QualifiedRule = QualifiedRule<'i>;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        Ok(exhaust(input))
    }
    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _: SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        Ok((prelude, exhaust(input)))
    }
}

impl<'i> AtRuleParser<'i> for CSSRuleListParser {
    type PreludeNoBlock = &'i str;
    type PreludeBlock = &'i str;
    type AtRule = QualifiedRule<'i>;
    type Error = ();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let html = r#"<html><head>
<style>h1 { color:blue; }</style>
        </head>
        <body><h1>Big Text</h1></body>
        </html>"#;
        let expected = r#"<html><head>
<style>h1 { color:blue; }</style>
        </head>
        <body><h1 style=" color:blue; ">Big Text</h1>
        </body></html>"#;
        let inlined = inline(html).unwrap();
        assert_eq!(inlined, expected)
    }
}
