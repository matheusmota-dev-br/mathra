//! Nemeth Braille Code for Mathematics and Science Notation (BANA, 2022). Section numbers (§) refer to
//! that document; `spec/braille.md` lists what is covered and what is not.

use super::{cells, letter, Braille};
use crate::tree::*;

pub fn transcribe(node: &Node) -> Braille {
    let mut w = Writer::default();
    w.node(node, &Ctx::default());
    w.finish()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Number,
    /// A letter written alone, which may need the English-letter indicator (§6.3).
    Letter,
    Comparison,
    Function,
    Punctuation,
    Minus,
    Indicator,
    /// Opening and closing grouping signs.
    Open,
    Close,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
enum Item {
    Cells(String, Kind),
    /// A space; `true` after a function name, where a letter takes no English-letter indicator (§6.4.2).
    Space(bool),
    Newline,
}

#[derive(Default)]
struct Writer {
    items: Vec<Item>,
    /// The level of what was written last: "" is the baseline, otherwise its level indicator (§14).
    level: String,
    missing: Vec<char>,
}

#[derive(Debug, Clone, Default)]
struct Ctx {
    /// Level indicator of this part of the expression: "" for the baseline, ⠘ superscript, ⠰ subscript…
    level: String,
    /// Inside an enclosed list (§3.5): no numeric indicator after its spaces, no English-letter indicator.
    list: bool,
    /// Inside a matrix or a system: a numeric indicator on every number (§25.8.1).
    table: bool,
    /// Radicals already open around this one (§16: the inner ones take ⠨ per depth).
    radicals: usize,
}

impl Ctx {
    fn at(&self, level: String) -> Ctx {
        Ctx { level, ..self.clone() }
    }
}

const SUPERSCRIPT: &str = "⠘";
const SUBSCRIPT: &str = "⠰";

impl Writer {
    fn last(&self) -> Option<&Item> {
        self.items.last()
    }

    /// Cells at a level: when the level changes, its indicator first (§14); ⠐ returns to the baseline.
    fn put(&mut self, ctx: &Ctx, text: &str, kind: Kind) {
        if ctx.level != self.level {
            let indicator = if ctx.level.is_empty() { "⠐".to_string() } else { ctx.level.clone() };
            self.items.push(Item::Cells(indicator, Kind::Indicator));
            self.level = ctx.level.clone();
        }
        self.items.push(Item::Cells(text.to_string(), kind));
    }

    fn put_dots(&mut self, ctx: &Ctx, dots: &str, kind: Kind) {
        self.put(ctx, &cells(dots), kind);
    }

    /// A space. After it, what follows is read at `level` with no indicator: the baseline after a
    /// comparison sign (§14.8.7), the level of a function name after the name (§14.8).
    fn space_to(&mut self, level: &str) {
        self.space(level, false);
    }

    fn space(&mut self, level: &str, after_function: bool) {
        if !matches!(self.last(), None | Some(Item::Space(_) | Item::Newline)) {
            self.items.push(Item::Space(after_function));
        }
        self.level = level.to_string();
    }

    fn missing(&mut self, ctx: &Ctx, c: char) {
        if !self.missing.contains(&c) {
            self.missing.push(c);
        }
        self.put(ctx, "⠿", Kind::Other);
    }

    /// Numeric indicator (§3.3): at the start of a line, after a space or a punctuation mark, after a minus
    /// that starts a line or follows a space, and on every number of a matrix.
    fn needs_numeric_indicator(&self, ctx: &Ctx) -> bool {
        if ctx.table {
            return true;
        }
        let n = self.items.len();
        match self.last() {
            None | Some(Item::Newline) => true,
            Some(Item::Space(_)) => !ctx.list,
            Some(Item::Cells(_, Kind::Punctuation)) => true,
            Some(Item::Cells(_, Kind::Minus)) => n < 2 || matches!(self.items[n - 2], Item::Space(_) | Item::Newline),
            _ => false,
        }
    }

    fn number(&mut self, value: &str, ctx: &Ctx) {
        let mut s = String::new();
        if self.needs_numeric_indicator(ctx) && ctx.level == self.level {
            s.push('⠼');
        }
        for c in value.chars() {
            s.push_str(&match c {
                '.' => cells("46"),
                d => cells(digit(d)),
            });
        }
        self.put(ctx, &s, Kind::Number);
    }

    fn finish(mut self) -> Braille {
        self.english_letter_indicators();
        while matches!(self.items.last(), Some(Item::Space(_) | Item::Newline)) {
            self.items.pop();
        }
        let text = self
            .items
            .iter()
            .map(|i| match i {
                Item::Cells(c, _) => c.clone(),
                Item::Space(_) => "⠀".to_string(),
                Item::Newline => "\n".to_string(),
            })
            .collect();
        Braille { text, missing: self.missing }
    }

    /// §6.3: a single letter standing alone takes the English-letter indicator ⠰: a space, a punctuation
    /// mark or the edge on both sides; a grouping sign counts on one side only, as in `(a, 2x, y = z)`
    /// (Ex. 6-30) but not in `(x)`. Never next to a comparison sign (§6.4.7) or after a function name (§6.4.2).
    fn english_letter_indicators(&mut self) {
        let items = self.items.clone();
        let before = |i: usize| -> Option<bool> {
            if i == 0 {
                return Some(false);
            }
            match &items[i - 1] {
                Item::Newline | Item::Cells(_, Kind::Punctuation) => Some(false),
                Item::Space(after_function) => {
                    (!after_function && !matches!(items.get(i.wrapping_sub(2)), Some(Item::Cells(_, Kind::Comparison | Kind::Function)))).then_some(false)
                }
                Item::Cells(_, Kind::Open) => Some(true),
                _ => None,
            }
        };
        let after = |i: usize| -> Option<bool> {
            match items.get(i + 1) {
                None | Some(Item::Newline) | Some(Item::Cells(_, Kind::Punctuation)) => Some(false),
                Some(Item::Space(_)) => (!matches!(items.get(i + 2), Some(Item::Cells(_, Kind::Comparison)))).then_some(false),
                Some(Item::Cells(_, Kind::Close)) => Some(true),
                _ => None,
            }
        };
        for (i, item) in self.items.iter_mut().enumerate() {
            if let Item::Cells(text, Kind::Letter) = item {
                // Some(grouping?) on each side; grouping signs on both sides do not count
                if let (Some(a), Some(b)) = (before(i), after(i)) {
                    if !(a && b) {
                        text.insert(0, '⠰');
                    }
                }
            }
        }
    }

    // ---------- nodes ----------

    fn node(&mut self, node: &Node, ctx: &Ctx) {
        match node {
            Node::Number { value, .. } => self.number(value, ctx),
            Node::Identifier { name, variant } => self.identifier(name, *variant, ctx),
            Node::Function { name, .. } => self.function_name(name, ctx),
            Node::Operator { symbol, .. } | Node::FixedDelimiter { symbol, .. } => self.symbol(*symbol, ctx),
            Node::LargeOperator { symbol, .. } => self.large_operator(*symbol, ctx),
            Node::Row { children } => self.row(children, ctx),
            Node::Fraction { num, den, rule, .. } => self.fraction(num, den, *rule, ctx),
            Node::Root { radicand, index } => self.root(radicand, index.as_deref(), ctx),
            Node::Scripts { base, sup, sub } => self.scripts(base, sup.as_deref(), sub.as_deref(), ctx),
            Node::Delimited { open, close, parts, middles } => self.delimited(*open, *close, parts, middles, ctx),
            Node::Table { shape, rows, .. } => self.table(*shape, rows, ctx),
            Node::Accent { base, accent, below, .. } => self.accent(base, *accent, *below, ctx),
            Node::Bar { base, above } => self.accent(base, '\u{0304}', !above, ctx),
            Node::Brace { base, label, .. } => {
                self.node(base, ctx);
                if label.is_some() {
                    self.missing(ctx, '⏞');
                }
            }
            Node::Text { parts } => {
                for p in parts {
                    match p {
                        TextPart::Text(t) => self.text(t, ctx),
                        TextPart::Math(m) => self.node(m, ctx),
                    }
                }
            }
            Node::Arrow { symbol, above, below } => self.labeled_arrow(*symbol, above.as_deref(), below.as_deref(), ctx),
            Node::Stacked { base, above, below } => self.modified(base, below.as_deref(), above.as_deref(), ctx),
            Node::Framed { base, .. }
            | Node::Canceled { base, .. }
            | Node::Colored { base, .. }
            | Node::ForcedClass { base, .. }
            | Node::Style { base, .. }
            | Node::Multicolumn { base, .. }
            | Node::Raised { base, .. }
            | Node::Scaled { base, .. }
            | Node::Smashed { base, .. }
            | Node::Overlapped { base, .. }
            | Node::Quantity { unit: base, .. } => self.node(base, ctx),
            Node::Space { .. } | Node::Phantom { .. } | Node::Rule { .. } => {}
            Node::VerticalArrow { symbol, .. } => self.missing(ctx, *symbol),
            Node::LongDivision { dividend, divisor } => {
                self.number(dividend, ctx);
                self.put_dots(ctx, "46 34", Kind::Other);
                self.number(divisor, ctx);
            }
            Node::ColumnArithmetic { operation, terms } => {
                for (i, t) in terms.iter().enumerate() {
                    if i > 0 {
                        self.symbol(*operation, ctx);
                    }
                    self.number(t, ctx);
                }
            }
        }
    }

    fn row(&mut self, children: &[Node], ctx: &Ctx) {
        let lists = enclosed_lists(children);
        let mut i = 0;
        while i < children.len() {
            let child = &children[i];
            let list_ctx;
            let ctx = if lists[i] && !ctx.list {
                list_ctx = Ctx { list: true, ..ctx.clone() };
                &list_ctx
            } else {
                ctx
            };
            // a mixed number: a whole number followed by a fraction of numbers (§13.4)
            if let (Node::Number { value, .. }, Some(Node::Fraction { num, den, rule: true, .. })) = (child, children.get(i + 1)) {
                if matches!(**num, Node::Number { .. }) && matches!(**den, Node::Number { .. }) && !value.contains('.') {
                    self.number(value, ctx);
                    self.put_dots(ctx, "456 1456", Kind::Other);
                    self.node(num, ctx);
                    self.put_dots(ctx, "34", Kind::Other);
                    self.node(den, ctx);
                    self.put_dots(ctx, "456 3456", Kind::Other);
                    i += 2;
                    continue;
                }
            }
            self.node(child, ctx);
            // a function name is followed by a space (§18), unless punctuation or a closing sign follows
            if is_function(child) && children.get(i + 1).is_some_and(|n| !matches!(n, Node::Operator { class: Class::Punct | Class::Close, .. })) {
                self.space(&ctx.level, true);
            }
            i += 1;
        }
    }

    fn identifier(&mut self, name: &str, variant: Variant, ctx: &Ctx) {
        let mut chars = name.chars();
        let (Some(c), None) = (chars.next(), chars.next()) else {
            // a word: its letters, uncontracted
            self.text(name, ctx);
            return;
        };
        let Some(base) = self.letter_cells(c) else {
            self.missing(ctx, c);
            return;
        };
        // §7: a typeform indicator, then the alphabetic indicator
        let prefix = match variant {
            Variant::Bold | Variant::BoldItalic => "⠸⠰",
            Variant::DoubleStruck => "⠠⠸⠰",
            Variant::Calligraphic => "⠈⠰",
            _ => "",
        };
        let kind = if prefix.is_empty() && c.is_ascii_alphabetic() && !ctx.list && !ctx.table { Kind::Letter } else { Kind::Other };
        self.put(ctx, &format!("{prefix}{base}"), kind);
    }

    /// A Latin letter (capitals with ⠠) or a Greek one (⠨, capitals ⠨⠠, alternative forms ⠨⠈), §6.1.
    fn letter_cells(&self, c: char) -> Option<String> {
        if c.is_ascii_alphabetic() {
            let base = cells(letter(c)?);
            return Some(if c.is_ascii_uppercase() { format!("⠠{base}") } else { base });
        }
        let (dots, capital, alternative) = greek(c)?;
        Some(format!("⠨{}{}{}", if alternative { "⠈" } else { "" }, if capital { "⠠" } else { "" }, cells(dots)))
    }

    fn function_name(&mut self, name: &str, ctx: &Ctx) {
        let letters: String = name.chars().filter(|c| !c.is_whitespace()).filter_map(|c| letter(c).map(cells)).collect();
        self.put(ctx, &letters, Kind::Function);
    }

    /// Words inside the equation: letters uncontracted, capitals with ⠠, numbers with their indicator.
    fn text(&mut self, t: &str, ctx: &Ctx) {
        let mut word = String::new();
        let mut digits = String::new();
        let flush_word = |w: &mut Self, word: &mut String| {
            if !word.is_empty() {
                w.put(ctx, word, Kind::Other);
                word.clear();
            }
        };
        for c in t.chars().filter(|c| !matches!(c, '\u{E000}'..='\u{E003}')) {
            if c.is_ascii_digit() {
                flush_word(self, &mut word);
                digits.push(c);
                continue;
            }
            if !digits.is_empty() {
                self.number(&digits, ctx);
                digits.clear();
            }
            if c.is_whitespace() {
                flush_word(self, &mut word);
                self.space_to(&ctx.level);
            } else if let Some(l) = self.letter_cells(c) {
                word.push_str(&l);
            } else {
                flush_word(self, &mut word);
                self.symbol(c, ctx);
            }
        }
        flush_word(self, &mut word);
        if !digits.is_empty() {
            self.number(&digits, ctx);
        }
    }

    fn symbol(&mut self, c: char, ctx: &Ctx) {
        if let Some(l) = greek(c).and_then(|_| self.letter_cells(c)) {
            self.put(ctx, &l, Kind::Other);
            return;
        }
        let Some((dots, kind)) = sign(c) else {
            self.missing(ctx, c);
            return;
        };
        match kind {
            // comparison signs are spaced (§21.13)
            Kind::Comparison => {
                self.space_to("");
                self.put_dots(ctx, dots, kind);
                self.space_to("");
            }
            // the mathematical comma: ⠪ at script level (§14), and a space after it on the baseline
            Kind::Punctuation if c == ',' => {
                if ctx.level.is_empty() {
                    self.put_dots(ctx, dots, kind);
                    self.space_to("");
                } else {
                    self.put_dots(ctx, "246", Kind::Other);
                }
            }
            // other punctuation returns to the baseline (§14.8)
            Kind::Punctuation => {
                self.put_dots(ctx, dots, kind);
                self.level = String::new();
                if matches!(c, ';' | ':') {
                    self.space_to("");
                }
            }
            _ => self.put_dots(ctx, dots, kind),
        }
    }

    fn large_operator(&mut self, c: char, ctx: &Ctx) {
        let dots = match c {
            '∑' => "46 6 234",
            '∏' => "46 6 1234",
            '∫' => "2346",
            '∬' => "2346 2346",
            '∭' => "2346 2346 2346",
            '∮' => "2346 4 1246 14 12456",
            other => {
                self.missing(ctx, other);
                return;
            }
        };
        self.put_dots(ctx, dots, Kind::Other);
    }

    fn fraction(&mut self, num: &Node, den: &Node, rule: bool, ctx: &Ctx) {
        if !rule {
            // binomial coefficient: upper part, directly-under indicator, lower part (§15.6)
            self.node(num, ctx);
            self.put_dots(ctx, "146", Kind::Other);
            self.node(den, ctx);
            return;
        }
        // simple ⠹⠌⠼, complex ⠠⠹⠠⠌⠠⠼, hypercomplex ⠠⠠⠹… (§13)
        let order = fraction_depth(num).max(fraction_depth(den));
        let prefix = "⠠".repeat(order);
        let inner = Ctx { list: false, ..ctx.clone() };
        self.put(ctx, &format!("{prefix}⠹"), Kind::Other);
        self.node(num, &inner);
        self.put(ctx, &format!("{prefix}⠌"), Kind::Other);
        self.node(den, &inner);
        self.put(ctx, &format!("{prefix}⠼"), Kind::Other);
    }

    fn root(&mut self, radicand: &Node, index: Option<&Node>, ctx: &Ctx) {
        // §16: index-of-radical ⠣, radical ⠜, termination ⠻; inner radicals take ⠨ per depth
        let order = "⠨".repeat(ctx.radicals);
        if let Some(i) = index {
            self.put(ctx, "⠣", Kind::Other);
            self.node(i, ctx);
        }
        self.put(ctx, &format!("{order}⠜"), Kind::Other);
        let inner = Ctx { radicals: ctx.radicals + 1, ..ctx.clone() };
        self.node(radicand, &inner);
        self.put(ctx, &format!("{order}⠻"), Kind::Other);
    }

    fn scripts(&mut self, base: &Node, sup: Option<&Node>, sub: Option<&Node>, ctx: &Ctx) {
        // Σ, ∏, lim, max…: limits under and over, as a modified expression (§15)
        let limits = match base {
            Node::LargeOperator { symbol, .. } => !matches!(symbol, '∫' | '∬' | '∭' | '∮'),
            Node::Function { limits, .. } => *limits && sub.is_some(),
            _ => false,
        };
        if limits {
            self.modified(base, sub, sup, ctx);
            return;
        }
        self.node(base, ctx);
        // primes come first, at the level of the base (§14.12)
        let (primes, sup) = split_primes(sup);
        if primes > 0 {
            self.put(ctx, &"⠄".repeat(primes), Kind::Other);
        }
        if let Some(s) = sub {
            let numeric = matches!(s, Node::Number { value, .. } if !value.contains('.'));
            let on_letter = matches!(base, Node::Identifier { name, .. } if name.chars().count() == 1) || matches!(base, Node::Function { .. });
            if numeric && on_letter && ctx.level.is_empty() {
                // x₁: a numeric subscript of first order needs no indicator (§14.6)
                if let Node::Number { value, .. } = s {
                    let digits: String = value.chars().map(|d| cells(digit(d))).collect();
                    self.put(ctx, &digits, Kind::Other);
                }
            } else {
                self.node(s, &ctx.at(format!("{}{SUBSCRIPT}", ctx.level)));
            }
        }
        if let Some(s) = sup {
            let level = ctx.at(format!("{}{SUPERSCRIPT}", ctx.level));
            if matches!(s, Node::Operator { symbol: '°' | '∘', .. }) {
                // the degree sign is written at superscript level (§23.10)
                self.put_dots(&level, "46 16", Kind::Other);
            } else {
                self.node(&s, &level);
            }
        }
    }

    /// Five steps (§15): ⠐, the expression, ⠩ under / ⠣ over (the under part first), the modifier, ⠻.
    fn modified(&mut self, base: &Node, under: Option<&Node>, over: Option<&Node>, ctx: &Ctx) {
        self.put(ctx, "⠐", Kind::Indicator);
        self.node(base, ctx);
        if let Some(u) = under {
            self.put(ctx, "⠩", Kind::Other);
            self.node(u, ctx);
        }
        if let Some(o) = over {
            self.put(ctx, "⠣", Kind::Other);
            self.node(o, ctx);
        }
        self.put(ctx, "⠻", Kind::Other);
    }

    fn accent(&mut self, base: &Node, accent: char, below: bool, ctx: &Ctx) {
        let modifier = match accent {
            '\u{0304}' => "156",
            '\u{0302}' => "456 126",
            '\u{20D7}' | '→' => "1246 135",
            '\u{0307}' => "16",
            '\u{0303}' => "4 156",
            other => {
                self.node(base, ctx);
                self.missing(ctx, other);
                return;
            }
        };
        // x̄: a bar over a single letter or digit is contracted (§15.3)
        let single = matches!(base, Node::Identifier { name, .. } if name.chars().count() == 1) || matches!(base, Node::Number { value, .. } if value.len() == 1);
        if accent == '\u{0304}' && !below && single {
            self.node(base, ctx);
            self.put_dots(ctx, "156", Kind::Other);
            return;
        }
        self.put(ctx, "⠐", Kind::Indicator);
        self.node(base, ctx);
        self.put(ctx, if below { "⠩" } else { "⠣" }, Kind::Other);
        self.put_dots(ctx, modifier, Kind::Other);
        self.put(ctx, "⠻", Kind::Other);
    }

    fn labeled_arrow(&mut self, symbol: char, above: Option<&Node>, below: Option<&Node>, ctx: &Ctx) {
        if above.is_none() && below.is_none() {
            self.symbol(symbol, ctx);
            return;
        }
        // a modified arrow takes its uncontracted form (§22)
        let dots = match symbol {
            '→' => "1246 25 25 135",
            '←' => "1246 246 25 25",
            '↔' => "1246 246 25 25 135",
            '⇒' => "1246 2356 2356 135",
            '⇔' => "1246 246 2356 2356 135",
            other => {
                self.missing(ctx, other);
                return;
            }
        };
        self.space_to("");
        self.put(ctx, "⠐", Kind::Indicator);
        self.put_dots(ctx, dots, Kind::Comparison);
        if let Some(b) = below {
            self.put(ctx, "⠩", Kind::Other);
            self.node(b, ctx);
        }
        if let Some(a) = above {
            self.put(ctx, "⠣", Kind::Other);
            self.node(a, ctx);
        }
        self.put(ctx, "⠻", Kind::Other);
        self.space_to("");
    }

    fn delimited(&mut self, open: Option<char>, close: Option<char>, parts: &[Node], middles: &[char], ctx: &Ctx) {
        // an enclosed list: its items have no numeric or English-letter indicators (§3.5)
        let list = parts.iter().any(|p| {
            let c = p.children();
            c.iter().any(|c| matches!(c, Node::Operator { symbol: ',', .. })) && !c.iter().any(|c| matches!(c, Node::Operator { class: Class::Rel, .. }))
        });
        let inner = Ctx { list: ctx.list || list, ..ctx.clone() };
        if let Some(o) = open {
            self.symbol(o, ctx);
        }
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                self.symbol(middles.get(i - 1).copied().unwrap_or('|'), &inner);
            }
            self.node(p, &inner);
        }
        if let Some(c) = close {
            self.symbol(c, ctx);
        }
    }

    /// Matrices, determinants and systems (§19.6, §25.8): a braille line per row, enlarged grouping signs
    /// on every line, one blank cell between entries, a numeric indicator on every number.
    fn table(&mut self, shape: TableKind, rows: &[Vec<Node>], ctx: &Ctx) {
        let (open, close) = match shape {
            TableKind::PMatrix => ("6 12356", "6 23456"),
            TableKind::BMatrix => ("4 6 12356", "4 6 23456"),
            TableKind::BraceMatrix => ("46 6 12356", "46 6 23456"),
            TableKind::VMatrix => ("6 1256", "6 1256"),
            TableKind::VVMatrix => ("6 1256 6 1256", "6 1256 6 1256"),
            TableKind::Cases => ("46 6 12356", ""),
            TableKind::RCases => ("", "46 6 23456"),
            _ => ("", ""),
        };
        let inner = Ctx { table: true, list: false, ..ctx.clone() };
        if !self.items.is_empty() {
            self.items.push(Item::Newline);
        }
        for (r, row) in rows.iter().enumerate() {
            if r > 0 {
                self.items.push(Item::Newline);
            }
            self.level = ctx.level.clone();
            if !open.is_empty() {
                self.put_dots(ctx, open, Kind::Other);
            }
            for (j, cell) in row.iter().enumerate() {
                if j > 0 {
                    self.space_to(&ctx.level);
                }
                self.node(cell, &inner);
            }
            if !close.is_empty() {
                self.put_dots(ctx, close, Kind::Other);
            }
        }
        self.items.push(Item::Newline);
        self.level = ctx.level.clone();
    }
}

fn digit(d: char) -> &'static str {
    match d {
        '1' => "2",
        '2' => "23",
        '3' => "25",
        '4' => "256",
        '5' => "26",
        '6' => "235",
        '7' => "2356",
        '8' => "236",
        '9' => "35",
        _ => "356",
    }
}

/// Greek letters (§6.1.4, §6.1.5): (dots of the letter, capital, alternative form).
fn greek(c: char) -> Option<(&'static str, bool, bool)> {
    let lower = c.to_lowercase().next()?;
    let capital = c != lower;
    let (dots, alternative) = match lower {
        'α' => ("1", false),
        'β' => ("12", false),
        'γ' => ("1245", false),
        'δ' => ("145", false),
        'ϵ' | 'ε' => ("15", false),
        'ζ' => ("1356", false),
        'η' => ("156", false),
        'θ' => ("1456", false),
        'ϑ' => ("1456", true),
        'ι' => ("24", false),
        'κ' => ("13", false),
        'λ' => ("123", false),
        'μ' => ("134", false),
        'ν' => ("1345", false),
        'ξ' => ("1346", false),
        'ο' => ("135", false),
        'π' => ("1234", false),
        'ρ' => ("1235", false),
        'σ' => ("234", false),
        'ς' => ("234", true),
        'τ' => ("2345", false),
        'υ' => ("136", false),
        'ϕ' => ("124", false),
        'φ' => ("124", true),
        'χ' => ("12346", false),
        'ψ' => ("13456", false),
        'ω' => ("2456", false),
        _ => return None,
    };
    Some((dots, capital, alternative))
}

/// Signs that are one symbol: operations (§20, §23), comparisons (§21, §22), grouping (§19) and
/// punctuation (§8).
fn sign(c: char) -> Option<(&'static str, Kind)> {
    use Kind::*;
    Some(match c {
        '+' => ("346", Other),
        '−' | '-' => ("36", Minus),
        '×' => ("4 16", Other),
        '·' | '⋅' => ("16", Other),
        '÷' => ("46 34", Other),
        '±' => ("346 36", Other),
        '∓' => ("36 346", Other),
        '!' => ("12346", Other),
        '%' => ("4 356", Other),
        '∪' => ("46 346", Other),
        '∩' => ("46 146", Other),
        '/' => ("456 34", Other),
        '=' => ("46 13", Comparison),
        '≠' => ("34 46 13", Comparison),
        '<' => ("5 13", Comparison),
        '>' => ("46 2", Comparison),
        '≤' => ("5 13 156", Comparison),
        '≥' => ("46 2 156", Comparison),
        '≡' => ("456 123", Comparison),
        '∼' => ("4 156", Comparison),
        '≈' => ("4 156 4 156", Comparison),
        '∝' => ("456 123456", Comparison),
        '∈' => ("4 15", Comparison),
        '∉' => ("34 4 15", Comparison),
        '⊂' => ("456 5 13", Comparison),
        '⊆' => ("456 5 13 156", Comparison),
        '→' | '⟶' => ("1246 135", Comparison),
        '←' | '⟵' => ("1246 246 25 25", Comparison),
        '↔' | '⟷' => ("1246 246 25 25 135", Comparison),
        '⇒' | '⟹' => ("1246 2356 2356 135", Comparison),
        '⇔' | '⟺' => ("1246 246 2356 2356 135", Comparison),
        // negation: / before the comparison sign (§21.1)
        '≮' => ("34 5 13", Comparison),
        '≯' => ("34 46 2", Comparison),
        '≰' => ("34 5 13 156", Comparison),
        '≱' => ("34 46 2 156", Comparison),
        '≢' => ("34 456 123", Comparison),
        '≁' => ("34 4 156", Comparison),
        '≉' => ("34 4 156 4 156", Comparison),
        '⊄' => ("34 456 5 13", Comparison),
        '⊈' => ("34 456 5 13 156", Comparison),
        '↛' => ("34 1246 135", Comparison),
        '∅' => ("456 356", Other),
        '∞' => ("6 123456", Other),
        '∂' => ("4 145", Other),
        '∇' | '𝛁' => ("46 1246", Other),
        '∴' => ("6 16", Other),
        '∀' => ("4 12346", Other),
        '∃' => ("4 123456", Other),
        '′' | '\'' => ("3", Other),
        '″' => ("3 3", Other),
        '…' | '⋯' => ("3 3 3", Other),
        '(' => ("12356", Open),
        ')' => ("23456", Close),
        '[' => ("4 12356", Open),
        ']' => ("4 23456", Close),
        '{' => ("46 12356", Open),
        '}' => ("46 23456", Close),
        '|' => ("1256", Other),
        '‖' => ("1256 1256", Other),
        '⟨' => ("46 46 12356", Open),
        '⟩' => ("46 46 23456", Close),
        ',' => ("6", Punctuation),
        ';' => ("456 23", Punctuation),
        ':' => ("456 25", Punctuation),
        '.' => ("456 256", Punctuation),
        _ => return None,
    })
}

fn is_function(n: &Node) -> bool {
    match n {
        Node::Function { .. } => true,
        Node::Scripts { base, .. } => matches!(**base, Node::Function { .. }),
        _ => false,
    }
}

/// Which children of a row lie inside an enclosed list (§3.5): grouping signs that hold commas at their
/// own depth, as in `[0, 1]` or `(a, 2x, b)`.
fn enclosed_lists(children: &[Node]) -> Vec<bool> {
    let mut inside = vec![false; children.len()];
    let class = |n: &Node| match n {
        Node::Operator { class, .. } => Some(*class),
        _ => None,
    };
    for open in 0..children.len() {
        if class(&children[open]) != Some(Class::Open) {
            continue;
        }
        let mut depth = 0;
        let mut comma = false;
        for close in open + 1..children.len() {
            match class(&children[close]) {
                Some(Class::Open) => depth += 1,
                Some(Class::Close) if depth > 0 => depth -= 1,
                Some(Class::Close) => {
                    if comma {
                        inside[open + 1..close].iter_mut().for_each(|b| *b = true);
                    }
                    break;
                }
                Some(Class::Punct) if depth == 0 && matches!(children[close], Node::Operator { symbol: ',', .. }) => comma = true,
                // a comparison sign inside makes it not an enclosed list (Ex. 6-30)
                Some(Class::Rel) if depth == 0 => break,
                _ => {}
            }
        }
    }
    inside
}

/// How deep the fractions inside go: 0 for a simple fraction's parts.
fn fraction_depth(n: &Node) -> usize {
    match n {
        Node::Fraction { num, den, rule: true, .. } => 1 + fraction_depth(num).max(fraction_depth(den)),
        Node::Row { children } => children.iter().map(fraction_depth).max().unwrap_or(0),
        Node::Delimited { parts, .. } => parts.iter().map(fraction_depth).max().unwrap_or(0),
        Node::Root { radicand, .. } => fraction_depth(radicand),
        // a fraction at script level does not make the outer one complex (§13.5)
        Node::Scripts { base, .. } => fraction_depth(base),
        Node::Colored { base, .. } | Node::ForcedClass { base, .. } | Node::Style { base, .. } | Node::Framed { base, .. } => fraction_depth(base),
        _ => 0,
    }
}

/// The primes at the start of a superscript, and what remains of it.
fn split_primes(sup: Option<&Node>) -> (usize, Option<Node>) {
    let count = |n: &Node| match n {
        Node::Operator { symbol: '′', .. } => 1,
        Node::Operator { symbol: '″', .. } => 2,
        Node::Operator { symbol: '‴', .. } => 3,
        _ => 0,
    };
    match sup {
        Some(n) if count(n) > 0 => (count(n), None),
        Some(Node::Row { children }) if children.first().is_some_and(|c| count(c) > 0) => {
            let k = children.iter().take_while(|c| count(c) > 0).count();
            let primes = children[..k].iter().map(count).sum();
            let rest = &children[k..];
            (primes, if rest.is_empty() { None } else { Some(Node::row(rest.to_vec()).simplify()) })
        }
        other => (0, other.cloned()),
    }
}
