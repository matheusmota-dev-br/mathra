//! Hostile input versus the output: the SVG must stay valid XML that is safe to embed,
//! whatever comes in the source.

use mathra::{convert, Options};

fn valid_svg(tex: &str) -> String {
    let r = convert(tex, &Options::default()).unwrap_or_else(|e| panic!("{tex:?}: {e}"));
    if let Err(e) = roxmltree::Document::parse(&r.svg) {
        panic!("{tex:?} produced invalid XML: {e}");
    }
    r.svg
}

#[test]
fn markup_in_source_does_not_become_markup() {
    for tex in [
        r"\text{</title><script>alert(1)</script>}",
        r"\text{]]>}",
        r"a --> b <!-- c",
        r#"\text{&amp; "aspas" 'simples'}"#,
        r"\text{<?xml version='1.0'?>}",
        r#"\color{red" onload="x}{a}"#,
    ] {
        let svg = valid_svg(tex);
        let doc = roxmltree::Document::parse(&svg).unwrap();
        assert!(doc.descendants().all(|n| n.tag_name().name() != "script"), "{tex}");
        assert!(doc.descendants().all(|n| n.attributes().all(|a| !a.name().starts_with("on"))), "{tex}");
    }
}

#[test]
fn characters_forbidden_in_xml() {
    // XML 1.0 accepts no control characters (except tab, LF, CR), nor U+FFFE/U+FFFF, not even escaped
    for tex in ["a\u{0}b", "a\u{1}b", "x\u{8}y", "x\u{1b}[31my", "a\u{7f}b", "a\u{FFFE}b", "a\u{FFFF}b", "a\rb", "a\tb"] {
        valid_svg(tex);
    }
}

#[test]
fn assorted_unicode_in_source() {
    for tex in ["😀", "x + 😀", "\u{202E}abc", "a\u{0301}", "\u{200B}x", "مرحبا", "日本語", "\u{1F600}^{\u{1F600}}"] {
        valid_svg(tex);
    }
}

#[test]
fn long_source_stays_valid() {
    let tex = "x+".repeat(20_000) + "x";
    let svg = valid_svg(&tex);
    assert!(svg.len() > tex.len());
}

#[test]
fn unique_ids_across_different_equations() {
    let a = valid_svg("x");
    let b = valid_svg("y");
    let id = |s: &str| s.split("id=\"").nth(1).unwrap().split('"').next().unwrap().to_string();
    assert_ne!(id(&a), id(&b));
    assert_eq!(id(&a), id(&valid_svg("x")), "same input, same id");
}
