//! Speech (screen readers) and linear text: what comes out for people who do not see the drawing.

use mathra::{convert, Locale, Options};

fn options(loc: Locale) -> Options {
    Options { locale: loc, ..Options::default() }
}

fn speech(tex: &str, loc: Locale) -> String {
    convert(tex, &options(loc)).unwrap_or_else(|e| panic!("{tex}: {e}")).speech
}

fn text(tex: &str, loc: Locale) -> String {
    convert(tex, &options(loc)).unwrap_or_else(|e| panic!("{tex}: {e}")).text
}

/// (input, pt-BR snippet, English snippet)
const SPEECHES: &[(&str, &str, &str)] = &[
    (r"\sqrt{x}", "raiz quadrada de", "square root of"),
    (r"\sqrt[2]{x}", "raiz quadrada de", "square root of"),
    (r"\sqrt[3]{x}", "raiz cúbica de", "cube root of"),
    (r"\sqrt[n]{x}", "raiz de índice n de", "root of index n of"),
    (r"\vec{v}", "vetor v", "vector v"),
    (r"\hat{x}", "x chapéu", "x hat"),
    (r"\dot{x}", "x ponto", "x dot"),
    (r"\ddot{x}", "x dois pontos", "x double dot"),
    (r"\tilde{x}", "x til", "x tilde"),
    (r"\overarc{AB}", "arco", "arc"),
    (r"\breve{x}", "x", "x"),
    (r"\overline{AB}", "barra", "bar"),
    (r"\bar{x}", "x barra", "x bar"),
    (r"\max_{x} f", "índice x", "sub x"),
    (r"\underline{x}", "x", "x"),
    (r"\underbrace{a+b}_{n}", "(n)", "(n)"),
    (r"\text{if $x$ is even}", "if", "if"),
    (r"\xrightarrow{a}", "seta para a direita", "right arrow"),
    (r"\xleftarrow{a}", "seta para a esquerda", "left arrow"),
    (r"\xleftrightarrow{a}", "seta dupla", "double arrow"),
    (r"\xRightarrow{a}", "implica", "implies"),
    (r"\xrightarrow[b]{a}", "(b)", "(b)"),
    (r"\xLeftarrow{a}", "a", "a"),
    (r"\overset{\text{def}}{=}", "com def em cima", "with def above"),
    (r"\underset{n}{\max}", "com n embaixo", "with n below"),
    (r"\begin{CD} A @>>> B \\ @VVV @AAA \\ C @= D \end{CD}", "seta para baixo", "down arrow"),
    (r"\begin{CD} A @AAA B \end{CD}", "seta para cima", "up arrow"),
    (r"\begin{CD} A @| B \end{CD}", "igual", "equals"),
    (r"\sum_{i=1}^{n} i", "de", "from"),
    (r"\sum_{i} i", "sobre", "over"),
    (r"\int^{1} f", "até 1 de", "to 1 of"),
    (r"\int f", "integral", "integral"),
    (r"\lim_{x \to 0} f", "limite com x tendendo a 0", "limit as x"),
    (r"\sin^2 x", "seno ao quadrado de x", "sine squared of x"),
    (r"\log_2 x", "logaritmo na base 2 de x", "log base 2 of x"),
    (r"\sin^{-1} y", "arco seno de y", "inverse sine of y"),
    (r"\log^{-1} y", "inversa de logaritmo de y", "inverse log of y"),
    (r"\sec^2 x", "secante ao quadrado de x", "secant squared of x"),
    (r"\cot x", "cotangente de x", "cotangent of x"),
    (r"|x|", "módulo de x", "absolute value of x"),
    (r"\lvert x \rvert", "módulo de x", "absolute value of x"),
    (r"|a + b|", "módulo de, a mais b, fim do módulo", "absolute value of, a plus b, end absolute value"),
    (r"\|v\|", "norma de v", "norm of v"),
    (r"\left\| u + v \right\|", "norma de, u mais v, fim da norma", "norm of, u plus v, end norm"),
    (r"|a| + |b|", "módulo de a mais módulo de b", "absolute value of a plus absolute value of b"),
    (r"a | b", "a barra b", "a bar b"),
    (r"[a]", "fecha colchetes", "close bracket"),
    (r"\csc x", "cossecante de x", "cosecant of x"),
    (r"\sinh^2 x", "senh ao quadrado", "sinh squared x"),
    (r"\lim_{x \to 0}^{+} f", "mais", "plus"),
    (r"x_{i}", "índice i", "sub i"),
    (r"x^{2}", "ao quadrado", "squared"),
    (r"x^{3}", "ao cubo", "cubed"),
    (r"f^{\prime}", "linha", "prime"),
    (r"f''", "duas linhas", "double prime"),
    (r"30^{\circ}", "graus", "degrees"),
    (r"a^{+}", "pela direita", "from the right"),
    (r"a^{-}", "pela esquerda", "from the left"),
    (r"\begin{vmatrix} a & b \\ c & d \end{vmatrix}", "determinante", "determinant"),
    (r"\begin{cases} 1 & x>0 \\ 0 & x\le 0 \end{cases}", "casos", "cases"),
    (r"\frac{1}{2}", "um meio", "over"),
    (r"\frac{3}{4}", "três quartos", "over"),
    (r"\frac{11}{2}", "11", "11"),
    (r"\iint f", "integral dupla", "double integral"),
    (r"\iiint f", "integral tripla", "triple integral"),
    (r"\oint f", "integral de linha", "contour integral"),
    (r"\bigcup_i A", "união", "union"),
    (r"\bigcap_i A", "interseção", "intersection"),
    (r"\coprod_i A", "operador", "operator"),
    (r"a \mp b", "menos ou mais", "minus or plus"),
    (r"a \neq b", "é diferente de", "is not equal to"),
    (r"a \propto b", "é proporcional a", "is proportional to"),
    (r"p \Rightarrow q", "implica", "implies"),
    (r"p \Leftrightarrow q", "se e somente se", "if and only if"),
    (r"A \rightleftharpoons B", "em equilíbrio com", "in equilibrium with"),
    (r"x \notin A", "não pertence a", "is not in"),
    (r"A \subset B", "está contido em", "is a subset of"),
    (r"A \subseteq B", "está contido ou é igual a", "is a subset of or equal to"),
    (r"A \cup B", "união", "union"),
    (r"A \cap B", "interseção", "intersection"),
    (r"\nabla f", "nabla", "nabla"),
    (r"\forall x", "para todo", "for all"),
    (r"\exists x", "existe", "there exists"),
    (r"\emptyset", "conjunto vazio", "empty set"),
    (r"\{ a \}", "fecha chaves", "close brace"),
    (r"a ; b", "ponto e vírgula", "semicolon"),
    (r"a : b", "dois-pontos", "colon"),
    (r"n!", "fatorial", "factorial"),
    (r"30°", "graus", "degrees"),
    (r"1, \ldots, n", "e assim por diante", "and so on"),
    (r"\triangle ABC", "triângulo", "triangle"),
    (r"r \perp s", "é perpendicular a", "is perpendicular to"),
    (r"r \parallel s", "é paralelo a", "is parallel to"),
    (r"\angle A", "ângulo", "angle"),
    (r"5\%", "por cento", "percent"),
    (r"a \approx b", "é aproximadamente", "is approximately"),
    (r"a \equiv b", "é côngruo a", "is congruent to"),
    (r"x \to 0", "tende a", "tends to"),
    (r"\cancel{x}", "x cancelado", "x canceled"),
    (r"\circled{1}", "1 circulado", "circled 1"),
    (r"\boxed{x}", "x", "x"),
];

#[test]
fn speech_in_portuguese_and_english() {
    let mut failures = Vec::new();
    for (tex, pt, en) in SPEECHES {
        let f = speech(tex, Locale::PtBr);
        if !f.contains(pt) {
            failures.push(format!("pt {tex}: «{f}» lacks «{pt}»"));
        }
        let f = speech(tex, Locale::En);
        if !f.contains(en) {
            failures.push(format!("en {tex}: «{f}» lacks «{en}»"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// (input, expected snippet of the linear text)
const TEXTS: &[(&str, &str)] = &[
    (r"\sqrt[3]{x}", "∛"),
    (r"\sqrt[4]{x}", "∜"),
    (r"\sqrt[n]{x}", "√[n]"),
    (r"f'^{2}", "′^"),
    (r"f''", "″"),
    (r"\tan^{-1} x", "tg^(−1) x"),
    (r"\log_2 x", "log_2 x"),
    (r"\sin^2 x + 1", "sen^2 x + 1"),
    (r"\lim_{x \to 0} f(x)", "lim_(x → 0) f(x)"),
    (r"\begin{pmatrix} a & b \end{pmatrix}", "([a, b])"),
    (r"\begin{vmatrix} a & b \end{vmatrix}", "|[a, b]|"),
    (r"\begin{bmatrix} a & b \end{bmatrix}", "[[a, b]]"),
    (r"\vec{v}", "v⃗"),
    (r"\overline{AB}", "‾"),
    (r"\underline{x}", "x"),
    (r"\underbrace{a+b}_{n}", " [n]"),
    (r"\text{if $x$}", "if x"),
    (r"a\quad b", "a b"),
    (r"a\,b", "a"),
    (r"\phantom{x}y", "y"),
    (r"\circled{1}", "(1)"),
    (r"\boxed{x}", "x"),
    (r"\xrightarrow{a}", "→[a]"),
    (r"\overset{a}{=}", "="),
    (r"\begin{CD} A @VVV B \end{CD}", "↓"),
    (r"\divchave{17}{5}", "resto"),
];

#[test]
fn linear_text() {
    let mut failures = Vec::new();
    for (tex, snippet) in TEXTS {
        let t = text(tex, Locale::PtBr);
        if !t.contains(snippet) {
            failures.push(format!("{tex}: «{t}» lacks «{snippet}»"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(text(r"\divchave{17}{5}", Locale::En).contains("remainder"));
}

#[test]
fn public_shortcuts() {
    assert!(mathra::tex_to_svg("x").unwrap().starts_with("<svg"));
    let (math_box, warnings) = mathra::typeset(r"\frac{a}{b}", &Options::default()).unwrap();
    assert!(math_box.width > 0.0 && warnings.is_empty());
    let (_, warnings) = mathra::typeset(r"\madeupcommand", &Options::default()).unwrap();
    assert!(!warnings.is_empty());
    assert!(mathra::typeset(r"\frac{a}", &Options { strict: true, ..Options::default() }).is_err());
}

#[test]
fn errors_have_readable_message() {
    let e = mathra::Error::MissingGlyph { character: '\u{10FFFF}' };
    assert!(e.to_string().contains("U+10FFFF"), "{e}");
    let e = convert(r"\frac{a}", &Options { strict: true, ..Options::default() }).unwrap_err();
    assert!(e.to_string().starts_with("column "), "{e}");
    let _: &dyn std::error::Error = &e;
}

#[test]
fn locale_from_text() {
    for s in ["pt-BR", "pt", "PT_BR"] {
        assert_eq!(s.parse::<Locale>().unwrap(), Locale::PtBr);
    }
    for s in ["en", "en-US", "en_us"] {
        assert_eq!(s.parse::<Locale>().unwrap(), Locale::En);
    }
    assert!("xx".parse::<Locale>().unwrap_err().contains("xx"));
    assert_eq!(Locale::En.to_string(), "en");
}

#[test]
fn spelled_out_functions_in_english() {
    for (tex, snippet) in [(r"\sin x", "sine of"), (r"\cos x", "cosine of"), (r"\tan x", "tangent of"), (r"\log x", "log of"), (r"\ln x", "natural log of"), (r"\sec x", "sec")] {
        let f = speech(tex, Locale::En);
        assert!(f.contains(snippet), "{tex}: {f}");
    }
}

#[test]
fn hand_built_tree_with_impossible_division() {
    // the reader refuses \divchave without integers, but the tree is public and may come from elsewhere
    let node = mathra::Node::LongDivision { dividend: "a".into(), divisor: "0".into() };
    assert!(mathra::output::speech::speak(&node, Locale::PtBr).contains('?'));
    assert!(mathra::output::text::linear(&node, Locale::PtBr).contains("resto"));
}

#[test]
fn hand_built_large_operator_without_limits() {
    let base = mathra::Node::LargeOperator { symbol: '∑', limits: None };
    let node = mathra::Node::Scripts { base: Box::new(base), sup: None, sub: None };
    assert_eq!(mathra::output::speech::speak(&node, Locale::PtBr), "somatório");
    assert!(node.children().is_empty());
}

#[test]
fn greek_letters_by_locale() {
    for (tex, pt, en) in [
        (r"\gamma", "gama", "gamma"),
        (r"\varepsilon", "épsilon", "epsilon"),
        (r"\xi", "csi", "xi"),
        (r"\rho", "rô", "rho"),
        (r"\varphi", "fi", "phi"),
        (r"\chi", "qui", "chi"),
        (r"\omega", "ômega", "omega"),
        (r"\Gamma", "gama maiúsculo", "capital gamma"),
        (r"\Theta", "teta maiúsculo", "capital theta"),
        (r"\Lambda", "lambda maiúsculo", "capital lambda"),
        (r"\Pi", "pi maiúsculo", "capital pi"),
        (r"\Sigma", "sigma maiúsculo", "capital sigma"),
        (r"\Phi", "fi maiúsculo", "capital phi"),
        (r"\Psi", "psi maiúsculo", "capital psi"),
    ] {
        assert_eq!(speech(tex, Locale::PtBr), pt, "{tex}");
        assert_eq!(speech(tex, Locale::En), en, "{tex}");
    }
}
