//! Braille output (output/braille): every expected string comes from an example of the official source,
//! cited next to it. Nemeth: BANA, *The Nemeth Braille Code for Mathematics and Science Notation* (2022).
//! CMU: *Código Matemático Unificado para a Língua Portuguesa* (MEC/SEESP, 2006), as transcribed by the
//! Instituto Benjamin Constant. See spec/braille.md.

use mathra::{convert, BrailleCode, Locale, Options};

/// The examples of each code in its language: the CMU's in pt-BR (decimal comma, thousands point).
fn braille(tex: &str, code: BrailleCode) -> String {
    let locale = if code == BrailleCode::Cmu { Locale::PtBr } else { Locale::En };
    let r = convert(tex, &Options { locale, braille: Some(code), strict: true, ..Options::default() }).unwrap_or_else(|e| panic!("{tex}: {e}"));
    assert!(r.applied.iter().all(|a| a.rule != "S-BRL-1"), "{tex}: {:?}", r.applied);
    r.braille
}

fn check(code: BrailleCode, cases: &[(&str, &str, &str)]) {
    for (tex, expected, source) in cases {
        assert_eq!(braille(tex, code), *expected, "{tex} ({source})");
    }
}

#[test]
fn nemeth_numbers_and_operations() {
    check(
        BrailleCode::Nemeth,
        &[
            ("1+x+y=0", "⠼⠂⠬⠭⠬⠽⠀⠨⠅⠀⠼⠴", "Ex. 3-14"),
            ("x = -1", "⠭⠀⠨⠅⠀⠤⠼⠂", "§3.3"),
            (".35", "⠼⠨⠒⠢", "§3"),
            ("3.14", "⠼⠒⠨⠂⠲", "§3"),
            ("(0 = x)", "⠷⠴⠀⠨⠅⠀⠭⠾", "§3.4"),
            ("y = 2 \\sin x", "⠽⠀⠨⠅⠀⠼⠆⠎⠊⠝⠀⠭", "§3.3"),
            ("x \\neq y", "⠭⠀⠌⠨⠅⠀⠽", "§21.1"),
            ("a=b", "⠁⠀⠨⠅⠀⠃", "Ex. 6-31"),
        ],
    );
}

#[test]
fn nemeth_letters_and_the_english_letter_indicator() {
    check(
        BrailleCode::Nemeth,
        &[
            ("n, s", "⠰⠝⠠⠀⠰⠎", "Ex. 6-8"),
            ("\\cos A", "⠉⠕⠎⠀⠠⠁", "Ex. 6-15"),
            ("(a, 2x, b)", "⠷⠁⠠⠀⠆⠭⠠⠀⠃⠾", "Ex. 6-23, enclosed list"),
            ("\\{a, b, c, d\\}", "⠨⠷⠁⠠⠀⠃⠠⠀⠉⠠⠀⠙⠨⠾", "Ex. 6-25"),
            ("i = 1, 2, \\ldots, n", "⠊⠀⠨⠅⠀⠼⠂⠠⠀⠼⠆⠠⠀⠄⠄⠄⠠⠀⠰⠝", "Ex. 6-29"),
            ("(a, 2x, y = z)", "⠷⠰⠁⠠⠀⠼⠆⠭⠠⠀⠽⠀⠨⠅⠀⠵⠾", "Ex. 6-30"),
            ("\\{x: x > 0\\}", "⠨⠷⠰⠭⠸⠒⠀⠭⠀⠨⠂⠀⠼⠴⠨⠾", "Ex. 8-46"),
            ("[0, 1]", "⠈⠷⠴⠠⠀⠂⠈⠾", "§3.5"),
            ("\\Sigma", "⠨⠠⠎", "§6.1.4"),
            ("\\mathbb{R}", "⠠⠸⠰⠠⠗", "Ex. 7-2"),
        ],
    );
}

#[test]
fn nemeth_fractions_levels_and_radicals() {
    check(
        BrailleCode::Nemeth,
        &[
            ("\\frac{1}{3}", "⠹⠂⠌⠒⠼", "§13"),
            ("4\\frac{3}{8}", "⠼⠲⠸⠹⠒⠌⠦⠸⠼", "§13, mixed number"),
            ("\\frac{\\frac{a}{b}}{c}", "⠠⠹⠹⠁⠌⠃⠼⠠⠌⠉⠠⠼", "§13, complex fraction"),
            ("x^2+1", "⠭⠘⠆⠐⠬⠂", "§14"),
            ("x_1", "⠭⠂", "§14.6"),
            ("x_{i,j}", "⠭⠰⠊⠪⠚", "§14"),
            ("x_a^n", "⠭⠰⠁⠘⠝", "§14.11"),
            ("x'^2", "⠭⠄⠘⠆", "§14.12"),
            ("\\cos^2 x", "⠉⠕⠎⠘⠆⠀⠭", "§18"),
            ("\\log_2 x", "⠇⠕⠛⠆⠀⠭", "§14.6"),
            ("90^\\circ", "⠼⠔⠴⠘⠨⠡", "§23.10"),
            ("\\sqrt{x+y}", "⠜⠭⠬⠽⠻", "§16"),
            ("\\sqrt[3]{x+y}", "⠣⠒⠜⠭⠬⠽⠻", "§16"),
            ("\\sqrt{x^2+1}", "⠜⠭⠘⠆⠐⠬⠂⠻", "§16"),
            ("\\sqrt{x+\\sqrt{x+y}+z}", "⠜⠭⠬⠨⠜⠭⠬⠽⠨⠻⠬⠵⠻", "§16, nested"),
        ],
    );
}

#[test]
fn nemeth_modified_expressions_limits_and_integrals() {
    check(
        BrailleCode::Nemeth,
        &[
            ("\\lim_{x\\to 0} f(x)", "⠐⠇⠊⠍⠩⠭⠀⠫⠕⠀⠼⠴⠻⠀⠋⠷⠭⠾", "Ex. 15-3"),
            ("\\sum_{n=1}^{\\infty}", "⠐⠨⠠⠎⠩⠝⠀⠨⠅⠀⠼⠂⠣⠠⠿⠻", "Ex. 15-37"),
            ("\\int_a^b f(x)dx", "⠮⠰⠁⠘⠃⠐⠋⠷⠭⠾⠙⠭", "Ex. 23-16"),
            ("\\bar{x}", "⠭⠱", "§15.3, contracted bar"),
            ("\\hat{x}", "⠐⠭⠣⠸⠣⠻", "§15"),
            ("\\vec{AB}", "⠐⠠⠁⠠⠃⠣⠫⠕⠻", "§15"),
            ("\\binom{n}{k}", "⠷⠝⠩⠅⠾", "Ex. 15-43"),
        ],
    );
}

#[test]
fn nemeth_matrices_have_a_line_per_row() {
    let b = braille(r"\begin{pmatrix} 1 & a \\ 2 & b \end{pmatrix}", BrailleCode::Nemeth);
    // enlarged parentheses on every line, a numeric indicator on every number, no letter indicator (§25.8)
    assert_eq!(b, "⠠⠷⠼⠂⠀⠁⠠⠾\n⠠⠷⠼⠆⠀⠃⠠⠾");
}

#[test]
fn cmu_numbers_and_operations() {
    check(
        BrailleCode::Cmu,
        &[
            ("1.720", "⠼⠁⠄⠛⠃⠚", "§3.1"),
            ("3,2", "⠼⠉⠂⠃", "§3.2.1"),
            ("\\frac{3}{4}", "⠼⠒⠙", "§3.2.2"),
            ("2\\frac{3}{4}", "⠼⠃⠼⠒⠙", "§3.2.2"),
            ("6 \\pm 2", "⠼⠋⠖⠒⠤⠼⠃", "§4.1"),
            ("6 \\times 2", "⠼⠋⠦⠼⠃", "§4.1"),
            ("6 \\div 2", "⠼⠋⠲⠼⠃", "§4.1"),
            ("\\pi \\approx 3,1416", "⠈⠏⠈⠶⠼⠉⠂⠁⠙⠁⠋", "§4.2"),
            ("5x = 40b", "⠼⠑⠭⠶⠼⠙⠚⠐⠃", "§1.1"),
            ("x \\neq y", "⠭⠘⠶⠽", "§4.3"),
            ("n!", "⠝⠘⠄", "§7.5.1"),
        ],
    );
}

#[test]
fn cmu_fractions_powers_and_roots() {
    check(
        BrailleCode::Cmu,
        &[
            ("\\frac{a}{c}", "⠁⠲⠉", "§5.1"),
            ("a+\\frac{b}{c}", "⠁⠖⠃⠲⠉", "§5.1"),
            ("\\frac{a}{c \\cdot x}", "⠁⠲⠢⠉⠄⠭⠔", "§5.1"),
            ("\\frac{a+b}{c}", "⠢⠁⠖⠃⠔⠲⠉", "§5.1"),
            ("\\frac{a+\\frac{b}{c}}{d+e}", "⠢⠁⠖⠃⠲⠉⠔⠲⠢⠙⠖⠑⠔", "§5.1"),
            ("\\frac{\\frac{a+b}{c+d}}{x+y}", "⠢⠢⠁⠖⠃⠔⠲⠢⠉⠖⠙⠔⠔⠲⠢⠭⠖⠽⠔", "§5.1"),
            ("x^2", "⠭⠡⠼⠃", "§5.2"),
            ("x^{-1}", "⠭⠡⠤⠼⠁", "§5.2"),
            ("x^{a+b}", "⠭⠡⠢⠁⠖⠃⠔", "§5.2"),
            ("7x^3-2x^2+x+1", "⠼⠛⠭⠡⠼⠉⠤⠼⠃⠭⠡⠼⠃⠖⠭⠖⠼⠁", "§5.4"),
            ("\\sqrt{x}", "⠫⠱⠭", "§5.3"),
            ("\\sqrt[3]{x}", "⠫⠼⠉⠱⠭", "§5.3"),
            ("\\sqrt[n]{a+b}", "⠫⠝⠱⠢⠁⠖⠃⠔", "§5.3"),
            ("\\sqrt{x^2+y^2}", "⠫⠱⠢⠭⠡⠼⠃⠖⠽⠡⠼⠃⠔", "§5.4"),
            ("\\frac{x^2+1}{x^2-1}", "⠢⠭⠡⠼⠃⠖⠼⠁⠔⠲⠢⠭⠡⠼⠃⠤⠼⠁⠔", "§5.4"),
            ("x^{\\sqrt{a+5}}", "⠭⠡⠫⠱⠢⠁⠖⠼⠑⠔", "§5.4"),
        ],
    );
}

#[test]
fn cmu_sets_functions_and_calculus() {
    check(
        BrailleCode::Cmu,
        &[
            ("x \\in A", "⠭⠣⠂⠨⠁", "§6.1"),
            ("A \\cup B", "⠨⠁⠸⠜⠨⠃", "§6.1"),
            ("\\{x, y, z\\}", "⠐⠇⠭⠂⠀⠽⠂⠀⠵⠸⠂", "§6.1"),
            ("\\mathbb{R}", "⠸⠗", "§3.5"),
            ("\\Delta", "⠘⠙", "§1.2"),
            ("f(x)", "⠋⠣⠭⠜", "§7.1"),
            ("\\lim_{x \\to c}", "⠇⠊⠍⠄⠭⠒⠂⠉⠱", "§7.2"),
            ("\\frac{d}{dx}", "⠙⠲⠢⠙⠭⠔", "§7.3"),
            ("\\sum_{i=1}^{n}", "⠘⠎⠊⠶⠼⠁⠒⠝⠱", "§7.5.1"),
            ("\\int_1^4 x^2 dx", "⠯⠼⠁⠒⠼⠙⠱⠭⠡⠼⠃⠐⠙⠭", "§7.7"),
            ("\\log_b x", "⠇⠕⠛⠄⠃⠱⠭", "§7.5.2"),
            ("\\sin^2 x", "⠎⠑⠝⠄⠡⠼⠃⠭", "§7.7"),
            ("\\vec{AB}", "⠒⠂⠢⠨⠁⠨⠃⠔", "§8.1"),
            ("\\overline{AB}", "⠈⠉⠢⠨⠁⠨⠃⠔", "§6.1"),
        ],
    );
}

#[test]
fn the_locale_chooses_the_code() {
    let code = |locale| convert("x", &Options { locale, ..Options::default() }).unwrap().braille_code;
    assert_eq!(code(Locale::PtBr), BrailleCode::Cmu);
    assert_eq!(code(Locale::Es), BrailleCode::Cmu);
    assert_eq!(code(Locale::En), BrailleCode::Nemeth);
    assert_eq!(code(Locale::Fr), BrailleCode::Nemeth);
    // the option forces either code
    let r = convert(r"\frac{3}{4}", &Options { locale: Locale::PtBr, braille: Some(BrailleCode::Nemeth), ..Options::default() }).unwrap();
    assert_eq!(r.braille, "⠹⠒⠌⠲⠼");
    assert_eq!("CMU".parse::<BrailleCode>(), Ok(BrailleCode::Cmu));
    assert!("ueb".parse::<BrailleCode>().unwrap_err().contains("unknown braille code"));
}

#[test]
fn braille_reaches_the_svg_and_the_json() {
    let r = convert(r"\frac{3}{4}", &Options { locale: Locale::PtBr, ..Options::default() }).unwrap();
    assert!(r.svg.contains(r#"<mathra:braille code="cmu">⠼⠒⠙</mathra:braille>"#), "{}", r.svg);
    let json: serde_json::Value = serde_json::from_str(&r.to_json()).unwrap();
    assert_eq!(json["braille"], "⠼⠒⠙");
    assert_eq!(json["braille_code"], "cmu");
    // every input notation gives the same braille
    for (src, input) in [("3/4", mathra::Input::AsciiMath), ("<math><mfrac><mn>3</mn><mn>4</mn></mfrac></math>", mathra::Input::MathMl)] {
        assert_eq!(convert(src, &Options { locale: Locale::PtBr, input, ..Options::default() }).unwrap().braille, "⠼⠒⠙");
    }
}

#[test]
fn s_brl_1_a_symbol_without_a_sign_is_the_full_cell() {
    for code in BrailleCode::ALL {
        let r = convert(r"a \star b", &Options { braille: Some(code), ..Options::default() }).unwrap();
        assert!(r.braille.contains('⠿'), "{code}: {}", r.braille);
        let note = r.applied.iter().find(|a| a.rule == "S-BRL-1").unwrap_or_else(|| panic!("{code}: {:?}", r.applied));
        assert!(note.description.contains('⋆'), "{}", note.description);
        // it is a note on the output, not a problem with the input
        assert!(r.warnings.is_empty());
    }
}

#[test]
fn every_construction_of_the_corpus_has_braille_in_both_codes() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");
    for file in ["constructions.json", "packages.json"] {
        let json = std::fs::read_to_string(format!("{dir}/{file}")).unwrap();
        for (name, tex) in serde_json::from_str::<Vec<(String, String)>>(&json).unwrap() {
            for code in BrailleCode::ALL {
                let Ok(r) = convert(&tex, &Options { braille: Some(code), ..Options::default() }) else { continue };
                assert!(r.braille.chars().all(|c| ('\u{2800}'..='\u{28FF}').contains(&c) || c == '\n'), "{code} {name}: {}", r.braille);
            }
        }
    }
}
