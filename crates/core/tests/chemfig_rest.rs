//! Issue #214: chemfig's default bond values at the start of a structure, `\charge`, `-U>`,
//! `\chemleft`/`\chemright`, `\chemup`/`\chemdown`, `\chemskipalign`, distant bonds whose mark sits
//! inside an atom group, and `\merge` (left out with a warning). Section numbers are the chemfig
//! manual's (v1.83).
#![cfg(feature = "chem")]

use mathra::chem::model::{ArrowKind, Body, BondKind, ChargeContent, Item, LewisMark, Molecule, Scheme, UArc};
use mathra::tree::Node;
use mathra::{convert, Input, Options};

fn tolerant(tex: &str) -> mathra::Output {
    convert(tex, &Options::default()).unwrap_or_else(|e| panic!("{tex}: {e}"))
}

/// Read in strict mode, and with no warning in tolerant mode.
fn strict(tex: &str) -> mathra::Output {
    let out = convert(tex, &Options { strict: true, ..Default::default() }).unwrap_or_else(|e| panic!("{tex}: {e}"));
    assert!(tolerant(tex).warnings.is_empty(), "{tex}: {:?}", tolerant(tex).warnings);
    out
}

fn scheme_of(node: &Node) -> Scheme {
    match node {
        Node::Chemistry { scheme } => scheme.clone(),
        Node::Row { children } => children.iter().find_map(|c| matches!(c, Node::Chemistry { .. }).then(|| scheme_of(c))).expect("a scheme"),
        other => panic!("{other:?}"),
    }
}

fn molecule(body: &str) -> Molecule {
    match &scheme_of(strict(&format!("\\chemfig{{{body}}}")).tree.single()).items[0] {
        Item::Compound(c) => match &c.body {
            Body::Structure(m) => m.clone(),
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
}

/// Tolerant mode warns with `rule`; strict mode refuses.
fn warns(tex: &str, rule: &str) -> mathra::Output {
    let out = tolerant(tex);
    assert!(out.warnings.iter().any(|w| w.rule == rule), "{tex}: expected {rule}, got {:?}", out.warnings);
    assert!(convert(tex, &Options { strict: true, ..Default::default() }).is_err(), "{tex}: strict mode accepted it");
    out
}

#[test]
fn defaults_at_the_start_of_a_structure() {
    // section 2.6: every bond 20° and 1.5 times as long, unless it says otherwise
    let m = molecule("[:20,1.5]A-B-C-[:-80,0.7]D-E-F");
    let angles: Vec<f32> = m.bonds.iter().map(|b| b.angle).collect();
    let lengths: Vec<f32> = m.bonds.iter().map(|b| b.length).collect();
    assert_eq!(angles, [20.0, 20.0, -80.0, 20.0, 20.0]);
    assert_eq!(lengths, [1.5, 1.5, 0.7, 1.5, 1.5]);
    let m = molecule("[:20,1.5]A-B");
    assert_eq!((m.bonds[0].angle, m.bonds[0].length), (20.0, 1.5));
    // the issue's other form: an ordinary bond argument, as before
    let m = molecule("A-[:20,1.5]B");
    assert_eq!((m.bonds[0].angle, m.bonds[0].length), (20.0, 1.5));
    // all five fields: departure and arrival on the second atom of each group; a predefined angle
    let m = molecule("[:30,1.5,2,2,red,thick]ABC-DEF=GHI");
    assert!(m.bonds.iter().all(|b| (b.angle, b.departure, b.arrival) == (30.0, 1, 1)), "{:?}", m.bonds);
    assert_eq!(molecule("[1]A-B").bonds[0].angle, 45.0);
    // the TikZ field applies to every bond: draw=none hides them
    assert!(molecule("[,,,,draw=none]A-B-C").bonds.iter().all(|b| b.kind == BondKind::Invisible));
    // a relative angle still counts from the previous bond
    let m = molecule("[:10]A-[::30]B-[::30]C-D");
    assert_eq!(m.bonds.iter().map(|b| b.angle).collect::<Vec<_>>(), [40.0, 70.0, 10.0]);
    // a branch takes the structure's default too
    let m = molecule("[:90]A-B(-C)-D");
    assert!(m.bonds.iter().all(|b| b.angle == 90.0), "{:?}", m.bonds);
}

#[test]
fn charges_around_an_atom() {
    // section 5.8: \charge{position=charge,…}{atom}, the Lewis shortcuts and text
    let m = molecule(r"R-\charge{90=\|,-90=\|}{O}-H");
    assert_eq!(m.atoms[1].label, "O");
    let marks: Vec<_> = m.atoms[1].charges.iter().map(|c| (c.angle, c.content.clone())).collect();
    assert_eq!(marks, [(90.0, ChargeContent::Mark { mark: LewisMark::Line }), (-90.0, ChargeContent::Mark { mark: LewisMark::Line })]);
    let m = molecule(r#"\charge{30=\:,120=$\ominus$,210:5pt=$\delta^+$,north east=\.,0=\"}{Fe}"#);
    let c = &m.atoms[0].charges;
    assert_eq!(c.len(), 5);
    assert_eq!(c[0].content, ChargeContent::Mark { mark: LewisMark::Pair });
    assert_eq!(c[1].content, ChargeContent::Text { tex: r"$\ominus$".into() });
    assert_eq!((c[2].angle, c[2].offset), (-150.0, 0.5));
    assert_eq!((c[3].angle, &c[3].content), (45.0, &ChargeContent::Mark { mark: LewisMark::Radical }));
    assert_eq!(c[4].content, ChargeContent::Mark { mark: LewisMark::Vacancy });
    // the piece it is around: N in NH_3, the second piece after H_3
    assert_eq!(molecule(r"\charge{90=\.}{N}H_3").atoms[0].charges[0].piece, 0);
    assert_eq!(molecule(r"H_3\charge{90=\.}{N}").atoms[0].charges[0].piece, 1);
    // the anchor option and extra sep
    let m = molecule(r"\charge{[extra sep=0pt]45[anchor=180+\chargeangle]=$\scriptstyle\oplus$}{A}-[2]B");
    let c = &m.atoms[0].charges[0];
    assert_eq!(c.anchor, Some(-135.0));
    assert!((c.offset + 0.15).abs() < 1e-6, "{}", c.offset);
    // on an empty vertex of a ring
    let m = molecule(r"*5(---\charge{90:2pt=$\scriptstyle\oplus$}{}-\charge{135:2pt=$\scriptstyle-$}{}-)");
    assert_eq!(m.atoms.iter().map(|a| a.charges.len()).sum::<usize>(), 2);
    // a formal charge counts in the formula; a partial one does not
    assert_eq!(mathra::chem::formula::of(&molecule(r"\charge{90=$\oplus$}{N}H_4")).written(), "H₄N⁺");
    assert_eq!(mathra::chem::formula::of(&molecule(r"\charge{90=$\delta^+$}{C}")).written(), "C");
    // outside \chemfig, around one atom
    let out = strict(r"\charge{90=\|}{X}");
    let s = scheme_of(out.tree.single());
    assert!(matches!(&s.items[0], Item::Compound(c) if matches!(&c.body, Body::Structure(m) if m.atoms[0].charges.len() == 1)));
    assert_ne!(out.svg, strict(r"\chemfig{X}").svg);
    // an unknown position is left out with a warning
    warns(r"\chemfig{\charge{somewhere=\.}{X}}", "E-REC-9");
}

#[test]
fn u_arrow() {
    // sections 6.2 and 6.9: a forward arrow with an arc; radius 0.333 of the length, 60° each way
    let s = scheme_of(strict(r"\schemestart A\arrow{-U>[$x$][$y$]}B\schemestop").tree.single());
    let Item::Arrow(a) = &s.items[1] else { panic!() };
    assert_eq!((a.kind, a.arc), (ArrowKind::Forward, Some(UArc { radius: 0.333, half_angle: 60.0 })));
    assert!(a.above.is_some() && a.below.is_some());
    let s = scheme_of(strict(r"\schemestart A\arrow{-U>[123][456][][-0.25][-90]}B\schemestop").tree.single());
    let Item::Arrow(a) = &s.items[1] else { panic!() };
    assert_eq!(a.arc, Some(UArc { radius: -0.25, half_angle: 90.0 }));
    // drawn: the arc adds strokes to the plain arrow; read as "reacts to give"
    let u = strict(r"\schemestart A\arrow{-U>[$x$][$y$]}B\schemestop");
    let plain = strict(r"\schemestart A\arrow{->[$x$][$y$]}B\schemestop");
    assert!(u.svg.matches("<path").count() + u.svg.matches("<line").count() > plain.svg.matches("<path").count() + plain.svg.matches("<line").count());
    assert_eq!(u.speech, plain.speech);
    // no longer an unknown arrow type
    assert!(tolerant(r"\schemestart A\arrow{-U>}B\schemestop").warnings.is_empty());
}

#[test]
fn chemfig_delimiters() {
    // section 6.8: \chemleft and \chemright are \left and \right around the material
    assert_eq!(strict(r"\chemleft[ \chemfig{A-B} \chemright]").tree, strict(r"\left[ \chemfig{A-B} \right]").tree);
    assert_eq!(strict(r"\chemleft\{\chemfig{A-[1]B}\chemright\}").tree, strict(r"\left\{\chemfig{A-[1]B}\right\}").tree);
    // \chemup and \chemdown: over and under it
    assert_eq!(strict(r"\chemup[\chemfig{A-[1]B-[7]C}\chemdown]").tree, strict(r"\overbracket{\underbracket{{\chemfig{A-[1]B-[7]C}}}}").tree);
    assert_eq!(strict(r"\chemup\{X\chemdown\}").tree, strict(r"\overbrace{\underbrace{{X}}}").tree);
    assert_eq!(strict(r"\chemup(X\chemdown.").tree, strict(r"\overparen{{X}}").tree);
    // inside a scheme, the delimited material is one compound, a sub-scheme in it
    let s = scheme_of(strict(r"\schemestart A\arrow \chemleft[\subscheme{B\arrow[-90,2]C}\chemright] \arrow D\schemestop").tree.single());
    assert_eq!(s.items.len(), 5, "{:?}", s.items);
    assert!(matches!(&s.items[2], Item::Compound(c) if matches!(&c.body, Body::Math { tex } if tex.starts_with(r"\chemleft[") && tex.ends_with(r"\chemright]"))));
    let s = scheme_of(strict(r"\schemestart X\arrow \chemup\{\chemfig{A-[1]B-[7]C}\chemdown\} \arrow Y\schemestop").tree.single());
    assert_eq!(s.items.len(), 5, "{:?}", s.items);
    // a \chemdown alone is ignored with a warning, as chemfig does; \chemup without one too
    warns(r"x \chemdown]", "E-REC-9");
    warns(r"\chemup[ x", "E-REC-5");
}

#[test]
fn skip_align() {
    // section 5.4.3: the group's baseline on the bond's line
    let m = molecule(r"A-\chemskipalign.-B");
    assert!(m.atoms[1].skip_align && m.atoms[1].label == ".");
    assert_ne!(strict(r"\chemfig{A-\chemskipalign.-B}").svg, strict(r"\chemfig{A-.-B}").svg);
    // outside a structure it changes nothing
    assert_eq!(strict(r"x\chemskipalign y").tree, strict("xy").tree);
}

#[test]
fn distant_bonds() {
    // the issue's example: the mark at the start of a group belongs to its atom, B
    let m = molecule("A-?[a]B-C-?[a]");
    assert_eq!(m.atoms.iter().map(|a| a.label.as_str()).collect::<Vec<_>>(), ["A", "B", "C", ""]);
    assert_eq!((m.bonds.len(), m.bonds[3].from, m.bonds[3].to), (4, 1, 3));
    // section 2.8: every later mark of a name is bonded to the first, with its own link
    let m = molecule("A-B(-[1]W-X?)(-[7]Y-Z?)-C?");
    let distant: Vec<(String, String)> = m.bonds.iter().filter(|b| m.atoms[b.from].label == "X").map(|b| (m.atoms[b.from].label.clone(), m.atoms[b.to].label.clone())).collect();
    assert_eq!(distant, [("X".to_string(), "Z".to_string()), ("X".to_string(), "C".to_string())]);
    let m = molecule("A?[a]-B(-[1]W?[a,2]-X?[b])(-[7]Y-Z?[b,1])-C?[b,{>}]");
    let kind = |from: &str, to: &str| m.bonds.iter().find(|b| m.atoms[b.from].label == from && m.atoms[b.to].label == to).map(|b| b.kind);
    assert_eq!((kind("A", "W"), kind("X", "Z"), kind("X", "C")), (Some(BondKind::Double), Some(BondKind::Single), Some(BondKind::Wedge)));
    // several marks after one atom; the piece the mark follows
    assert_eq!(molecule("A?[a]-[:-72]B-C?[a]?[b]-[:72]D-[:144]E?[a]?[b]").bonds.len(), 7);
    let m = molecule("H_3C?-A-B?");
    assert_eq!(m.bonds[2].departure, 1);
    // a mark no other one names draws nothing, as in chemfig
    assert_eq!(molecule("A?[z]-B").bonds.len(), 1);
}

#[test]
fn merge_is_left_out() {
    // section 6.11: \merge joins compounds named by \arrow(name--name), which mathra does not name
    let out = warns(r"\schemestart A\arrow{<=>}[90]B\merge>(c1)(c2)--()C\schemestop", "E-REC-1");
    let s = scheme_of(out.tree.single());
    assert_eq!(s.items.iter().filter(|i| matches!(i, Item::Compound(_))).count(), 3, "{:?}", s.items);
    // outside a scheme chemfig does not define it: its arguments go with it
    let out = warns(r"\chemfig{A-B}\merge{->}(c1)(c2)--(c3)", "E-REC-1");
    assert!(!out.text.contains("c1"), "{}", out.text);
}

#[test]
fn mathml_keeps_the_fallback() {
    for tex in [r"\chemfig{[:20,1.5]A-B}", r"\charge{90=\|}{X}", r"\schemestart A\arrow{-U>[$x$]}B\schemestop", r"\chemleft[\chemfig{A-B}\chemright]"] {
        let out = strict(tex);
        let back = convert(&out.mathml, &Options { input: Input::MathMl, ..Default::default() });
        assert!(back.is_ok(), "{tex}: {}", out.mathml);
    }
}
