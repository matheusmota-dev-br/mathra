//! The boxes of marked groups (`Output::boxes`): what an editor reads to place its cursor. Marking a
//! piece must not move anything, and each box must sit where its content is drawn.

use mathra::layout::{Item, MathBox};
use mathra::output::boxes::GroupBox;
use mathra::Options;

fn options() -> Options {
    Options { fallback_fonts: Vec::new(), ..Options::default() }
}

fn boxes(tex: &str) -> Vec<GroupBox> {
    mathra::convert(tex, &options()).unwrap_or_else(|e| panic!("{tex}: {e}")).boxes
}

/// The box marked `m=<key>`.
fn marked<'a>(list: &'a [GroupBox], key: &str) -> &'a GroupBox {
    list.iter().find(|b| b.attributes.iter().any(|(k, v)| k == "data-m" && v == key)).unwrap_or_else(|| panic!("no box m={key} in {list:?}"))
}

/// Everything drawn, groups opened, as text rounded to a hundredth of a unit: two layouts that print
/// the same draw the same.
fn drawing(b: &MathBox) -> Vec<String> {
    fn walk(items: &[Item], out: &mut Vec<String>) {
        let r = |v: f32| (v * 100.0).round() / 100.0;
        for item in items {
            match item {
                Item::Glyph { id, x, y, scale, .. } => out.push(format!("g{} {} {} {}", id.0, r(*x), r(*y), r(*scale))),
                Item::Rule { x, y, width, height, .. } => out.push(format!("r {} {} {} {}", r(*x), r(*y), r(*width), r(*height))),
                Item::Line { x1, y1, x2, y2, .. } => out.push(format!("l {} {} {} {}", r(*x1), r(*y1), r(*x2), r(*y2))),
                Item::Ellipse { cx, cy, rx, ry, .. } => out.push(format!("e {} {} {} {}", r(*cx), r(*cy), r(*rx), r(*ry))),
                Item::Transformed { matrix, items } => {
                    out.push(format!("t {:?}", matrix.map(r)));
                    walk(items, out);
                }
                Item::Group { items, .. } => walk(items, out),
            }
        }
    }
    let mut out = Vec::new();
    walk(&b.items, &mut out);
    out
}

fn typeset(tex: &str) -> MathBox {
    mathra::typeset(tex, &options()).unwrap_or_else(|e| panic!("{tex}: {e}")).0
}

/// Marking every atom and every slot, as an editor does, draws exactly what the bare formula draws.
/// A marked operator keeps its class only through `\mathbin`, `\mathrel`…: `\htmlData{…}{=}`, like
/// `{=}`, is an ordinary atom.
#[test]
fn marking_moves_nothing() {
    let pairs = [
        (
            r"x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}",
            r"\htmlData{m=1}{x}\mathrel{\htmlData{m=2}{=}}\htmlData{m=3}{\frac{\htmlData{m=4}{\mathbin{\htmlData{m=5}{-}}\htmlData{m=6}{b}\mathbin{\htmlData{m=7}{\pm}}\htmlData{m=8}{\sqrt{\htmlData{m=9}{\htmlData{m=10}{b}^{\htmlData{m=11}{2}}\mathbin{\htmlData{m=12}{-}}\htmlData{m=13}{4}\htmlData{m=14}{a}\htmlData{m=15}{c}}}}}}{\htmlData{m=16}{\htmlData{m=17}{2}\htmlData{m=18}{a}}}}",
        ),
        (r"\sum_{n=1}^{\infty} a_n", r"\htmlData{m=1}{\sum_{\htmlData{m=2}{n=1}}^{\htmlData{m=3}{\infty}}}\htmlData{m=4}{a}_{\htmlData{m=5}{n}}"),
        (
            r"\int_0^1 f(x)\,dx",
            r"\htmlData{m=1}{\int_{\htmlData{m=2}{0}}^{\htmlData{m=3}{1}}}\htmlData{m=4}{f}\mathopen{\htmlData{m=5}{(}}\htmlData{m=6}{x}\mathclose{\htmlData{m=7}{)}}\,\htmlData{m=8}{d}\htmlData{m=9}{x}",
        ),
        (r"\left(\frac{a}{b}\right)^2", r"\htmlData{m=1}{\left(\htmlData{m=2}{\frac{\htmlData{m=3}{a}}{\htmlData{m=4}{b}}}\right)}^{\htmlData{m=5}{2}}"),
        (r"a + b \le c", r"\htmlData{m=1}{a}\mathbin{\htmlData{m=2}{+}}\htmlData{m=3}{b}\mathrel{\htmlData{m=4}{\le}}\htmlData{m=5}{c}"),
        (r"\sqrt[3]{x}", r"\htmlData{m=1}{\sqrt[\htmlData{m=2}{3}]{\htmlData{m=3}{x}}}"),
        (
            r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
            r"\htmlData{m=1}{\begin{pmatrix} \htmlData{m=2}{a} & \htmlData{m=3}{b} \\ \htmlData{m=4}{c} & \htmlData{m=5}{d} \end{pmatrix}}",
        ),
        (r"\lim_{x \to 0} \frac{\sin x}{x}", r"\htmlData{m=1}{\lim_{\htmlData{m=2}{x \to 0}}}\htmlData{m=3}{\frac{\htmlData{m=4}{\sin x}}{\htmlData{m=5}{x}}}"),
    ];
    for (bare, marked) in pairs {
        let (a, b) = (typeset(bare), typeset(marked));
        assert_eq!(drawing(&a), drawing(&b), "marking moved something in {bare}");
        assert_eq!((a.width, a.height, a.depth), (b.width, b.height, b.depth), "marking changed the size of {bare}");
    }
}

#[test]
fn a_formula_without_marks_has_no_boxes() {
    assert!(boxes(r"\frac{a}{b}").is_empty());
}

/// One glyph: its box starts at the origin, its top is its height above the baseline (y down).
#[test]
fn a_single_box_is_in_svg_coordinates() {
    let b = typeset("x");
    let list = boxes(r"\htmlData{m=x}{x}");
    let x = marked(&list, "x");
    assert_eq!((x.x, x.baseline, x.level), (0.0, 0.0, 0));
    assert!((x.y + b.height).abs() < 0.01, "top {} for height {}", x.y, b.height);
    assert!((x.height - b.height - b.depth).abs() < 0.01);
    // the formula's width also counts the last glyph's italic correction; the box is the glyph's own
    assert!(x.width > 0.0 && x.width <= b.width + 0.01, "width {} in {}", x.width, b.width);
}

#[test]
fn numerator_above_denominator_inside_the_fraction() {
    let list = boxes(r"\htmlData{m=f}{\frac{\htmlData{m=n}{a+b}}{\htmlData{m=d}{c}}}");
    let (f, n, d) = (marked(&list, "f"), marked(&list, "n"), marked(&list, "d"));
    assert!(n.y + n.height <= d.y, "numerator {n:?} not above denominator {d:?}");
    for inner in [n, d] {
        assert!(inner.x >= f.x - 0.01 && inner.x + inner.width <= f.x + f.width + 0.01, "{inner:?} outside {f:?}");
        assert!(inner.y >= f.y - 0.01 && inner.y + inner.height <= f.y + f.height + 0.01, "{inner:?} outside {f:?}");
        assert_eq!(inner.level, 1);
    }
    assert!(n.baseline < 0.0 && d.baseline > 0.0, "baselines {} and {}", n.baseline, d.baseline);
}

/// Atoms of a row: left to right, without overlap, the gap between two at most an italic correction
/// (well under a tenth of an em), on one baseline.
#[test]
fn atoms_of_a_row_follow_each_other() {
    let list = boxes(r"\htmlData{m=a}{a}\htmlData{m=b}{b}\htmlData{m=c}{c}");
    let (a, b, c) = (marked(&list, "a"), marked(&list, "b"), marked(&list, "c"));
    for (l, r) in [(a, b), (b, c)] {
        let gap = r.x - (l.x + l.width);
        assert!((-0.01..100.0).contains(&gap), "gap {gap} between {l:?} and {r:?}");
    }
    assert!(a.baseline == b.baseline && b.baseline == c.baseline);
}

#[test]
fn an_exponent_sits_above_and_smaller() {
    let list = boxes(r"\htmlData{m=x}{x}^{\htmlData{m=e}{2}}");
    let (x, e) = (marked(&list, "x"), marked(&list, "e"));
    assert!(e.x >= x.x + x.width - 0.01, "exponent {e:?} not right of {x:?}");
    assert!(e.baseline < x.baseline, "exponent baseline {} not above {}", e.baseline, x.baseline);
    assert!(e.height < x.height);
}

/// `\scalebox` scales the box and a `\reflectbox` keeps it inside the mirrored width.
#[test]
fn boxes_follow_scaling_and_mirroring() {
    let one = marked(&boxes(r"\htmlData{m=x}{x}"), "x").clone();
    let two = marked(&boxes(r"\scalebox{2}{$\htmlData{m=x}{x}$}"), "x").clone();
    assert!((two.width - 2.0 * one.width).abs() < 0.01 && (two.height - 2.0 * one.height).abs() < 0.01);
    let list = boxes(r"\reflectbox{$\htmlData{m=a}{a}\htmlData{m=b}{bb}$}");
    let (a, b) = (marked(&list, "a"), marked(&list, "b"));
    assert!(b.x + b.width <= a.x + 0.01, "mirrored: {b:?} should be left of {a:?}");
}

/// A turned group's box covers its turned corners.
#[test]
fn a_turned_box_covers_the_turn() {
    let flat = marked(&boxes(r"\htmlData{m=x}{xyz}"), "x").clone();
    let turned = marked(&boxes(r"\rotatebox{90}{$\htmlData{m=x}{xyz}$}"), "x").clone();
    assert!((turned.height - flat.width).abs() < 0.5, "{turned:?} from {flat:?}");
}

#[test]
fn the_json_has_the_boxes() {
    let json: serde_json::Value = serde_json::from_str(&mathra::convert(r"\htmlData{m=1}{x}", &options()).unwrap().to_json()).unwrap();
    let b = &json["boxes"][0];
    assert_eq!(b["attributes"]["data-m"], "1");
    assert_eq!(b["level"], 0);
    for k in ["x", "y", "width", "height", "baseline"] {
        assert!(b[k].is_number(), "{k}");
    }
}
