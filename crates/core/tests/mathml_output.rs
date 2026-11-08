//! MathML output (spec/mathml.md): MathML Core with `intent` where the tree knows more than the
//! drawing, a `<semantics>` fallback for what MathML cannot draw (S-MML-1), and a round trip: reading
//! the output back as MathML gives the same tree.

use mathra::{convert, Input, Locale, Options};
use serde_json::Value;

fn corpus(name: &str) -> Value {
    let path = format!("{}/../../corpus/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

/// Every equation of the corpus, with its locale and notation: (where, locale, input, source).
fn equations() -> Vec<(String, Locale, Input, String)> {
    let mut out = Vec::new();
    for file in ["constructions.json", "packages.json"] {
        for pair in corpus(file).as_array().unwrap() {
            out.push((format!("{file}: {}", pair[0].as_str().unwrap()), Locale::default(), Input::Latex, pair[1].as_str().unwrap().to_string()));
        }
    }
    for section in corpus("gallery.json").as_array().unwrap() {
        for row in section["rows"].as_array().unwrap() {
            let locale = row["locale"].as_str().unwrap_or("en").parse().unwrap();
            let input = row["input"].as_str().unwrap_or("latex").parse().unwrap();
            let name = row.get("name").unwrap_or(&row["caption"]).as_str().unwrap();
            out.push((format!("gallery: {name}"), locale, input, row["tex"].as_str().unwrap().to_string()));
        }
    }
    let reference = corpus("reference.json");
    for (group, sections) in reference.as_object().unwrap() {
        for section in sections.as_array().unwrap() {
            for (i, row) in section["rows"].as_array().unwrap().iter().enumerate() {
                out.push((format!("reference {group}/{} #{i}", section["id"].as_str().unwrap()), Locale::default(), Input::Latex, row["tex"].as_str().unwrap().to_string()));
            }
        }
    }
    out
}

/// Equations whose MathML is not a fallback but does not read back to the same tree, and why.
/// The drawing is right; what is lost is structure MathML has no place for (spec/mathml.md, "Round trip").
const KNOWN: &[(&str, &str)] = &[
    // siunitx: Quantity and Unit come back as their drawing (number, space, upright letters), and the
    // `group` flag of a number under five digits draws nothing MathML can show
    (r"constructions.json: \SI (siunitx)", "siunitx quantity"),
    (r"constructions.json: \qty (siunitx)", "siunitx quantity"),
    (r"constructions.json: \ang (siunitx)", "siunitx group flag"),
    (r"packages.json: siunitx \SI", "siunitx quantity"),
    ("gallery: units", "siunitx quantity"),
    ("reference science/chemistry #9", "mhchem \\pu: siunitx quantity"),
    ("reference science/units #0", "siunitx quantity"),
    ("reference science/units #1", "siunitx quantity"),
    ("reference science/units #2", "siunitx group flag"),
    ("reference science/units #3", "siunitx group flag"),
    ("reference science/units #4", "siunitx quantity"),
    ("reference science/units #5", "siunitx group flag"),
    ("reference science/units #6", "siunitx quantity"),
    // \mathrel, \mathop…: MathML has no atom class for a group, only spacing on one <mo>
    (r"constructions.json: mathtools \DeclarePairedDelimiter", "forced class"),
    (r"constructions.json: \underset/\overset", "forced class"),
    ("reference latex/operators #8", "forced class"),
    ("reference latex/integrals #9", "forced class"),
    ("reference latex/symbols #10", "forced class"),
    ("reference science/physics #10", "forced class"),
    // an operator the author named `sen` reads back as the sine it is
    (r"constructions.json: \DeclareMathOperator", "author's function named like a known one"),
    (r"packages.json: \operatorname", "author's function named like a known one"),
    // table structure MathML has no attribute for: \intertext rows, \cline and dashed rules, outer
    // rules, \arraystretch, \multicolumn's own rules
    (r"constructions.json: \intertext", "table structure"),
    (r"constructions.json: array \multicolumn", "table structure"),
    (r"constructions.json: array \cline", "table structure"),
    ("reference latex/matrices #8", "table structure"),
    ("reference latex/matrices #9", "table structure"),
    // math inside \text: MathML text holds no math, so it comes back as text and math side by side
    (r"constructions.json: \colorbox", "math inside text"),
    (r"constructions.json: \text with math inside", "math inside text"),
    // drawn with Core's mspace and mpadded, read back as a space and as the content
    (r"constructions.json: \rule", "rule as mspace"),
    ("reference latex/display #5", "\\vcenter"),
    // the border labels become a plain table
    (r"constructions.json: \bordermatrix", "border matrix"),
];

#[test]
fn the_corpus_round_trips_through_mathml() {
    let (mut checked, mut fallbacks, mut failures) = (0, 0, Vec::new());
    for (name, locale, input, tex) in equations() {
        let options = Options { locale, input, ..Options::default() };
        let Ok(first) = convert(&tex, &options) else { continue };
        if first.mathml.contains("<semantics>") {
            fallbacks += 1;
            let note = first.applied.iter().find(|a| a.rule == "S-MML-1");
            assert!(note.is_some(), "{name}: a fallback without S-MML-1");
            eprintln!("fallback {name}: {}", note.unwrap().description.split(" has no").next().unwrap());
            continue;
        }
        checked += 1;
        let back = convert(&first.mathml, &Options { input: Input::MathMl, ..options });
        let same = back.as_ref().is_ok_and(|b| b.tree == first.tree);
        if !same {
            failures.push((name, tex, first.tree.to_json(), back.map(|b| b.tree.to_json()).unwrap_or_else(|e| e.to_string()), first.mathml));
        }
    }
    let unexpected: Vec<_> = failures.iter().filter(|f| !KNOWN.iter().any(|(k, _)| *k == f.0)).collect();
    for f in &unexpected {
        eprintln!("{}\n  tex   {}\n  tree  {}\n  back  {}\n  mml   {}\n", f.0, f.1, f.2, f.3, f.4);
    }
    eprintln!("round trip: {checked} checked, {} identical, {} known differences, {fallbacks} fallbacks", checked - failures.len(), failures.len());
    assert!(unexpected.is_empty(), "{} equations do not round-trip", unexpected.len());
    // a known difference that went away: take it off the list
    let gone: Vec<_> = KNOWN.iter().filter(|(k, _)| !failures.iter().any(|f| f.0 == *k)).collect();
    assert!(gone.is_empty(), "round-trip now: {gone:?}");
}

fn mml(tex: &str) -> String {
    convert(tex, &Options::default()).unwrap_or_else(|e| panic!("{tex}: {e}")).mathml
}

fn pt(tex: &str) -> String {
    convert(tex, &Options { locale: Locale::PtBr, ..Options::default() }).unwrap().mathml
}

#[test]
fn the_quadratic_formula_of_the_issue() {
    assert_eq!(
        mml(r"\frac{-b \pm \sqrt{b^2-4ac}}{2a}"),
        concat!(
            r#"<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mfrac>"#,
            "<mrow><mo>−</mo><mi>b</mi><mo>±</mo><msqrt><msup><mi>b</mi><mn>2</mn></msup><mo>−</mo><mn>4</mn><mi>a</mi><mi>c</mi></msqrt></mrow>",
            "<mrow><mn>2</mn><mi>a</mi></mrow></mfrac></math>"
        )
    );
}

#[test]
fn display_and_inline() {
    let inline = convert("x", &Options { display: false, ..Options::default() }).unwrap().mathml;
    assert!(inline.starts_with(r#"<math xmlns="http://www.w3.org/1998/Math/MathML" display="inline">"#), "{inline}");
    assert!(mml("x").contains(r#"display="block""#));
}

#[test]
fn in_the_json_of_every_conversion() {
    let r = convert(r"\sqrt{x}", &Options::default()).unwrap();
    let json: Value = serde_json::from_str(&r.to_json()).unwrap();
    assert_eq!(json["mathml"], Value::String(r.mathml.clone()));
    assert!(r.mathml.contains("<msqrt><mi>x</mi></msqrt>"));
    assert_eq!(mathra::mathml(&r.tree, Locale::default(), true, ""), r.mathml);
}

#[test]
fn intent_where_the_drawing_does_not_say_it() {
    // pt-BR writes sen, tg, mdc: the core concept says which function (W3C Intent Core Concepts)
    let m = pt(r"\sen x + \tg x + \cos x + \mdc(a, b)");
    // function application, with TeX's thin space before the argument (none before a parenthesis)
    assert!(m.contains("<mi intent=\"sine\">sen</mi><mo lspace=\"0em\" rspace=\"0.1667em\">\u{2061}</mo><mi>x</mi>"), "{m}");
    assert!(m.contains("mdc</mi><mo>\u{2061}</mo><mo stretchy=\"false\">(</mo>"), "{m}");
    assert!(m.contains(r#"<mi intent="tangent">tg</mi>"#), "{m}");
    assert!(m.contains(r#"<mi intent="greatest-common-divisor">mdc</mi>"#), "{m}");
    // cos is written the same everywhere: no intent
    assert!(m.contains("<mi>cos</mi>"), "{m}");
    // English writes sin: the drawing already says it
    assert!(!mml(r"\sin x").contains("intent"));
    // L-INT-1: ]a, b[ is an open interval in pt-BR, and its bounds are the arguments
    let m = pt(r"x \in ]a, b[");
    assert!(m.contains(r#"<mrow intent="open-interval($a,$b)"><mo stretchy="false">]</mo><mi arg="a">a</mi>"#), "{m}");
    assert!(pt(r"x \in ]0, 1]").contains("open-closed-interval"));
    assert!(!mml(r"x \in ]a, b[").contains("intent"), "English has no reversed-bracket intervals");
    assert!(!pt(r"[a, b]").contains("intent"));
    // \binom draws (n over k): the concept says it is the binomial coefficient
    assert!(mml(r"\binom{n}{k}").contains(r#"<mrow intent="binomial-coefficient($n,$k)">"#));
    // siunitx units carry MathML 4's unit property
    assert!(mml(r"\SI{3}{\kilo\gram}").contains(r#"intent=":unit""#));
}

#[test]
fn only_mathml_core_elements() {
    // the elements of MathML Core (W3C, MathML Core, section 3); <menclose>, <mfenced> and the rest are not
    let core = [
        "math",
        "semantics",
        "annotation",
        "annotation-xml",
        "mrow",
        "mi",
        "mn",
        "mo",
        "mtext",
        "ms",
        "mspace",
        "mfrac",
        "msqrt",
        "mroot",
        "msub",
        "msup",
        "msubsup",
        "munder",
        "mover",
        "munderover",
        "mmultiscripts",
        "mprescripts",
        "none",
        "mtable",
        "mtr",
        "mtd",
        "mstyle",
        "mpadded",
        "mphantom",
        "merror",
        "maction",
    ];
    for (name, locale, input, tex) in equations() {
        let Ok(r) = convert(&tex, &Options { locale, input, ..Options::default() }) else { continue };
        let doc = roxmltree::Document::parse(&r.mathml).unwrap_or_else(|e| panic!("{name}: invalid XML {e}\n{}", r.mathml));
        for n in doc.descendants().filter(|n| n.is_element()) {
            assert!(core.contains(&n.tag_name().name()), "{name}: <{}> is not MathML Core", n.tag_name().name());
            assert_eq!(n.tag_name().namespace(), Some("http://www.w3.org/1998/Math/MathML"));
        }
    }
}

#[test]
fn s_mml_1_semantics_with_the_source() {
    for tex in [r"\cancel{x} + 1", r"\boxed{E = mc^2}", r"\begin{CD} A @>f>> B \end{CD}", r"\begin{tikzcd} A \arrow[r] & B \end{tikzcd}", r"\divchave{125}{5}"] {
        let r = convert(tex, &Options::default()).unwrap_or_else(|e| panic!("{tex}: {e}"));
        assert!(r.mathml.contains("<semantics><mrow>"), "{tex}: {}", r.mathml);
        let doc = roxmltree::Document::parse(&r.mathml).unwrap();
        let annotation = doc.descendants().find(|n| n.tag_name().name() == "annotation").unwrap();
        assert_eq!(annotation.attribute("encoding"), Some("application/x-tex"));
        assert_eq!(annotation.text(), Some(tex), "the source, unchanged");
        // a note, not a warning: the input has no problem (as S-BRL-1)
        assert!(r.applied.iter().any(|a| a.rule == "S-MML-1"), "{tex}");
        assert!(r.warnings.is_empty(), "{tex}: {:?}", r.warnings);
        // the closest drawing is still there
        assert!(doc.descendants().any(|n| matches!(n.tag_name().name(), "mi" | "mn" | "mtext" | "mtable")), "{tex}");
    }
    #[cfg(feature = "chem")]
    {
        let r = convert(r"\chemfig{H_3C-*6(=-=-=-)}", &Options::default()).unwrap();
        assert!(r.mathml.contains("<mtext>") && r.mathml.contains(r#"<annotation encoding="application/x-tex">"#), "{}", r.mathml);
    }
    let am = convert("cancel(x)", &Options { input: Input::AsciiMath, ..Options::default() }).unwrap();
    assert!(am.mathml.contains(r#"<annotation encoding="text/x-asciimath">cancel(x)</annotation>"#), "{}", am.mathml);
    // what MathML draws has no fallback and no note
    let plain = convert(r"\frac{1}{2}", &Options::default()).unwrap();
    assert!(!plain.mathml.contains("semantics") && plain.applied.iter().all(|a| a.rule != "S-MML-1"));
}

#[test]
fn letters_in_their_variant() {
    // MathML Core draws variants as Unicode's mathematical letters; a single letter is italic on its own
    assert_eq!(
        mml(r"\mathbf{x} \mathbb{R} \mathcal{L} x \Gamma \mathrm{d}"),
        concat!(
            r#"<math xmlns="http://www.w3.org/1998/Math/MathML" display="block">"#,
            r#"<mi>𝐱</mi><mi>ℝ</mi><mi>ℒ</mi><mi>x</mi><mi mathvariant="normal">Γ</mi><mi mathvariant="normal">d</mi></math>"#
        )
    );
}

#[test]
fn hostile_text_stays_text() {
    for tex in [
        r"\text{</mtext><script>alert(1)</script>}",
        r#"\text{&amp; "quotes" <b>}"#,
        r#"\color{red" onload="x}{a}"#,
        r"\operatorname{<i>} x",
        "a\u{0}b\u{1b}c\u{FFFF}",
        r"\cancel{x} </annotation><script>x</script>",
    ] {
        let r = convert(tex, &Options::default()).unwrap_or_else(|e| panic!("{tex:?}: {e}"));
        let doc = roxmltree::Document::parse(&r.mathml).unwrap_or_else(|e| panic!("{tex:?}: invalid XML {e}\n{}", r.mathml));
        assert!(doc.descendants().all(|n| !matches!(n.tag_name().name(), "script" | "b" | "i")), "{tex:?}: {}", r.mathml);
        assert!(doc.descendants().all(|n| n.attributes().all(|a| !a.name().starts_with("on"))), "{tex:?}");
    }
}

#[test]
fn spaces_at_the_ends_of_text_survive() {
    let m = mml(r"\text{if } x");
    assert!(m.contains("<mtext>if\u{A0}</mtext>"), "{m}");
    let back = convert(&m, &Options { input: Input::MathMl, ..Options::default() }).unwrap();
    assert_eq!(back.tree, convert(r"\text{if } x", &Options::default()).unwrap().tree);
}

#[test]
fn deep_trees_fit_on_the_stack() {
    // the writer recurses like the layout: at the reader's nesting limit it must fit where the
    // conversion fits (crossed-out content in crossed-out content is a loop, not a recursion)
    let bottom = (mathra::input::latex::NESTING_LIMIT / 2) - 2;
    let fractions = format!("{}x{}", r"\frac{".repeat(bottom), "}{y}".repeat(bottom));
    let cancels = format!("{}x{}", r"\cancel{".repeat(bottom), "}".repeat(bottom));
    let matrices = format!("{}x{}", r"\begin{matrix}".repeat(bottom / 2), r"\end{matrix}".repeat(bottom / 2));
    for tex in [fractions.clone(), cancels, matrices] {
        let r = std::thread::spawn(move || {
            let r = convert(&tex, &Options::default()).unwrap();
            let options = roxmltree::ParsingOptions { nodes_limit: u32::MAX, ..Default::default() };
            (r.mathml.len(), roxmltree::Document::parse_with_options(&r.mathml, options).is_ok())
        })
        .join();
        assert!(r.is_ok_and(|(len, valid)| len > 0 && valid));
    }
    // and the fractions read back to the same tree
    let first = convert(&fractions, &Options::default()).unwrap();
    assert_eq!(convert(&first.mathml, &Options { input: Input::MathMl, ..Options::default() }).unwrap().tree, first.tree);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn the_node_keeps_its_size() {
    // the layout and the writers recurse over Node: a bigger node is a deeper stack
    assert_eq!(std::mem::size_of::<mathra::Node>(), 176);
}
