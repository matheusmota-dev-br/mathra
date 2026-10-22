//! The fonts an equation can be set in.

use mathra::font::MathFont;
use mathra::{convert, convert_in_font, Options};

#[test]
fn stix_two_is_the_default() {
    assert_eq!(MathFont::default(), MathFont::StixTwo);
    assert_eq!(MathFont::StixTwo.name(), "STIX Two Math");
    let a = convert("x^2", &Options::default()).unwrap().svg;
    let b = convert_in_font("x^2", &Options::default(), MathFont::StixTwo).unwrap().svg;
    assert_eq!(a, b);
    assert!(a.contains("<mathra:font>STIX Two Math</mathra:font>"));
}

#[cfg(feature = "latin-modern")]
mod latin_modern {
    use super::*;
    use mathra::{typeset, typeset_in_font};

    #[test]
    fn has_its_own_metrics() {
        let options = Options { display: false, ..Default::default() };
        let (stix, _) = typeset(r"\text{Hello} \sum_{i=1}^n \sqrt{x_i}", &options).unwrap();
        let (lm, _) = typeset_in_font(r"\text{Hello} \sum_{i=1}^n \sqrt{x_i}", &options, MathFont::LatinModern).unwrap();
        assert!((stix.width - lm.width).abs() > 1.0, "{} {}", stix.width, lm.width);
        assert!(lm.width > 0.0 && lm.height > 0.0);
    }

    #[test]
    fn words_are_a_third_of_an_em_apart() {
        assert_eq!(MathFont::StixTwo.word_space(), 0.25);
        assert!((MathFont::LatinModern.word_space() - 0.332).abs() < 1e-6);
        let options = Options { display: false, ..Default::default() };
        let w = |t: &str| typeset_in_font(t, &options, MathFont::LatinModern).unwrap().0.width;
        assert!((w(r"\text{a b}") - w(r"\text{ab}") - 332.0).abs() < 1e-3);
    }

    #[test]
    fn draws_its_own_glyphs() {
        let svg = convert_in_font(r"\left( \frac{a}{b} \right)", &Options::default(), MathFont::LatinModern).unwrap().svg;
        assert!(svg.contains("<mathra:font>Latin Modern Math</mathra:font>"));
        let stix = convert(r"\left( \frac{a}{b} \right)", &Options::default()).unwrap().svg;
        assert_ne!(svg, stix);
        assert_eq!(MathFont::LatinModern.get().units_per_em(), 1000.0);
    }
}
