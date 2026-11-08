//! upgreek's capitals and var forms (#168): upright Greek letters, each drawn by the font.

use mathra::tree::{Node, Variant};
use mathra::{convert, font, Options};

/// upgreek.dtx: the capitals and the var forms; `\upvarrho` and `\upvarsigma` are `\let` to `\uprho` and
/// `\upsigma`. `\upomicron` comes from MathJax's upgreek extension.
const LETTERS: &[(&str, char)] = &[
    ("Upgamma", 'Γ'),
    ("Updelta", 'Δ'),
    ("Uptheta", 'Θ'),
    ("Uplambda", 'Λ'),
    ("Upxi", 'Ξ'),
    ("Uppi", 'Π'),
    ("Upsigma", 'Σ'),
    ("Upupsilon", 'Υ'),
    ("Upphi", 'Φ'),
    ("Uppsi", 'Ψ'),
    ("Upomega", 'Ω'),
    ("upvarepsilon", 'ε'),
    ("upvartheta", 'ϑ'),
    ("upvarpi", 'ϖ'),
    ("upvarrho", 'ρ'),
    ("upvarsigma", 'σ'),
    ("upvarphi", 'φ'),
    ("upomicron", 'ο'),
];

/// The node an equation of one command reads as, out of its row.
fn only(node: &Node) -> &Node {
    match node {
        Node::Row { children } if children.len() == 1 => only(&children[0]),
        other => other,
    }
}

fn strict() -> Options {
    Options { strict: true, ..Options::default() }
}

#[test]
fn every_letter_is_upright_greek_in_strict_mode() {
    for &(name, letter) in LETTERS {
        let tex = format!("\\{name}");
        let out = convert(&tex, &strict()).unwrap_or_else(|e| panic!("{tex}: {e}"));
        match only(&out.tree) {
            Node::Identifier { name: drawn, variant } => {
                assert_eq!(drawn.chars().collect::<Vec<_>>(), vec![letter], "{tex}");
                assert_eq!(*variant, Variant::Upright, "{tex}");
            }
            other => panic!("{tex}: {other:?}"),
        }
        let tolerant = convert(&tex, &Options::default()).unwrap();
        assert!(tolerant.warnings.is_empty(), "{tex}: {:?}", tolerant.warnings);
        assert!(font::font().glyph(letter).is_some(), "{tex}: STIX Two Math has no {letter}");
    }
}

#[test]
fn the_issue_examples_read_without_warnings() {
    for tex in [r"\Updelta E = 0", r"\upvarepsilon_0", r"\Upomega = \Upgamma \upvarphi"] {
        let out = convert(tex, &Options::default()).unwrap();
        assert!(out.warnings.is_empty(), "{tex}: {:?}", out.warnings);
        convert(tex, &strict()).unwrap_or_else(|e| panic!("{tex}: {e}"));
    }
    // upright, not the italic of \Delta in \varDelta
    assert_eq!(convert(r"\Updelta", &Options::default()).unwrap().text, convert(r"\Delta", &Options::default()).unwrap().text);
}
