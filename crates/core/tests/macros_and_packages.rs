//! Substitution macros (L-MAC-1), package commands and the boundary of what mathra does not do (spec/boundary.md).

use mathra::tree::{Node, TableKind};
use mathra::{convert, Locale, Options};

fn pt(tex: &str) -> mathra::Output {
    convert(tex, &Options { locale: Locale::PtBr, ..Default::default() }).unwrap_or_else(|e| panic!("{tex}: {e}"))
}

fn error(tex: &str) -> String {
    convert(tex, &Options { strict: true, ..Default::default() }).err().map(|e| e.to_string()).unwrap_or_else(|| panic!("{tex} should fail"))
}

#[test]
fn l_mac_1_newcommand_with_argument_and_default() {
    assert_eq!(pt(r"\newcommand{\vet}[1]{\vec{#1}} \vet{F}").speech, "vetor F");
    assert_eq!(pt(r"\newcommand{\pot}[2][2]{#2^{#1}} \pot{x} + \pot[3]{y}").speech, "x ao quadrado mais y ao cubo");
    assert!(pt(r"\newcommand\um{1} \um").applied.iter().any(|a| a.rule == "L-MAC-1"));
}

#[test]
fn l_mac_1_def_let_and_renewcommand() {
    assert_eq!(pt(r"\def\q#1{#1^2} \q{x}").text, "x^2");
    assert_eq!(pt(r"\let\foo\alpha \foo").text, "α");
    // \renewcommand can even change a built-in command
    assert_eq!(pt(r"\renewcommand{\frac}[2]{#1/#2} \frac{a}{b}").text, "a/b");
}

#[test]
fn l_mac_1_declared_operators_and_delimiters() {
    assert_eq!(pt(r"\DeclareMathOperator{\Ln}{Ln} \Ln x").text, "Ln x");
    let r = pt(r"\DeclarePairedDelimiter{\abs}{\lvert}{\rvert} \abs*{\frac{a}{b}}");
    assert!(matches!(r.tree.children()[0], Node::Delimited { .. }), "{:?}", r.tree);
}

#[test]
fn l_mac_1_new_environment() {
    assert_eq!(pt(r"\newenvironment{par}{\left(}{\right)} \begin{par} x \end{par}").text, "(x)");
}

#[test]
fn l_mac_1_recursive_macro_stops_with_error() {
    assert!(error(r"\newcommand{\a}{\a x} \a").contains("too many macros"));
}

#[test]
fn physics_and_braket_packages() {
    assert_eq!(pt(r"\dv{f}{x}").text, "(df)/(dx)");
    assert_eq!(pt(r"\pdv{f}{x}{y}").text, "(∂^2f)/(∂x ∂y)");
    assert_eq!(pt(r"\ket{\psi}").text, "|ψ⟩");
    assert_eq!(pt(r"\braket{\phi|\psi}").text, "⟨ϕ | ψ⟩");
}

#[test]
fn amscd_commutative_diagram() {
    let r = pt(r"\begin{CD} A @>f>> B \\ @VgVV @VVhV \\ C @>>k> D \end{CD}");
    let Node::Table { shape, rows, .. } = &r.tree.children()[0] else { panic!("{:?}", r.tree) };
    assert_eq!(*shape, TableKind::Diagram);
    assert_eq!(rows.len(), 3);
    assert!(matches!(rows[1][0], Node::VerticalArrow { symbol: '↓', .. }));
    assert!(r.speech.starts_with("diagrama: linha 1: A, seta para a direita (f), B"), "{}", r.speech);
}

#[test]
fn substack_multicolumn_and_intertext() {
    pt(r"\sum_{\substack{0 \le i \le m \\ 0 < j < n}} P(i, j)");
    let r = pt(r"\begin{array}{|c|c|} \hline \multicolumn{2}{|c|}{\text{Total}} \\ \hline 1 & 2 \\ \hline \end{array}");
    let Node::Table { rows, .. } = &r.tree.children()[0] else { panic!() };
    assert!(matches!(rows[0][0], Node::Multicolumn { columns: 2, .. }));
    let r = pt(r"\begin{align} a &= b \intertext{logo} c &= d \end{align}");
    let Node::Table { intertexts, .. } = &r.tree.children()[0] else { panic!() };
    assert_eq!(intertexts, &[1]);
}

#[test]
fn plain_tex_cases_and_matrix() {
    assert_eq!(pt(r"\cases{1 & x > 0 \cr 0 & x \le 0}").text, "{1, x > 0; 0, x ≤ 0");
    assert_eq!(pt(r"\matrix{a & b \cr c & d}").text, "[[a, b], [c, d]]");
}

#[test]
fn latex_boxes_and_spaces() {
    for tex in [
        r"\genfrac{(}{)}{0pt}{}{n}{k}",
        r"\sfrac{1}{2} + \unitfrac[3]{m}{s}",
        r"\sideset{_a^b}{_c^d}\prod",
        r"x + \smash{\frac{a}{b}} + \mathclap{\text{c}}",
        r"\raisebox{0.5em}{alto} \scalebox{1.6}{grande} \rule{2em}{0.4pt}",
        r"1\textsuperscript{a} \verb|x^2|",
        r"\llbracket 1, n \rrbracket",
        r"\usepackage{amsmath} \makeatletter x",
    ] {
        pt(tex);
    }
}

#[test]
fn boundary_tex_programming_and_diagrams_stay_out() {
    for (tex, reason) in [
        (r"\def\a{1}\ifx\a\a sim\fi", r"\ifx"),
        (r"\catcode`\@=11 x", r"\catcode"),
        (r"\begin{tikzcd} A \arrow[r] & B \end{tikzcd}", "tikzcd"),
        (r"\xymatrix{A \ar[r] & B}", r"\xymatrix"),
        (r"\tikz \draw (0,0)--(1,1);", r"\tikz"),
        (r"\chemfig{H-C}", r"\chemfig"),
    ] {
        assert!(error(tex).contains(reason), "{tex}");
    }
}
