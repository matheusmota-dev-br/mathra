//! One test per rule in spec/rules.md and spec/svg.md. The test name starts with the rule id.

use mathra::{convert, Class, Locale, Node, Options};

fn pt(tex: &str) -> mathra::Output {
    convert(tex, &Options { locale: Locale::PtBr, ..Default::default() }).unwrap()
}

fn en(tex: &str) -> mathra::Output {
    convert(tex, &Options { locale: Locale::En, ..Default::default() }).unwrap()
}

fn number(value: &str) -> Node {
    Node::Number { value: value.into(), group: false, thousands: None }
}

fn rules(r: &mathra::Output) -> Vec<&'static str> {
    r.applied.iter().map(|a| a.rule).collect()
}

#[test]
fn l_num_1_comma_between_digits_is_decimal_in_pt_br() {
    let r = pt("3,14");
    assert_eq!(r.tree.children(), &[number("3.14")]);
    assert!(rules(&r).contains(&"L-NUM-1"));
}

#[test]
fn l_num_1_comma_followed_by_space_stays_separator() {
    let r = pt("f(1, 5)");
    let children = r.tree.children();
    assert_eq!(children[2], number("1"));
    assert_eq!(children[3], Node::Operator { symbol: ',', class: Class::Punct });
    assert_eq!(children[4], number("5"));
}

#[test]
fn l_num_1_in_english_comma_is_never_decimal() {
    let r = en("3,14");
    assert_eq!(r.tree.children()[0], number("3"));
    assert_eq!(r.tree.children()[2], number("14"));
}

#[test]
fn l_num_1_dot_is_decimal_in_both_locales() {
    assert_eq!(pt("3.14").tree.children(), &[number("3.14")]);
    assert_eq!(en("3.14").tree.children(), &[number("3.14")]);
}

#[test]
fn l_num_2_locale_decides_how_the_same_number_is_written() {
    assert_eq!(pt("3.14").text, "3,14");
    assert_eq!(en("3.14").text, "3.14");
    assert!(rules(&pt("3.14")).contains(&"L-NUM-2"));
}

#[test]
fn l_num_1_and_2_comma_in_braces_is_ordinary_as_in_tex() {
    // {,} is not a number, but it does not get punctuation spacing either
    let r = en("3{,}14");
    assert_eq!(r.tree.children()[1], Node::Operator { symbol: ',', class: Class::Ord });
    assert_eq!(r.text, "3,14");
}

#[test]
fn l_fun_1_accepts_brazilian_name_and_writes_by_locale() {
    assert_eq!(pt(r"\sen x").tree.children()[0], Node::Function { name: "sin".into(), limits: false });
    assert_eq!(pt(r"\sin x").text, "sen x");
    assert_eq!(en(r"\sen x").text, "sin x");
    assert_eq!(pt(r"\tg x").speech, "tangente de x");
}

#[test]
fn l_esp_2_leading_minus_is_unary() {
    let r = pt("-b + c");
    assert!(rules(&r).contains(&"L-ESP-2"));
    assert_eq!(r.text, "−b + c");
}

#[test]
fn l_esp_2_minus_after_relation_is_unary() {
    assert_eq!(pt("x = -1").text, "x = −1");
}

#[test]
fn l_esp_1_relation_has_more_space_than_operator() {
    // the width grows more with '=' (5 mu on both sides) than with '+' (4 mu)
    let width = |tex: &str| {
        let svg = pt(tex).svg;
        let start = svg.find("viewBox=\"0 ").unwrap() + 11;
        let fields: Vec<f32> = svg[start..].split('"').next().unwrap().split(' ').map(|v| v.parse().unwrap()).collect();
        fields[1]
    };
    let with_rel = width("a=b");
    let with_bin = width("a+b");
    // the glyphs have similar widths; a 2 mu difference = 1000/18*2 ≈ 111 units
    assert!(with_rel > with_bin, "rel {with_rel} should be wider than bin {with_bin}");
}

#[test]
fn l_let_1_letters_in_math_italic() {
    assert_eq!(mathra::layout::math_italic('x'), '𝑥');
    assert_eq!(mathra::layout::alphabet::character('R', mathra::tree::Variant::DoubleStruck), 'ℝ');
    assert_eq!(mathra::layout::math_italic('h'), 'ℎ');
    assert_eq!(mathra::layout::math_italic('θ'), '𝜃');
    assert_eq!(mathra::layout::math_italic('2'), '2');
}

#[test]
fn s_svg_2_portuguese_speech_in_title() {
    let r = pt(r"x \le 2,5");
    assert_eq!(r.speech, "x é menor ou igual a 2,5");
    assert!(r.svg.contains(">x é menor ou igual a 2,5</title>"));
    assert!(r.svg.contains(r#"role="img""#));
}

#[test]
fn s_svg_3_source_recoverable_from_metadata() {
    let svg = pt(r"a < b").svg;
    assert!(svg.contains(r#"<mathra:source type="latex">a &lt; b</mathra:source>"#));
    assert!(svg.contains("<mathra:tree>"));
}

#[test]
fn s_svg_4_style_does_not_leak_into_the_page() {
    let svg = pt("x").svg;
    let style = svg.split("<style>").nth(1).unwrap().split("</style>").next().unwrap();
    assert_eq!(style, "svg:root{color:#111}");
}

#[test]
fn s_svg_5_repeated_glyph_is_drawn_once() {
    let svg = pt("1111").svg;
    assert_eq!(svg.matches("<path ").count(), 1);
    assert_eq!(svg.matches("<use ").count(), 4);
}

#[test]
fn s_svg_6_transparent_text_layer() {
    let svg = pt("3,14 + x").svg;
    assert!(svg.contains(r#"fill="transparent" aria-hidden="true">3,14 + x</text>"#));
}

#[test]
fn s_svg_7_different_ids_for_different_equations() {
    let id = |svg: &str| svg.split("aria-labelledby=\"").nth(1).unwrap().split('"').next().unwrap().to_string();
    assert_ne!(id(&pt("x").svg), id(&pt("y").svg));
    assert_ne!(id(&pt("x").svg), id(&en("x").svg));
}

#[test]
fn d_det_1_same_input_same_bytes() {
    let a = pt(r"f(1, 5) = \sen x - 2,5").svg;
    let b = pt(r"f(1, 5) = \sen x - 2,5").svg;
    assert_eq!(a, b);
}

#[test]
fn error_points_to_column() {
    let strict = Options { strict: true, ..Default::default() };
    let e = convert(r"x + \foo", &strict).unwrap_err();
    assert_eq!(e.to_string(), r"column 5: unsupported command: \foo");
    let e = convert("{x", &strict).unwrap_err();
    assert_eq!(e.to_string(), "column 1: '{' without a matching '}'");
}

#[test]
fn l_txt_1_tex_quotes_become_typographic_in_text() {
    let r = en(r"\text{``quoted'' and `single' don't}");
    assert_eq!(r.text, "“quoted” and ‘single’ don’t");
    assert!(rules(&r).contains(&"L-TXT-1"));
    // in math mode ' is still a prime
    let r = en("f'");
    assert!(!rules(&r).contains(&"L-TXT-1"));
    assert!(r.speech.contains("prime"), "{}", r.speech);
}
