//! The README gallery: famous equations and constructions that show what mathra draws.
//! Each `docs/examples/<name>.svg` must be exactly what the current code produces, on a white card so
//! it stays readable on dark pages. After a layout change:
//! `MATHRA_UPDATE_EXAMPLES=1 cargo test -p mathra --test examples`.

use mathra::{convert, Locale, Options};

pub struct Example {
    pub name: &'static str,
    pub locale: Locale,
    pub caption: &'static str,
    pub tex: &'static str,
}

pub const EXAMPLES: &[Example] = &[
    Example { name: "euler", locale: Locale::En, caption: "Euler's identity", tex: r"e^{i\pi} + 1 = 0" },
    Example { name: "gaussian", locale: Locale::En, caption: "Gaussian integral", tex: r"\int_{-\infty}^{\infty} e^{-x^2}\,dx = \sqrt{\pi}" },
    Example { name: "basel", locale: Locale::En, caption: "Basel problem", tex: r"\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}" },
    Example { name: "quadratic", locale: Locale::En, caption: "Quadratic formula", tex: r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}" },
    Example {
        name: "maxwell",
        locale: Locale::En,
        caption: "Maxwell's equations",
        tex: r"\begin{aligned} \nabla \cdot \vec{E} &= \frac{\rho}{\varepsilon_0} & \nabla \cdot \vec{B} &= 0 \\ \nabla \times \vec{E} &= -\frac{\partial \vec{B}}{\partial t} & \nabla \times \vec{B} &= \mu_0 \vec{J} + \mu_0 \varepsilon_0 \frac{\partial \vec{E}}{\partial t} \end{aligned}",
    },
    Example {
        name: "schrodinger",
        locale: Locale::En,
        caption: "Schrödinger equation",
        tex: r"i\hbar \frac{\partial}{\partial t} \Psi(\mathbf{r}, t) = \left[ -\frac{\hbar^2}{2m} \nabla^2 + V(\mathbf{r}, t) \right] \Psi(\mathbf{r}, t)",
    },
    Example {
        name: "einstein",
        locale: Locale::En,
        caption: "Einstein field equations",
        tex: r"R_{\mu\nu} - \frac{1}{2} R\, g_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}",
    },
    Example { name: "fourier", locale: Locale::En, caption: "Fourier transform", tex: r"\hat{f}(\xi) = \int_{-\infty}^{\infty} f(x)\, e^{-2\pi i x \xi}\, dx" },
    Example { name: "cauchy", locale: Locale::En, caption: "Cauchy's integral formula", tex: r"f(a) = \frac{1}{2\pi i} \oint_{\gamma} \frac{f(z)}{z - a}\, dz" },
    Example { name: "bayes", locale: Locale::En, caption: "Bayes' theorem", tex: r"P(A \mid B) = \frac{P(B \mid A)\, P(A)}{P(B)}" },
    Example { name: "golden", locale: Locale::En, caption: "Golden ratio as a continued fraction", tex: r"\varphi = 1 + \cfrac{1}{1 + \cfrac{1}{1 + \cfrac{1}{1 + \cdots}}}" },
    Example {
        name: "matrix",
        locale: Locale::En,
        caption: "m × n matrix with dots",
        tex: r"A = \begin{pmatrix} a_{11} & a_{12} & \cdots & a_{1n} \\ a_{21} & a_{22} & \cdots & a_{2n} \\ \vdots & \vdots & \ddots & \vdots \\ a_{m1} & a_{m2} & \cdots & a_{mn} \end{pmatrix}",
    },
    Example { name: "piecewise", locale: Locale::En, caption: "Piecewise definition", tex: r"|x| = \begin{cases} x & \text{if } x \ge 0 \\ -x & \text{if } x < 0 \end{cases}" },
    Example { name: "binomial", locale: Locale::En, caption: "Binomial theorem", tex: r"(x + y)^n = \sum_{k=0}^{n} \binom{n}{k} x^{n-k} y^{k}" },
    Example { name: "braces", locale: Locale::En, caption: "Labeled brace", tex: r"\underbrace{1 + 2 + \cdots + n}_{n \text{ terms}} = \frac{n(n+1)}{2}" },
    Example { name: "chemistry", locale: Locale::En, caption: "Chemical equation (mhchem)", tex: r"\ce{CH4 + 2 O2 -> CO2 + 2 H2O}" },
    Example { name: "units", locale: Locale::En, caption: "Quantity with units (siunitx)", tex: r"g = \SI{9.81}{\meter\per\second\squared}" },
    Example { name: "diagram", locale: Locale::En, caption: "Commutative diagram (amscd)", tex: r"\begin{CD} A @>f>> B \\ @VgVV @VVhV \\ C @>k>> D \end{CD}" },
    Example {
        name: "tikzcd",
        locale: Locale::En,
        caption: "Pullback with bent arrows (tikz-cd)",
        tex: r#"\begin{tikzcd} T \arrow[drr, bend left, "x"] \arrow[ddr, bend right, "y"'] \arrow[dr, dashed, "\exists!" description] & & \\ & P \arrow[r, "q"] \arrow[d, "p"'] \arrow[dr, phantom, "\lrcorner" very near start] & X \arrow[d, "f"] \\ & Y \arrow[r, "g"'] & Z \end{tikzcd}"#,
    },
    Example {
        name: "xymatrix",
        locale: Locale::En,
        caption: "Short exact sequence (xymatrix)",
        tex: r"\xymatrix{ 0 \ar[r] & A \ar@{^{(}->}[r]^{\iota} \ar[d]_{\alpha} & B \ar@{->>}[r]^{\pi} \ar[d]^{\beta} & C \ar[r] \ar@{=}[d] & 0 \\ 0 \ar[r] & A' \ar[r] & B' \ar[r] & C \ar[r] & 0 }",
    },
    Example {
        name: "ptbr",
        locale: Locale::PtBr,
        caption: "pt-BR locale: sen, decimal comma, open interval",
        tex: r"\sen^2 \theta + \cos^2 \theta = 1,\quad x \in \left]0{,}5;\, 2{,}5\right[",
    },
    Example { name: "division", locale: Locale::PtBr, caption: "Long division, computed (pt-BR)", tex: r"\divchave{247}{5}" },
];

/// Padding around the equation, in font units (1000 per em).
const PADDING: f32 = 320.0;
/// Pixels per em in the README.
const PX_PER_EM: f32 = 30.0;

fn number(v: f32) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// The equation's SVG with padding, a white card behind it and a size in px.
fn on_card(svg: &str) -> String {
    let attr = |name: &str| svg.split(&format!(" {name}=\"")).nth(1).unwrap().split('"').next().unwrap().to_string();
    let v: Vec<f32> = attr("viewBox").split(' ').map(|n| n.parse().unwrap()).collect();
    let (x, y, w, h) = (v[0] - PADDING, v[1] - PADDING, v[2] + 2.0 * PADDING, v[3] + 2.0 * PADDING);
    let px = |units: f32| number(units / 1000.0 * PX_PER_EM);
    let svg = svg
        .replacen(&format!(" viewBox=\"{}\"", attr("viewBox")), &format!(" viewBox=\"{} {} {} {}\"", number(x), number(y), number(w), number(h)), 1)
        .replacen(&format!(" width=\"{}\"", attr("width")), &format!(" width=\"{}\"", px(w)), 1)
        .replacen(&format!(" height=\"{}\"", attr("height")), &format!(" height=\"{}\"", px(h)), 1);
    let open_end = svg.find('>').unwrap() + 1;
    let card = format!(r##"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="#ffffff"/>"##, number(x), number(y), number(w), number(h), number(PADDING / 2.0));
    format!("{}{card}{}", &svg[..open_end], &svg[open_end..])
}

#[test]
fn committed_examples_match_the_code() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/examples");
    let update = std::env::var_os("MATHRA_UPDATE_EXAMPLES").is_some();
    for e in EXAMPLES {
        // the gallery shows valid LaTeX: strict mode must accept every example
        let r = convert(e.tex, &Options { locale: e.locale, strict: true, ..Options::default() }).unwrap_or_else(|err| panic!("{}: {err}", e.name));
        let svg = on_card(&r.svg);
        let path = format!("{dir}/{}.svg", e.name);
        if update {
            std::fs::write(&path, &svg).unwrap();
        }
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(committed == svg, "docs/examples/{}.svg is stale: run MATHRA_UPDATE_EXAMPLES=1 cargo test -p mathra --test examples", e.name);
    }
}

#[test]
fn every_committed_example_is_in_the_list() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/examples");
    for f in std::fs::read_dir(dir).unwrap() {
        let name = f.unwrap().file_name().into_string().unwrap();
        let name = name.trim_end_matches(".svg");
        assert!(EXAMPLES.iter().any(|e| e.name == name), "docs/examples/{name}.svg is not generated by this test");
    }
}
