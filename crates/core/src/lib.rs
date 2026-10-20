//! mathra: equations as a tree of meaning, drawn with explicit rules.
//!
//! ```
//! let svg = mathra::tex_to_svg("3.14").unwrap();
//! assert!(svg.contains("-speech\">3.14</title>"));
//! ```

pub mod font;
pub mod input;
pub mod layout;
pub mod locale;
pub mod output;
pub mod rules;
pub mod tree;

use std::fmt;

pub use locale::Locale;
pub use rules::Applied;
pub use tree::{Class, Node};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct Options {
    pub locale: Locale,
    /// Display equation (like `$$…$$`), or inline with the text (like `$…$`).
    pub display: bool,
    /// Refuse invalid LaTeX, like TeX does. Without it, mathra recovers what it can and returns warnings.
    pub strict: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { locale: Locale::default(), display: true, strict: false }
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    pub svg: String,
    pub tree: Node,
    pub speech: String,
    pub text: String,
    /// Rules that changed something in this equation, in the order they acted.
    pub applied: Vec<Applied>,
    /// Input problems the tolerant mode worked around (empty in strict mode).
    pub warnings: Vec<Applied>,
}

impl Output {
    /// Every output as one JSON object: `svg`, `speech`, `text`, `tree`, `applied` and `warnings`, the
    /// last two as lists of `{"rule", "description"}`. What the JavaScript and Python packages return.
    pub fn to_json(&self) -> String {
        let rules = |list: &[Applied]| -> Vec<serde_json::Value> { list.iter().map(|a| serde_json::json!({ "rule": a.rule, "description": a.description })).collect() };
        let tree: serde_json::Value = serde_json::from_str(&self.tree.to_json()).expect("the tree serializes to valid JSON");
        serde_json::json!({
            "svg": self.svg,
            "speech": self.speech,
            "text": self.text,
            "tree": tree,
            "applied": rules(&self.applied),
            "warnings": rules(&self.warnings),
        })
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Syntax { column: usize, message: String },
    MissingGlyph { character: char },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Syntax { column, message } => write!(f, "column {column}: {message}"),
            Error::MissingGlyph { character } => {
                write!(f, "the {} font has no '{character}' character (U+{:04X})", font::NAME, *character as u32)
            }
        }
    }
}

impl std::error::Error for Error {}

pub fn convert(tex: &str, options: &Options) -> Result<Output, Error> {
    let mut registry = rules::Registry::default();
    let tree = input::latex::read_with(tex, options.locale, &mut registry, options.strict)?;
    let style = if options.display { layout::Style::display() } else { layout::Style::text() };
    let math_box = {
        let mut ctx = layout::Context { locale: options.locale, registry: &mut registry, strict: options.strict };
        layout::compose(&tree, style, &mut ctx)?
    };
    let speech = output::speech::speak(&tree, options.locale);
    let text = output::text::linear(&tree, options.locale);
    let tree_json = tree.to_json();
    registry.dedup();
    let svg = output::svg::generate(&output::svg::Content {
        math_box: &math_box,
        source: tex,
        tree_json: &tree_json,
        speech: &speech,
        text: &text,
        locale: options.locale,
        warnings: &registry.warnings,
    });
    Ok(Output { svg, tree, speech, text, applied: registry.applied, warnings: registry.warnings })
}

/// The composed box, without generating SVG: for callers that draw the equation inside something
/// else (tikra's figure labels). Also returns the tolerant mode's warnings.
pub fn typeset(tex: &str, options: &Options) -> Result<(layout::MathBox, Vec<Applied>), Error> {
    let mut registry = rules::Registry::default();
    let tree = input::latex::read_with(tex, options.locale, &mut registry, options.strict)?;
    let style = if options.display { layout::Style::display() } else { layout::Style::text() };
    let mut ctx = layout::Context { locale: options.locale, registry: &mut registry, strict: options.strict };
    let math_box = layout::compose(&tree, style, &mut ctx)?;
    registry.dedup();
    Ok((math_box, registry.warnings))
}

/// Shortcut with the default locale (English).
pub fn tex_to_svg(tex: &str) -> Result<String, Error> {
    convert(tex, &Options::default()).map(|r| r.svg)
}
