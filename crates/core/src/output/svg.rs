//! SVG with a contract (spec/svg.md): outlines plus speech, source, tree and a text layer.

use std::collections::HashSet;
use std::fmt::Write;

use crate::font::{font, NAME};
use crate::layout::{Item, MathBox};
use crate::locale::Locale;
use crate::VERSION;

pub struct Content<'a> {
    pub math_box: &'a MathBox,
    pub source: &'a str,
    pub tree_json: &'a str,
    pub speech: &'a str,
    pub text: &'a str,
    pub locale: Locale,
    pub warnings: &'a [crate::Applied],
}

fn n(v: f32) -> String {
    let rounded = (v * 1000.0).round() / 1000.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded}")
    }
}

pub fn generate(c: &Content) -> String {
    let f = font();
    let em = f.units_per_em();
    let math_box = c.math_box;
    let total_height = math_box.height + math_box.depth;
    let id = id_prefix(c);
    let mut s = String::new();

    // S-SVG-1: size in em and alignment on the baseline
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:mathra="urn:mathra:1" viewBox="0 {} {} {}" width="{}em" height="{}em" style="vertical-align: {}em" role="img" aria-labelledby="{id}-speech" lang="{}" data-mathra-version="{}">"#,
        n(-math_box.height),
        n(math_box.width),
        n(total_height),
        n(math_box.width / em),
        n(total_height / em),
        n(-math_box.depth / em),
        c.locale,
        VERSION,
        id = id,
    );

    // S-SVG-2: speech for screen readers
    let _ = write!(s, r#"<title id="{id}-speech">{}</title>"#, escape(c.speech));

    // S-SVG-3: source and tree, to recover the equation
    let _ = write!(
        s,
        r#"<metadata><mathra:source type="latex">{}</mathra:source><mathra:tree>{}</mathra:tree><mathra:font>{}</mathra:font>{}</metadata>"#,
        escape(c.source),
        escape(c.tree_json),
        NAME,
        c.warnings.iter().map(|a| format!(r#"<mathra:warning rule="{}">{}</mathra:warning>"#, a.rule, escape(&a.description))).collect::<String>(),
    );

    // S-SVG-4: color. Embedded in a page, it inherits the text color (currentColor).
    // As a standalone document (<img>, opened file), svg:root matches and uses a fixed dark ink:
    // the image does not know the page theme, and following the system theme vanishes on a light page.
    // A <style> inside an inline SVG applies to the whole page: hence nothing beyond svg:root.
    s.push_str("<style>svg:root{color:#111}</style>");

    // S-SVG-5: each glyph drawn only once, in order of first appearance (deterministic)
    s.push_str("<defs>");
    let mut seen = HashSet::new();
    for item in &math_box.items {
        if let Item::Glyph { id: g, .. } = item {
            if seen.insert(g.0) {
                let _ = write!(s, r#"<path id="{id}-g{}" d="{}"/>"#, g.0, f.outline(*g));
            }
        }
    }
    s.push_str("</defs>");

    s.push_str(r#"<g fill="currentColor" stroke="none" transform="scale(1,-1)">"#);
    for item in &math_box.items {
        match item {
            Item::Glyph { id: g, x, y, scale, color } => {
                let transform = if (*scale - 1.0).abs() < 1e-4 {
                    format!(r#"x="{}" y="{}""#, n(*x), n(*y))
                } else {
                    format!(r#"transform="translate({} {}) scale({})""#, n(*x), n(*y), n(*scale))
                };
                let _ = write!(s, r##"<use href="#{id}-g{}" {transform}{}/>"##, g.0, fill(color));
            }
            Item::Rule { x, y, width, height, color } => {
                let _ = write!(s, r#"<rect x="{}" y="{}" width="{}" height="{}"{}/>"#, n(*x), n(*y), n(*width), n(*height), fill(color));
            }
            Item::Line { x1, y1, x2, y2, thickness, color } => {
                let _ = write!(
                    s,
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" stroke-linecap="round"/>"#,
                    n(*x1),
                    n(*y1),
                    n(*x2),
                    n(*y2),
                    color.as_deref().unwrap_or("currentColor"),
                    n(*thickness)
                );
            }
            Item::Ellipse { cx, cy, rx, ry, thickness, color } => {
                let _ = write!(
                    s,
                    r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
                    n(*cx),
                    n(*cy),
                    n(*rx),
                    n(*ry),
                    color.as_deref().unwrap_or("currentColor"),
                    n(*thickness)
                );
            }
        }
    }
    s.push_str("</g>");

    // S-SVG-6: transparent text layer, to select and copy when the SVG is inline in the page
    let _ = write!(
        s,
        r#"<text x="0" y="0" font-size="{}" textLength="{}" lengthAdjust="spacingAndGlyphs" fill="transparent" aria-hidden="true">{}</text>"#,
        n(em),
        n(math_box.width),
        escape(c.text),
    );

    s.push_str("</svg>");
    s
}

/// S-SVG-7: unique ids per equation, so several equations on one page do not clash.
/// Derived from the input (FNV-1a), so the same equation always gets the same ids (D-DET-1).
fn id_prefix(c: &Content) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in c.source.bytes().chain(c.locale.code().bytes()).chain(VERSION.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("mathra-{:08x}", h as u32)
}

fn fill(color: &Option<String>) -> String {
    match color {
        Some(c) => format!(r#" fill="{}""#, escape(c)),
        None => String::new(),
    }
}

fn escape(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            // XML 1.0 cannot represent these, not even escaped (S-SVG-8)
            '\t' | '\n' | '\r' => output.push(c),
            '\u{0}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}' => output.push('\u{FFFD}'),
            other => output.push(other),
        }
    }
    output
}
