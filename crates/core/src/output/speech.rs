//! Phrase for screen readers, in the language of the locale.

use crate::layout::classes;
use crate::locale::Locale;
use crate::tree::*;

pub fn speak(node: &Node, loc: Locale) -> String {
    let mut f = Speaker { loc, words: Vec::new() };
    f.node(node);
    let phrase = f.words.join(" ");
    phrase.split_whitespace().collect::<Vec<_>>().join(" ").replace(" ,", ",").replace(" ;", ";").replace(" :", ":")
}

struct Speaker {
    loc: Locale,
    words: Vec<String>,
}

impl Speaker {
    fn t(&self, pt: &str, en: &str) -> String {
        match self.loc {
            Locale::PtBr => pt.to_string(),
            Locale::En => en.to_string(),
        }
    }

    fn say(&mut self, s: impl Into<String>) {
        let s = s.into();
        if !s.is_empty() {
            self.words.push(s);
        }
    }

    fn spoken(&self, node: &Node) -> String {
        speak(node, self.loc)
    }

    fn node(&mut self, node: &Node) {
        match node {
            Node::Number { value, thousands, .. } => {
                let n = super::written_number(value, *thousands, self.loc);
                self.say(n);
            }
            Node::Identifier { name, .. } => {
                let l = letter(name, self.loc);
                self.say(l);
            }
            Node::Function { name, .. } => {
                let f = match (self.loc, name.as_str()) {
                    (Locale::PtBr, "lim") => "limite de".to_string(),
                    (Locale::En, "lim") => "limit of".to_string(),
                    (_, "det") => self.t("determinante de", "determinant of"),
                    (_, "max") => self.t("máximo de", "maximum of"),
                    (_, "min") => self.t("mínimo de", "minimum of"),
                    _ => self.loc.speak_function(name),
                };
                self.say(f);
            }
            Node::Operator { symbol, class } | Node::FixedDelimiter { symbol, class, .. } => {
                let o = operator(*symbol, *class, self.loc);
                self.say(o);
            }
            Node::LargeOperator { symbol, .. } => {
                let o = large_operator(*symbol, self.loc);
                self.say(o);
            }
            Node::Row { children } => {
                let cl = classes(children, self.loc, None, false);
                for (child, class) in children.iter().zip(cl) {
                    match (child, class) {
                        (Node::Operator { symbol, .. }, Some(c)) => {
                            let o = operator(*symbol, c, self.loc);
                            self.say(o);
                        }
                        (other, _) => self.node(other),
                    }
                }
            }
            Node::Fraction { num, den, rule, .. } => {
                if !rule {
                    let (n, d) = (self.spoken(num), self.spoken(den));
                    self.say(self.t(&format!("{n} escolhe {d}"), &format!("{n} choose {d}")));
                } else if let Some(name) = ordinary_fraction(num, den, self.loc) {
                    self.say(name);
                } else if is_simple(num) && is_simple(den) {
                    let (n, d) = (self.spoken(num), self.spoken(den));
                    self.say(self.t(&format!("{n} sobre {d}"), &format!("{n} over {d}")));
                } else {
                    let (n, d) = (self.spoken(num), self.spoken(den));
                    self.say(self.t(&format!("fração, {n}, sobre {d}, fim da fração"), &format!("fraction, {n}, over {d}, end fraction")));
                }
            }
            Node::Root { radicand, index } => {
                let r = self.spoken(radicand);
                let which = match index.as_deref() {
                    None => self.t("raiz quadrada de", "square root of"),
                    Some(Node::Number { value, .. }) if value == "3" => self.t("raiz cúbica de", "cube root of"),
                    Some(Node::Number { value, .. }) if value == "2" => self.t("raiz quadrada de", "square root of"),
                    Some(i) => {
                        let i = self.spoken(i);
                        self.t(&format!("raiz de índice {i} de"), &format!("root of index {i} of"))
                    }
                };
                if is_simple(radicand) {
                    self.say(format!("{which} {r}"));
                } else {
                    self.say(format!("{which} {r}, {}", self.t("fim da raiz", "end root")));
                }
            }
            Node::Scripts { base, sup, sub } => self.scripts(base, sup.as_deref(), sub.as_deref()),
            Node::Delimited { open, close, parts, middles } => {
                if *open == Some('|') && *close == Some('|') && parts.len() == 1 {
                    let x = self.spoken(&parts[0]);
                    self.say(self.t(&format!("módulo de {x}"), &format!("absolute value of {x}")));
                    return;
                }
                if let Some(a) = open {
                    let o = operator(*a, Class::Open, self.loc);
                    self.say(o);
                }
                for (i, p) in parts.iter().enumerate() {
                    if i > 0 {
                        let o = operator(middles.get(i - 1).copied().unwrap_or('|'), Class::Rel, self.loc);
                        self.say(o);
                    }
                    self.node(p);
                }
                if let Some(f) = close {
                    let o = operator(*f, Class::Close, self.loc);
                    self.say(o);
                }
            }
            Node::Table { shape: kind, rows, .. } => self.table(*kind, rows),
            Node::Accent { base, accent, .. } => {
                let b = self.spoken(base);
                let phrase = match accent {
                    '\u{20D7}' | '→' => self.t(&format!("vetor {b}"), &format!("vector {b}")),
                    '\u{0302}' => self.t(&format!("{b} chapéu"), &format!("{b} hat")),
                    '\u{0304}' => self.t(&format!("{b} barra"), &format!("{b} bar")),
                    '\u{0307}' => self.t(&format!("{b} ponto"), &format!("{b} dot")),
                    '\u{0308}' => self.t(&format!("{b} dois pontos"), &format!("{b} double dot")),
                    '\u{0303}' => self.t(&format!("{b} til"), &format!("{b} tilde")),
                    '⏜' => self.t(&format!("arco {b}"), &format!("arc {b}")),
                    _ => b,
                };
                self.say(phrase);
            }
            Node::Bar { base, above } => {
                let b = self.spoken(base);
                if *above {
                    self.say(self.t(&format!("{b} barra"), &format!("{b} bar")));
                } else {
                    self.say(b);
                }
            }
            Node::Brace { base, label, .. } => {
                self.node(base);
                if let Some(r) = label {
                    let r = self.spoken(r);
                    self.say(format!("({r})"));
                }
            }
            Node::Text { parts } => {
                for p in parts {
                    match p {
                        TextPart::Text(t) => self.say(t.replace(['\u{E000}', '\u{E001}'], "").trim().to_string()),
                        TextPart::Math(m) => self.node(m),
                    }
                }
            }
            Node::Space { .. } | Node::Phantom { .. } => {}
            Node::Canceled { base, .. } => {
                let b = self.spoken(base);
                self.say(self.t(&format!("{b} cancelado"), &format!("{b} canceled")));
            }
            Node::Framed { base, shape } => {
                let b = self.spoken(base);
                match shape {
                    Frame::Circle => self.say(self.t(&format!("{b} circulado"), &format!("circled {b}"))),
                    Frame::Rectangle => self.say(b),
                }
            }
            Node::Colored { base, .. } | Node::ForcedClass { base, .. } | Node::Style { base, .. } => self.node(base),
            Node::Quantity { unit, .. } => self.node(unit),
            Node::Arrow { symbol, above, below } => {
                // a labeled arrow is a function or a reaction, not a limit: it is not read as "tends to"
                let o = match symbol {
                    '→' => self.t("seta para a direita", "right arrow"),
                    '←' => self.t("seta para a esquerda", "left arrow"),
                    '↔' => self.t("seta dupla", "double arrow"),
                    '⇒' => self.t("implica", "implies"),
                    '=' => self.t("igual", "equals"),
                    other => operator(*other, Class::Rel, self.loc),
                };
                self.say(o);
                if let Some(a) = above {
                    let a = self.spoken(a);
                    self.say(self.t(&format!("({a})"), &format!("({a})")));
                }
                if let Some(b) = below {
                    let b = self.spoken(b);
                    self.say(format!("({b})"));
                }
            }
            Node::Stacked { base, above, below } => {
                self.node(base);
                if let Some(a) = above {
                    let a = self.spoken(a);
                    self.say(self.t(&format!("com {a} em cima"), &format!("with {a} above")));
                }
                if let Some(b) = below {
                    let b = self.spoken(b);
                    self.say(self.t(&format!("com {b} embaixo"), &format!("with {b} below")));
                }
            }
            Node::Multicolumn { base, .. } | Node::Raised { base, .. } | Node::Scaled { base, .. } | Node::Smashed { base, .. } | Node::Overlapped { base, .. } => self.node(base),
            Node::Rule { .. } => {}
            Node::VerticalArrow { symbol, left, right } => {
                let arrow = match symbol {
                    '↓' => self.t("seta para baixo", "down arrow"),
                    '↑' => self.t("seta para cima", "up arrow"),
                    _ => self.t("igual", "equals"),
                };
                self.say(arrow);
                for r in [left, right].into_iter().flatten() {
                    let r = self.spoken(r);
                    self.say(format!("({r})"));
                }
            }
            Node::LongDivision { dividend, divisor } => {
                let (q, r) = divide(dividend, divisor);
                self.say(self.t(
                    &format!("divisão armada de {dividend} por {divisor}: quociente {q}, resto {r}"),
                    &format!("long division of {dividend} by {divisor}: quotient {q}, remainder {r}"),
                ));
            }
            Node::ColumnArithmetic { operation, terms } => {
                let a: i128 = terms[0].parse().unwrap_or(0);
                let b: i128 = terms[1].parse().unwrap_or(0);
                let (op, r) = match operation {
                    '+' => (self.t("mais", "plus"), a + b),
                    '−' => (self.t("menos", "minus"), a - b),
                    _ => (self.t("vezes", "times"), a.saturating_mul(b)),
                };
                self.say(self.t(&format!("conta armada: {a} {op} {b} igual a {r}"), &format!("written calculation: {a} {op} {b} equals {r}")));
            }
        }
    }

    fn scripts(&mut self, base: &Node, sup: Option<&Node>, sub: Option<&Node>) {
        if let Node::LargeOperator { symbol, .. } = base {
            let name = large_operator(*symbol, self.loc);
            let from = sub.map(|s| self.spoken(s));
            let to = sup.map(|s| self.spoken(s));
            let phrase = match (from, to) {
                (Some(d), Some(a)) => self.t(&format!("{name} de {d} até {a} de"), &format!("{name} from {d} to {a} of")),
                (Some(d), None) => self.t(&format!("{name} sobre {d} de"), &format!("{name} over {d} of")),
                (None, Some(a)) => self.t(&format!("{name} até {a} de"), &format!("{name} to {a} of")),
                (None, None) => name,
            };
            self.say(phrase);
            return;
        }
        if let (Node::Function { name, .. }, Some(s)) = (base, sub) {
            if name == "lim" {
                let s = self.spoken(s).replace("tende a", "tendendo a");
                self.say(self.t(&format!("limite com {s} de"), &format!("limit as {s} of")));
                if let Some(p) = sup {
                    let p = self.spoken(p);
                    self.say(p);
                }
                return;
            }
        }
        // sen² x, log₂ x: the scripts belong to the function name, before "de"/"of"
        if let Node::Function { name, .. } = base {
            let spoken = self.spoken(base);
            let (bare, link) = match spoken.rsplit_once(' ') {
                Some((b, l @ ("de" | "of"))) => (b.to_string(), Some(l.to_string())),
                _ => (spoken, None),
            };
            // sen⁻¹ x is the inverse function, not 1/sen x: "arco seno de x", "inverse sine of x"
            let inverse = sup.is_some_and(|s| self.spoken(s) == self.t("menos 1", "negative 1"));
            let trig = matches!(name.as_str(), "sin" | "cos" | "tan" | "cot" | "sec" | "csc");
            let bare = match (inverse, self.loc) {
                (false, _) => bare,
                (true, Locale::En) => format!("inverse {bare}"),
                (true, Locale::PtBr) if trig => format!("arco {bare}"),
                (true, Locale::PtBr) => format!("inversa de {bare}"),
            };
            self.say(bare);
            if let Some(s) = sub {
                let s = self.spoken(s);
                let phrase = if name == "log" { self.t(&format!("na base {s}"), &format!("base {s}")) } else { self.t(&format!("índice {s}"), &format!("sub {s}")) };
                self.say(phrase);
            }
            if let Some(s) = sup.filter(|_| !inverse) {
                let phrase = self.superscript(s);
                self.say(phrase);
            }
            if let Some(l) = link {
                self.say(l);
            }
            return;
        }
        self.node(base);
        if let Some(s) = sub {
            let s = self.spoken(s);
            self.say(self.t(&format!("índice {s}"), &format!("sub {s}")));
        }
        if let Some(s) = sup {
            let phrase = self.superscript(s);
            self.say(phrase);
        }
    }

    fn superscript(&mut self, s: &Node) -> String {
        {
            match s {
                Node::Number { value, .. } if value == "2" => self.t("ao quadrado", "squared"),
                Node::Number { value, .. } if value == "3" => self.t("ao cubo", "cubed"),
                Node::Operator { symbol: '′', .. } => self.t("linha", "prime"),
                Node::Operator { symbol: '″', .. } => self.t("duas linhas", "double prime"),
                Node::Operator { symbol: '°' | '∘', .. } => self.t("graus", "degrees"),
                Node::Operator { symbol: '+', .. } => self.t("pela direita", "from the right"),
                Node::Operator { symbol: '−', .. } => self.t("pela esquerda", "from the left"),
                other => {
                    let x = self.spoken(other);
                    if is_simple(other) {
                        self.t(&format!("elevado a {x}"), &format!("to the {x}"))
                    } else {
                        self.t(&format!("elevado a, {x}, fim do expoente"), &format!("to the power, {x}, end exponent"))
                    }
                }
            }
        }
    }

    fn table(&mut self, kind: TableKind, rows: &[Vec<Node>]) {
        use TableKind::*;
        let ncol = rows.iter().map(Vec::len).max().unwrap_or(0);
        let spoken_rows: Vec<String> =
            rows.iter().map(|l| l.iter().map(|c| self.spoken(c)).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(if kind == Aligned { " " } else { ", " })).collect();
        let body = |label: &str| -> String { spoken_rows.iter().enumerate().map(|(i, l)| format!("{label} {}: {l}", i + 1)).collect::<Vec<_>>().join("; ") };
        let phrase = match kind {
            Matrix | PMatrix | BMatrix | BraceMatrix | VVMatrix | SmallMatrix | BorderMatrix => {
                let r = rows.len();
                format!("{} {r} {} {ncol}: {}", self.t("matriz", "matrix"), self.t("por", "by"), body(&self.t("linha", "row")))
            }
            VMatrix => format!("{}: {}", self.t("determinante", "determinant"), body(&self.t("linha", "row"))),
            Cases => format!("{}: {}", self.t("casos", "cases"), body(&self.t("caso", "case"))),
            Array | Substack => spoken_rows.join("; "),
            Diagram => format!("{}: {}", self.t("diagrama", "diagram"), body(&self.t("linha", "row"))),
            Aligned | Centered => spoken_rows.join("; "),
        };
        self.say(phrase);
    }
}

/// A single token: number, letter, or the same thing with a simple subscript.
fn is_simple(node: &Node) -> bool {
    match node {
        Node::Number { .. } | Node::Identifier { .. } | Node::Operator { .. } => true,
        Node::Row { children } => children.len() == 1 && is_simple(&children[0]),
        Node::Scripts { base, sup, sub } => is_simple(base) && sup.as_deref().is_none_or(is_simple) && sub.as_deref().is_none_or(is_simple),
        _ => false,
    }
}

/// pt-BR: 1/2 → "um meio", 3/4 → "três quartos", the way it is read at school.
fn ordinary_fraction(num: &Node, den: &Node, loc: Locale) -> Option<String> {
    if loc != Locale::PtBr {
        return None;
    }
    let (Node::Number { value: n, .. }, Node::Number { value: d, .. }) = (num, den) else {
        return None;
    };
    let n: usize = n.parse().ok()?;
    let d: usize = d.parse().ok()?;
    if !(1..=10).contains(&n) || !(2..=10).contains(&d) {
        return None;
    }
    let names = ["", "um", "dois", "três", "quatro", "cinco", "seis", "sete", "oito", "nove", "dez"];
    let ordinals = ["", "", "meio", "terço", "quarto", "quinto", "sexto", "sétimo", "oitavo", "nono", "décimo"];
    let plural = if n > 1 { "s" } else { "" };
    Some(format!("{} {}{plural}", names[n], ordinals[d]))
}

fn divide(a: &str, b: &str) -> (String, String) {
    match (a.parse::<u128>(), b.parse::<u128>()) {
        (Ok(a), Ok(b)) if b > 0 => ((a / b).to_string(), (a % b).to_string()),
        _ => ("?".into(), "?".into()),
    }
}

fn large_operator(c: char, loc: Locale) -> String {
    let pt = loc == Locale::PtBr;
    match c {
        '∑' => pt_en(pt, "somatório", "sum"),
        '∏' => pt_en(pt, "produtório", "product"),
        '∫' => "integral",
        '∬' => pt_en(pt, "integral dupla", "double integral"),
        '∭' => pt_en(pt, "integral tripla", "triple integral"),
        '∮' => pt_en(pt, "integral de linha", "contour integral"),
        '⋃' => pt_en(pt, "união", "union"),
        '⋂' => pt_en(pt, "interseção", "intersection"),
        _ => pt_en(pt, "operador", "operator"),
    }
    .to_string()
}

fn letter(name: &str, locale: Locale) -> String {
    let pt = matches!(locale, Locale::PtBr);
    let greek = match name {
        "α" => pt_en(pt, "alfa", "alpha"),
        "β" => "beta",
        "γ" => pt_en(pt, "gama", "gamma"),
        "δ" => "delta",
        "ε" | "ϵ" => pt_en(pt, "épsilon", "epsilon"),
        "ζ" => "zeta",
        "η" => "eta",
        "θ" | "ϑ" => pt_en(pt, "teta", "theta"),
        "κ" => "kappa",
        "λ" => "lambda",
        "μ" => pt_en(pt, "mi", "mu"),
        "ν" => pt_en(pt, "ni", "nu"),
        "ξ" => pt_en(pt, "csi", "xi"),
        "π" => "pi",
        "ρ" => pt_en(pt, "rô", "rho"),
        "σ" => "sigma",
        "τ" => "tau",
        "φ" | "ϕ" => pt_en(pt, "fi", "phi"),
        "χ" => pt_en(pt, "qui", "chi"),
        "ψ" => "psi",
        "ω" => pt_en(pt, "ômega", "omega"),
        "Γ" => pt_en(pt, "gama maiúsculo", "capital gamma"),
        "Δ" => pt_en(pt, "delta maiúsculo", "capital delta"),
        "Θ" => pt_en(pt, "teta maiúsculo", "capital theta"),
        "Λ" => pt_en(pt, "lambda maiúsculo", "capital lambda"),
        "Π" => pt_en(pt, "pi maiúsculo", "capital pi"),
        "Σ" => pt_en(pt, "sigma maiúsculo", "capital sigma"),
        "Φ" => pt_en(pt, "fi maiúsculo", "capital phi"),
        "Ψ" => pt_en(pt, "psi maiúsculo", "capital psi"),
        "Ω" => pt_en(pt, "ômega maiúsculo", "capital omega"),
        _ => return name.to_string(),
    };
    greek.to_string()
}

fn operator(symbol: char, class: Class, locale: Locale) -> String {
    let pt = matches!(locale, Locale::PtBr);
    let text = match symbol {
        '+' => pt_en(pt, "mais", "plus"),
        '−' => {
            if pt {
                "menos"
            } else if class == Class::Ord {
                "negative"
            } else {
                "minus"
            }
        }
        '±' => pt_en(pt, "mais ou menos", "plus or minus"),
        '∓' => pt_en(pt, "menos ou mais", "minus or plus"),
        '·' | '×' | '∗' => pt_en(pt, "vezes", "times"),
        '/' | '÷' => pt_en(pt, "dividido por", "divided by"),
        '=' => pt_en(pt, "é igual a", "equals"),
        '<' => pt_en(pt, "é menor que", "is less than"),
        '>' => pt_en(pt, "é maior que", "is greater than"),
        '≤' => pt_en(pt, "é menor ou igual a", "is less than or equal to"),
        '≥' => pt_en(pt, "é maior ou igual a", "is greater than or equal to"),
        '≠' => pt_en(pt, "é diferente de", "is not equal to"),
        '≈' => pt_en(pt, "é aproximadamente", "is approximately"),
        '≡' => pt_en(pt, "é côngruo a", "is congruent to"),
        '∝' => pt_en(pt, "é proporcional a", "is proportional to"),
        '→' | '⟶' => pt_en(pt, "tende a", "tends to"),
        '⇒' | '⟹' => pt_en(pt, "implica", "implies"),
        '⇔' | '⟺' => pt_en(pt, "se e somente se", "if and only if"),
        '⇌' => pt_en(pt, "em equilíbrio com", "in equilibrium with"),
        '∈' => pt_en(pt, "pertence a", "is in"),
        '∉' => pt_en(pt, "não pertence a", "is not in"),
        '⊂' => pt_en(pt, "está contido em", "is a subset of"),
        '⊆' => pt_en(pt, "está contido ou é igual a", "is a subset of or equal to"),
        '∪' => pt_en(pt, "união", "union"),
        '∩' => pt_en(pt, "interseção", "intersection"),
        '∞' => pt_en(pt, "infinito", "infinity"),
        '∂' => pt_en(pt, "derivada parcial", "partial"),
        '∇' => "nabla",
        '∀' => pt_en(pt, "para todo", "for all"),
        '∃' => pt_en(pt, "existe", "there exists"),
        '∅' => pt_en(pt, "conjunto vazio", "empty set"),
        '(' => pt_en(pt, "abre parênteses", "open paren"),
        ')' => pt_en(pt, "fecha parênteses", "close paren"),
        '[' => {
            if pt {
                if class == Class::Close {
                    "fecha intervalo"
                } else {
                    "abre colchetes"
                }
            } else {
                "open bracket"
            }
        }
        ']' => {
            if pt {
                if class == Class::Open {
                    "abre intervalo"
                } else {
                    "fecha colchetes"
                }
            } else {
                "close bracket"
            }
        }
        '{' => pt_en(pt, "abre chaves", "open brace"),
        '}' => pt_en(pt, "fecha chaves", "close brace"),
        ',' => pt_en(pt, "vírgula", "comma"),
        ';' => pt_en(pt, "ponto e vírgula", "semicolon"),
        ':' => pt_en(pt, "dois-pontos", "colon"),
        '!' => pt_en(pt, "fatorial", "factorial"),
        '|' | '∣' => pt_en(pt, "barra", "bar"),
        '°' => pt_en(pt, "graus", "degrees"),
        '%' => pt_en(pt, "por cento", "percent"),
        '…' | '⋯' => pt_en(pt, "e assim por diante", "and so on"),
        '∠' => pt_en(pt, "ângulo", "angle"),
        '△' => pt_en(pt, "triângulo", "triangle"),
        '⊥' => pt_en(pt, "é perpendicular a", "is perpendicular to"),
        '∥' => pt_en(pt, "é paralelo a", "is parallel to"),
        '′' => pt_en(pt, "linha", "prime"),
        other => return other.to_string(),
    };
    text.to_string()
}

/// The pt-BR or the English word, for the symbol tables above.
fn pt_en(pt: bool, pt_br: &'static str, en: &'static str) -> &'static str {
    if pt {
        pt_br
    } else {
        en
    }
}
