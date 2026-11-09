// The palettes that come with the editor. Each button inserts a snippet of LaTeX (`#?` an empty
// place, `#0` where the selection goes) or runs an action; mathra draws it from the same snippet.

import type { Words } from "./shared.ts";

export interface PaletteItem {
  /** LaTeX to insert at the cursor: `#?` is an empty place, `#0` where the selection goes. */
  insert?: string;
  /** An action of the field instead: `sup`, `sub`, `addRow`, `moveLeft`, `undo`… (see `MathraField.run`). */
  action?: string;
  /** The formula drawn on the button (LaTeX); by default the snippet, its places drawn empty. */
  preview?: string;
  /** Text on the button instead of a drawing. */
  label?: string;
  /** An icon on the button instead of a drawing (see `icon`). */
  icon?: string;
  /** The button's name, shown as its tooltip and read by screen readers. */
  title: Words;
  /** The name typed after `\` that does the same, for the command menu. */
  command?: string;
}

export interface PaletteSet {
  id: string;
  title: Words;
  items: PaletteItem[];
}

const t = (en: string, pt: string): Words => ({ en, "pt-BR": pt });

/** A symbol button: its command inserts it and names it in the command menu. */
const sym = (latex: string, en: string, pt: string): PaletteItem => ({ insert: latex, title: t(en, pt), command: /^\\[A-Za-z]+$/.test(latex) ? latex.slice(1) : undefined });

export const STRUCTURES: PaletteSet = {
  id: "structures",
  title: t("Structures", "Estruturas"),
  items: [
    { insert: "\\frac{#0}{#?}", title: t("Fraction", "Fração"), command: "frac" },
    { insert: "\\sqrt{#0}", title: t("Square root", "Raiz quadrada"), command: "sqrt" },
    { insert: "\\sqrt[#?]{#0}", title: t("n-th root", "Raiz de índice n"), command: "root" },
    { action: "sup", preview: "#?^{#?}", title: t("Exponent", "Expoente"), command: "sup" },
    { action: "sub", preview: "#?_{#?}", title: t("Index", "Índice") },
    { insert: "^{#?}_{#?}", preview: "#?^{#?}_{#?}", title: t("Exponent and index", "Expoente e índice") },
    { insert: "\\left(#0\\right)", preview: "\\left(#?\\right)", title: t("Parentheses", "Parênteses"), command: "paren" },
    { insert: "\\left[#0\\right]", preview: "\\left[#?\\right]", title: t("Brackets", "Colchetes"), command: "brackets" },
    { insert: "\\left\\{#0\\right\\}", preview: "\\left\\{#?\\right\\}", title: t("Braces", "Chaves"), command: "braces" },
    { insert: "\\left|#0\\right|", preview: "\\left|#?\\right|", title: t("Absolute value", "Módulo"), command: "abs" },
    { insert: "\\left\\|#0\\right\\|", preview: "\\left\\|#?\\right\\|", title: t("Norm", "Norma"), command: "norm" },
    { insert: "\\binom{#0}{#?}", title: t("Binomial coefficient", "Número binomial"), command: "binom" },
    { insert: "\\dfrac{#0}{#?}", title: t("Display fraction", "Fração grande"), command: "dfrac" },
    { insert: "\\text{#0}", preview: "\\text{abc}", title: t("Text", "Texto"), command: "text" },
    { insert: "\\overline{#0}", title: t("Bar over", "Barra em cima"), command: "overline" },
    { insert: "\\vec{#0}", title: t("Vector", "Vetor"), command: "vec" },
  ],
};

export const CALCULUS: PaletteSet = {
  id: "calculus",
  title: t("Calculus", "Cálculo"),
  items: [
    { insert: "\\sum_{#?}^{#?}#0", preview: "\\sum_{#?}^{#?}", title: t("Sum", "Somatório"), command: "sum" },
    { insert: "\\prod_{#?}^{#?}#0", preview: "\\prod_{#?}^{#?}", title: t("Product", "Produtório"), command: "prod" },
    { insert: "\\int_{#?}^{#?}#0", preview: "\\int_{#?}^{#?}", title: t("Integral", "Integral"), command: "int" },
    { insert: "\\int #0\\,dx", preview: "\\int #?\\,dx", title: t("Indefinite integral", "Integral indefinida") },
    { insert: "\\iint_{#?}#0", preview: "\\iint_{#?}", title: t("Double integral", "Integral dupla"), command: "iint" },
    { insert: "\\oint_{#?}#0", preview: "\\oint_{#?}", title: t("Contour integral", "Integral de linha"), command: "oint" },
    { insert: "\\lim_{#?}#0", preview: "\\lim_{x\\to a}", title: t("Limit", "Limite"), command: "lim" },
    { insert: "\\frac{d}{dx}#0", preview: "\\frac{d}{dx}", title: t("Derivative", "Derivada") },
    { insert: "\\frac{dy}{dx}", title: t("Derivative of y", "Derivada de y") },
    { insert: "\\frac{\\partial}{\\partial x}#0", preview: "\\frac{\\partial}{\\partial x}", title: t("Partial derivative", "Derivada parcial") },
    sym("\\partial", "Partial", "Parcial"),
    sym("\\nabla", "Nabla", "Nabla"),
    sym("\\infty", "Infinity", "Infinito"),
    sym("\\to", "Tends to", "Tende a"),
    sym("\\Delta", "Delta", "Delta"),
    sym("\\mathrm{d}", "Upright d", "d reto"),
  ],
};

export const MATRICES: PaletteSet = {
  id: "matrices",
  title: t("Matrices", "Matrizes"),
  items: [
    { insert: "\\begin{pmatrix}#?&#?\\\\#?&#?\\end{pmatrix}", title: t("Matrix in parentheses", "Matriz entre parênteses"), command: "pmatrix" },
    { insert: "\\begin{bmatrix}#?&#?\\\\#?&#?\\end{bmatrix}", title: t("Matrix in brackets", "Matriz entre colchetes"), command: "bmatrix" },
    { insert: "\\begin{vmatrix}#?&#?\\\\#?&#?\\end{vmatrix}", title: t("Determinant", "Determinante"), command: "vmatrix" },
    { insert: "\\begin{Bmatrix}#?&#?\\\\#?&#?\\end{Bmatrix}", title: t("Matrix in braces", "Matriz entre chaves"), command: "Bmatrix" },
    { insert: "\\begin{matrix}#?&#?\\\\#?&#?\\end{matrix}", title: t("Matrix without delimiters", "Matriz sem delimitadores"), command: "matrix" },
    { insert: "\\begin{pmatrix}#?\\\\#?\\end{pmatrix}", title: t("Column vector", "Vetor coluna") },
    { insert: "\\begin{cases}#?&#?\\\\#?&#?\\end{cases}", title: t("Cases", "Casos"), command: "cases" },
    { insert: "\\begin{aligned}#?&#?\\\\#?&#?\\end{aligned}", title: t("Aligned lines", "Linhas alinhadas"), command: "aligned" },
    { action: "addRow", icon: "rowAdd", title: t("Add a line below", "Adicionar linha abaixo") },
    { action: "addColumn", icon: "columnAdd", title: t("Add a column to the right", "Adicionar coluna à direita") },
    { action: "removeRow", icon: "rowRemove", title: t("Remove this line", "Remover esta linha") },
    { action: "removeColumn", icon: "columnRemove", title: t("Remove this column", "Remover esta coluna") },
  ],
};

const GREEK: [string, string, string][] = [
  ["alpha", "alpha", "alfa"], ["beta", "beta", "beta"], ["gamma", "gamma", "gama"], ["delta", "delta", "delta"], ["epsilon", "epsilon", "épsilon"], ["varepsilon", "epsilon (variant)", "épsilon (variante)"],
  ["zeta", "zeta", "zeta"], ["eta", "eta", "eta"], ["theta", "theta", "teta"], ["iota", "iota", "iota"], ["kappa", "kappa", "capa"], ["lambda", "lambda", "lambda"],
  ["mu", "mu", "mi"], ["nu", "nu", "ni"], ["xi", "xi", "csi"], ["pi", "pi", "pi"], ["rho", "rho", "rô"], ["sigma", "sigma", "sigma"],
  ["tau", "tau", "tau"], ["upsilon", "upsilon", "ípsilon"], ["phi", "phi", "fi"], ["varphi", "phi (variant)", "fi (variante)"], ["chi", "chi", "qui"], ["psi", "psi", "psi"], ["omega", "omega", "ômega"],
  ["Gamma", "Gamma", "Gama maiúsculo"], ["Delta", "Delta", "Delta maiúsculo"], ["Theta", "Theta", "Teta maiúsculo"], ["Lambda", "Lambda", "Lambda maiúsculo"], ["Xi", "Xi", "Csi maiúsculo"],
  ["Pi", "Pi", "Pi maiúsculo"], ["Sigma", "Sigma", "Sigma maiúsculo"], ["Phi", "Phi", "Fi maiúsculo"], ["Psi", "Psi", "Psi maiúsculo"], ["Omega", "Omega", "Ômega maiúsculo"],
];

export const GREEK_LETTERS: PaletteSet = {
  id: "greek",
  title: t("Greek", "Gregas"),
  items: GREEK.map(([name, en, pt]) => sym(`\\${name}`, en, pt)),
};

export const OPERATORS: PaletteSet = {
  id: "operators",
  title: t("Operators", "Operadores"),
  items: [
    sym("+", "Plus", "Mais"), sym("-", "Minus", "Menos"), sym("\\pm", "Plus or minus", "Mais ou menos"), sym("\\mp", "Minus or plus", "Menos ou mais"),
    sym("\\times", "Times", "Vezes"), sym("\\div", "Divided by", "Dividido por"), sym("\\cdot", "Dot", "Ponto"), sym("\\circ", "Composition", "Composição"),
    sym("\\ast", "Asterisk", "Asterisco"), sym("\\cup", "Union", "União"), sym("\\cap", "Intersection", "Interseção"), sym("\\setminus", "Set difference", "Diferença"),
    sym("\\oplus", "Direct sum", "Soma direta"), sym("\\otimes", "Tensor product", "Produto tensorial"), sym("\\wedge", "And", "E"), sym("\\vee", "Or", "Ou"),
    sym("\\neg", "Not", "Não"), sym("!", "Factorial", "Fatorial"), sym("\\%", "Percent", "Por cento"), sym("^{\\circ}", "Degree", "Grau"),
  ],
};

export const RELATIONS: PaletteSet = {
  id: "relations",
  title: t("Relations", "Relações"),
  items: [
    sym("=", "Equals", "Igual"), sym("\\ne", "Not equal", "Diferente"), sym("<", "Less than", "Menor que"), sym(">", "Greater than", "Maior que"),
    sym("\\le", "Less or equal", "Menor ou igual"), sym("\\ge", "Greater or equal", "Maior ou igual"), sym("\\approx", "Approximately", "Aproximadamente"), sym("\\equiv", "Equivalent", "Equivalente"),
    sym("\\sim", "Similar", "Semelhante"), sym("\\cong", "Congruent", "Congruente"), sym("\\propto", "Proportional", "Proporcional"), sym("\\in", "Belongs to", "Pertence"),
    sym("\\notin", "Does not belong", "Não pertence"), sym("\\subset", "Subset", "Subconjunto"), sym("\\subseteq", "Subset or equal", "Contido ou igual"), sym("\\supset", "Superset", "Contém"),
    sym("\\parallel", "Parallel", "Paralelo"), sym("\\perp", "Perpendicular", "Perpendicular"), sym("\\mid", "Divides", "Divide"), sym("\\ll", "Much less", "Muito menor"),
  ],
};

export const ARROWS: PaletteSet = {
  id: "arrows",
  title: t("Arrows", "Setas"),
  items: [
    sym("\\to", "Right arrow", "Seta para a direita"), sym("\\gets", "Left arrow", "Seta para a esquerda"), sym("\\leftrightarrow", "Both ways", "Seta dupla"),
    sym("\\Rightarrow", "Implies", "Implica"), sym("\\Leftarrow", "Is implied by", "É implicado por"), sym("\\Leftrightarrow", "If and only if", "Se e somente se"),
    sym("\\mapsto", "Maps to", "Leva em"), sym("\\uparrow", "Up arrow", "Seta para cima"), sym("\\downarrow", "Down arrow", "Seta para baixo"),
    sym("\\longrightarrow", "Long arrow", "Seta longa"), sym("\\Longrightarrow", "Long implies", "Implica (longa)"), sym("\\rightleftharpoons", "Equilibrium", "Equilíbrio"),
  ],
};

export const SETS: PaletteSet = {
  id: "sets",
  title: t("Sets and logic", "Conjuntos e lógica"),
  items: [
    sym("\\forall", "For all", "Para todo"), sym("\\exists", "There exists", "Existe"), sym("\\nexists", "There is no", "Não existe"), sym("\\emptyset", "Empty set", "Conjunto vazio"),
    sym("\\mathbb{N}", "Natural numbers", "Naturais"), sym("\\mathbb{Z}", "Integers", "Inteiros"), sym("\\mathbb{Q}", "Rationals", "Racionais"), sym("\\mathbb{R}", "Reals", "Reais"),
    sym("\\mathbb{C}", "Complex numbers", "Complexos"), sym("\\therefore", "Therefore", "Portanto"), sym("\\because", "Because", "Porque"), sym("\\ldots", "Dots", "Reticências"),
    sym("\\cdots", "Centered dots", "Reticências centradas"), sym("\\vdots", "Vertical dots", "Reticências verticais"), sym("\\ddots", "Diagonal dots", "Reticências diagonais"), sym("\\ell", "Script l", "l cursivo"),
    sym("\\hbar", "h-bar", "h cortado"), sym("\\angle", "Angle", "Ângulo"), sym("\\triangle", "Triangle", "Triângulo"), sym("\\degree", "Degree", "Grau"),
  ],
};

export const FUNCTIONS: PaletteSet = {
  id: "functions",
  title: t("Functions", "Funções"),
  items: [
    sym("\\sin", "Sine", "Seno"), sym("\\cos", "Cosine", "Cosseno"), sym("\\tan", "Tangent", "Tangente"), sym("\\cot", "Cotangent", "Cotangente"),
    sym("\\sec", "Secant", "Secante"), sym("\\csc", "Cosecant", "Cossecante"), sym("\\arcsin", "Arcsine", "Arco seno"), sym("\\arccos", "Arccosine", "Arco cosseno"),
    sym("\\arctan", "Arctangent", "Arco tangente"), sym("\\sinh", "Hyperbolic sine", "Seno hiperbólico"), sym("\\cosh", "Hyperbolic cosine", "Cosseno hiperbólico"), sym("\\exp", "Exponential", "Exponencial"),
    sym("\\log", "Logarithm", "Logaritmo"), { insert: "\\log_{#?}#0", preview: "\\log_{#?}", title: t("Logarithm in base", "Logaritmo na base") }, sym("\\ln", "Natural logarithm", "Logaritmo natural"), sym("\\det", "Determinant", "Determinante"),
    { insert: "\\max_{#?}#0", preview: "\\max_{#?}", title: t("Maximum", "Máximo"), command: "max" }, { insert: "\\min_{#?}#0", preview: "\\min_{#?}", title: t("Minimum", "Mínimo"), command: "min" },
    { insert: "f\\left(#0\\right)", preview: "f(#?)", title: t("Function of", "Função de") }, { insert: "\\operatorname{sen}", title: t("Sine (pt-BR)", "Seno (sen)") },
  ],
};

export const ACCENTS: PaletteSet = {
  id: "accents",
  title: t("Accents and styles", "Acentos e estilos"),
  items: [
    { insert: "\\vec{#0}", title: t("Vector", "Vetor"), command: "vec" },
    { insert: "\\hat{#0}", title: t("Hat", "Chapéu"), command: "hat" },
    { insert: "\\bar{#0}", title: t("Bar", "Barra"), command: "bar" },
    { insert: "\\dot{#0}", title: t("Dot", "Ponto"), command: "dot" },
    { insert: "\\ddot{#0}", title: t("Two dots", "Dois pontos"), command: "ddot" },
    { insert: "\\tilde{#0}", title: t("Tilde", "Til"), command: "tilde" },
    { insert: "\\overline{#0}", title: t("Line over", "Linha em cima"), command: "overline" },
    { insert: "\\underline{#0}", title: t("Line under", "Linha embaixo"), command: "underline" },
    { insert: "\\overrightarrow{#0}", title: t("Arrow over", "Seta em cima"), command: "overrightarrow" },
    { insert: "\\overbrace{#0}^{#?}", title: t("Brace over", "Chave em cima") },
    { insert: "\\underbrace{#0}_{#?}", title: t("Brace under", "Chave embaixo") },
    { insert: "\\boxed{#0}", title: t("Box", "Caixa"), command: "boxed" },
    { insert: "\\cancel{#0}", title: t("Cancel", "Cancelar"), command: "cancel" },
    { insert: "\\mathbf{#0}", preview: "\\mathbf{A}", title: t("Bold", "Negrito"), command: "mathbf" },
    { insert: "\\mathbb{#0}", preview: "\\mathbb{A}", title: t("Blackboard bold", "Duplo"), command: "mathbb" },
    { insert: "\\mathcal{#0}", preview: "\\mathcal{A}", title: t("Calligraphic", "Caligráfico"), command: "mathcal" },
    { insert: "\\mathfrak{#0}", preview: "\\mathfrak{A}", title: t("Fraktur", "Fraktur"), command: "mathfrak" },
    { insert: "\\mathrm{#0}", preview: "\\mathrm{A}", title: t("Upright", "Reto"), command: "mathrm" },
  ],
};

/** Every palette, in the order a toolbar shows them. */
export const PALETTES: PaletteSet[] = [STRUCTURES, CALCULUS, MATRICES, GREEK_LETTERS, OPERATORS, RELATIONS, ARROWS, SETS, FUNCTIONS, ACCENTS];

/** The formula drawn on an item's button: its preview, or its snippet, with empty places drawn as □. */
export function previewLatex(item: PaletteItem): string {
  let latex = item.preview ?? item.insert ?? "";
  if (/^[\^_]/.test(latex)) latex = `#?${latex}`;
  return latex.replace(/#[?0]/g, "\\htmlData{empty=1}{\\square}");
}

/** Every command a palette names, for the command menu: the name, what it does, how it looks. */
export function commandCatalog(sets: PaletteSet[] = PALETTES): { name: string; item: PaletteItem }[] {
  const seen = new Set<string>();
  const out: { name: string; item: PaletteItem }[] = [];
  for (const set of sets)
    for (const item of set.items) {
      if (!item.command || seen.has(item.command)) continue;
      seen.add(item.command);
      out.push({ name: item.command, item });
    }
  return out;
}
