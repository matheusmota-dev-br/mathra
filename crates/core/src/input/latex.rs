//! Reader for math LaTeX: the commands and environments listed in the README, plus mathra's extensions
//! (`\divchave`, `\contaarmada`).

use crate::locale::Locale;
use crate::rules::{self, Registry};
use crate::tree::*;
use crate::Error;

use super::symbols::{self, Symbol};

pub fn read(tex: &str, locale: Locale, registry: &mut Registry) -> Result<Node, Error> {
    read_with(tex, locale, registry, true)
}

/// `strict`: refuse invalid LaTeX like TeX does. Without it, recover what is possible and record warnings.
pub fn read_with(tex: &str, locale: Locale, registry: &mut Registry, strict: bool) -> Result<Node, Error> {
    let clean = prepare(tex, registry, strict);
    let mut macros = Macros { max_size: clean.chars().count() + MAX_GROWTH, ..Macros::default() };
    let mut reader = Reader { c: clean.chars().collect(), pos: 0, locale, registry, macros: &mut macros, variant: Variant::Italic, strict };
    let body = reader.top()?;
    Ok(body)
}

/// Substitution macros defined in the equation itself (`\newcommand`, `\def`, `\let`…).
/// There are no conditionals and no `\catcode`: mathra expands macros, it does not run TeX programs.
#[derive(Default)]
struct Macros {
    commands: std::collections::HashMap<String, Macro>,
    environments: std::collections::HashMap<String, (String, String)>,
    expansions: usize,
    /// How much the text may grow through expansions, counting the original size.
    max_size: usize,
    /// The too-many-expansions warning was already given (there is one, not one per discarded expansion).
    warned_excess: bool,
    /// Groups, commands and scripts open right now: protects the stack from inputs like `{{{{…`.
    depth: usize,
}

#[derive(Clone)]
struct Macro {
    arguments: usize,
    default_arg: Option<String>,
    body: String,
    /// `\DeclarePairedDelimiter`: opening and closing delimiters.
    paired: Option<(String, String)>,
}

/// More than this is almost certainly a macro that calls itself.
const EXPANSION_LIMIT: usize = 2000;
/// Characters the expansions may add to the original text.
const MAX_GROWTH: usize = 200_000;
/// Group, command and script levels open at the same time. Each level costs ~10 KB of stack unoptimized, so 64 fit in 1 MB.
pub const NESTING_LIMIT: usize = 64;
/// Columns of an `array`, even with a repeated `*{n}{…}`.
const MAX_COLUMNS: usize = 1000;

/// Bold markers inside the text of `\text{…}` (Unicode private use area).
pub const BOLD_START: char = '\u{E000}';
pub const BOLD_END: char = '\u{E001}';

/// L-ENT-1 and L-ENT-2: HTML entities and outer delimiters that arrive with the equation.
fn prepare(tex: &str, registry: &mut Registry, strict: bool) -> String {
    // the repair comes before the trim: a \frac eaten by JSON starts with a form feed, which trim would delete
    let mut s = tex.to_string();
    if !strict {
        if let Some(repaired) = repair_controls(&s) {
            registry.warn(rules::E_REC_2, "JSON control character in place of the backslash (\\frac stored unescaped): command rebuilt");
            s = repaired;
        }
    }
    s = s.trim().to_string();
    if !strict {
        if let Some(undone) = undo_doubled_backslashes(&s) {
            registry.warn(rules::E_REC_2, "doubled backslashes (LaTeX escaped twice): one level undone");
            s = undone;
        }
        // a lone backslash at the end is left over from cut text
        let trailing_backslashes = s.chars().rev().take_while(|c| *c == '\\').count();
        if trailing_backslashes % 2 == 1 {
            registry.warn(rules::E_REC_9, "lone '\\' at the end: ignored");
            s.pop();
        }
    }
    for (open, close) in [("$$", "$$"), ("\\[", "\\]"), ("\\(", "\\)"), ("$", "$")] {
        if s.len() >= open.len() + close.len() && s.starts_with(open) && s.ends_with(close) {
            s = s[open.len()..s.len() - close.len()].trim().to_string();
            registry.note(rules::L_ENT_2, format!("delimitadores externos {open}…{close} removidos"));
            break;
        }
    }
    s = s.replace("\\begingroup", "{").replace("\\endgroup", "}");
    let entities = [("&lt;", "<"), ("&gt;", ">"), ("&amp;", "&"), ("&nbsp;", "~"), ("&quot;", "\""), ("&#39;", "'")];
    for (ent, swap) in entities {
        if s.contains(ent) {
            s = s.replace(ent, swap);
            registry.note(rules::L_ENT_1, format!("HTML entity {ent} read as '{swap}'"));
        }
    }
    s
}

#[derive(Debug, Clone, PartialEq)]
enum Stop {
    Eof,
    CloseBrace,
    E,
    NewRow,
    End(String),
    Right(Option<char>),
    Middle(Option<char>),
    Hline,
    Cline(usize, usize),
    /// `\intertext{…}` inside an alignment.
    Intertext(Node),
}

struct Reader<'a> {
    c: Vec<char>,
    pos: usize,
    locale: Locale,
    registry: &'a mut Registry,
    macros: &'a mut Macros,
    /// Current variant, changed by `\bf`, `\rm`, `\mathrm{…}`.
    variant: Variant,
    strict: bool,
}

type R<T> = Result<T, Error>;

impl Reader<'_> {
    // ---------- utilities ----------

    fn current(&self) -> Option<char> {
        self.c.get(self.pos).copied()
    }

    fn em(&self, d: usize) -> Option<char> {
        self.c.get(self.pos + d).copied()
    }

    fn error<T>(&self, column: usize, message: impl Into<String>) -> R<T> {
        Err(Error::Syntax { column, message: message.into() })
    }

    /// In strict mode, an error; in tolerant mode, a warning, and the caller's recovery goes on.
    fn tolerate(&mut self, rule: &'static str, column: usize, problem: impl Into<String>, recovery: &str) -> R<()> {
        let problem = problem.into();
        if self.strict {
            return Err(Error::Syntax { column, message: problem });
        }
        self.registry.warn(rule, format!("column {column}: {problem}; {recovery}"));
        Ok(())
    }

    fn column(&self) -> usize {
        self.pos + 1
    }

    /// Runs `f` one level deeper. Going past the limit is an error in both modes: there is nothing to recover.
    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> R<T>) -> R<T> {
        if self.macros.depth >= NESTING_LIMIT {
            return self.error(self.column(), format!("nesting deeper than {NESTING_LIMIT} levels"));
        }
        self.macros.depth += 1;
        let r = f(self);
        self.macros.depth -= 1;
        r
    }

    fn skip_spaces(&mut self) {
        while self.current().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    fn read_command_name(&mut self) -> String {
        // we are already past the backslash
        let mut name = String::new();
        while let Some(c) = self.current().filter(|c| c.is_ascii_alphabetic()) {
            name.push(c);
            self.pos += 1;
        }
        if name.is_empty() {
            if let Some(c) = self.current() {
                name.push(c);
                self.pos += 1;
            }
        }
        name
    }

    /// Raw content between braces, honoring inner braces. Accepts a single character without braces.
    fn raw_text(&mut self) -> R<String> {
        self.skip_spaces();
        let col = self.column();
        match self.current() {
            Some('{') => {
                self.pos += 1;
                let mut level = 1;
                let mut s = String::new();
                while let Some(c) = self.current() {
                    self.pos += 1;
                    match c {
                        '\\' => {
                            s.push(c);
                            if let Some(d) = self.current() {
                                s.push(d);
                                self.pos += 1;
                            }
                            continue;
                        }
                        '{' => level += 1,
                        '}' => {
                            level -= 1;
                            if level == 0 {
                                return Ok(s);
                            }
                        }
                        _ => {}
                    }
                    s.push(c);
                }
                self.tolerate(rules::E_REC_4, col, "'{' without a matching '}'", "closed at the end")?;
                Ok(s)
            }
            Some(c) => {
                self.pos += 1;
                Ok(c.to_string())
            }
            None => {
                self.tolerate(rules::E_REC_7, col, "missing argument", "left empty")?;
                Ok(String::new())
            }
        }
    }

    fn raw_optional(&mut self) -> Option<String> {
        let saved = self.pos;
        self.skip_spaces();
        if self.current() != Some('[') {
            self.pos = saved;
            return None;
        }
        self.pos += 1;
        let mut level = 0;
        let mut s = String::new();
        while let Some(c) = self.current() {
            self.pos += 1;
            match c {
                '{' => level += 1,
                '}' => level -= 1,
                ']' if level == 0 => return Some(s),
                _ => {}
            }
            s.push(c);
        }
        self.pos = saved;
        None
    }

    /// Reads a raw snippet as math, with the current variant.
    fn sub_reader(&mut self, tex: &str, variant: Variant) -> R<Node> {
        let mut sub = Reader { c: tex.chars().collect(), pos: 0, locale: self.locale, registry: self.registry, macros: self.macros, variant, strict: self.strict };
        let (items, stop) = sub.list()?;
        match stop {
            Stop::Eof => Ok(Node::row(items).simplify()),
            other => {
                let col = sub.column();
                sub.tolerate(rules::E_REC_5, col, format!("unexpected closing: {other:?}"), "ignored")?;
                Ok(Node::row(items).simplify())
            }
        }
    }

    // ---------- structure ----------

    /// Whole body. Loose `\\` and `&` become rows and columns of an alignment.
    fn top(&mut self) -> R<Node> {
        let mut rows: Vec<Vec<Node>> = Vec::new();
        let mut cells: Vec<Node> = Vec::new();
        let mut had_e = false;
        loop {
            let (items, stop) = self.list()?;
            cells.push(Node::row(items).simplify());
            match stop {
                Stop::Eof => break,
                Stop::E => had_e = true,
                Stop::NewRow => rows.push(std::mem::take(&mut cells)),
                Stop::CloseBrace => self.tolerate(rules::E_REC_4, self.pos, "'}' without a matching '{'", "ignored")?,
                Stop::End(n) => self.tolerate(rules::E_REC_5, self.pos, format!("\\end{{{n}}} without a matching \\begin"), "ignored")?,
                Stop::Right(d) => {
                    self.tolerate(rules::E_REC_5, self.pos, "\\right without a matching \\left", "delimiter drawn alone")?;
                    if let Some(d) = d {
                        cells.push(Node::Operator { symbol: d, class: Class::Close });
                    }
                }
                Stop::Middle(_) => self.tolerate(rules::E_REC_5, self.pos, "\\middle outside \\left…\\right", "ignored")?,
                Stop::Hline | Stop::Cline(..) => {}
                Stop::Intertext(t) => cells.push(t),
            }
        }
        if rows.is_empty() && !had_e {
            return Ok(Node::row(cells.into_iter().flat_map(undo_line).collect()));
        }
        if !cells.iter().all(empty) || rows.is_empty() {
            rows.push(cells);
        }
        let kind = if had_e { TableKind::Aligned } else { TableKind::Centered };
        Ok(Node::Table { shape: kind, rows, columns: vec![], verticals: vec![], horizontals: vec![], between: vec![], intertexts: vec![] })
    }

    /// Reads atoms up to a stop (end, `}`, `&`, `\\`, `\end`, `\right`…).
    fn list(&mut self) -> R<(Vec<Node>, Stop)> {
        self.nested(Self::list_unguarded)
    }

    fn list_unguarded(&mut self) -> R<(Vec<Node>, Stop)> {
        let mut items: Vec<Node> = Vec::new();
        loop {
            self.skip_spaces();
            let Some(c) = self.current() else {
                return Ok((items, Stop::Eof));
            };
            match c {
                '}' => {
                    self.pos += 1;
                    return Ok((items, Stop::CloseBrace));
                }
                '&' => {
                    self.pos += 1;
                    return Ok((items, Stop::E));
                }
                '^' | '_' | '\'' => self.scripts(&mut items)?,
                '\\' => {
                    if let Some(p) = self.structural_command(&mut items)? {
                        return Ok((items, p));
                    }
                }
                _ => {
                    let node = self.atom()?;
                    items.extend(node);
                }
            }
        }
    }
}

fn empty(n: &Node) -> bool {
    matches!(n, Node::Row { children } if children.is_empty())
}

fn undo_line(n: Node) -> Vec<Node> {
    match n {
        Node::Row { children } => children,
        other => vec![other],
    }
}

impl Reader<'_> {
    // ---------- atoms ----------

    /// One atom, without scripts. May return several nodes (a number with a thin space, for example).
    fn atom(&mut self) -> R<Vec<Node>> {
        let col = self.column();
        let c = self.current().expect("called with a character");
        Ok(match c {
            '{' => vec![self.group()?],
            '0'..='9' | '.' if c != '.' || self.em(1).is_some_and(|d| d.is_ascii_digit()) => {
                vec![self.number()]
            }
            '~' => {
                self.pos += 1;
                vec![Node::Space { em: 0.333 }]
            }
            '$' => {
                // a stray dollar sign in the middle of the equation: a leftover delimiter
                self.pos += 1;
                vec![]
            }
            c if c.is_control() => {
                self.tolerate(rules::E_REC_9, col, format!("control character U+{:04X}", c as u32), "ignored")?;
                self.pos += 1;
                vec![]
            }
            '#' => {
                self.tolerate(rules::E_REC_9, col, "'#' outside a macro definition", "drawn as a symbol")?;
                self.pos += 1;
                vec![Node::Operator { symbol: '#', class: Class::Ord }]
            }
            c if c.is_alphabetic() => {
                self.pos += 1;
                vec![self.letter(c)]
            }
            c => {
                self.pos += 1;
                match symbols::character(c) {
                    Some(Symbol { symbol, class }) => vec![Node::Operator { symbol, class }],
                    None => vec![Node::Operator { symbol: c, class: Class::Ord }],
                }
            }
        })
    }

    fn letter(&self, c: char) -> Node {
        let variant = if c.is_ascii_alphabetic() || ('α'..='ω').contains(&c) || c == 'ı' || c == 'ȷ' { self.variant } else { Variant::Upright };
        Node::Identifier { name: c.to_string(), variant }
    }

    fn group(&mut self) -> R<Node> {
        let col = self.column();
        self.pos += 1;
        let variant = self.variant;
        let (items, stop) = self.list()?;
        self.variant = variant;
        match stop {
            Stop::CloseBrace => {}
            Stop::Eof => self.tolerate(rules::E_REC_4, col, "'{' without a matching '}'", "closed at the end")?,
            other => {
                self.tolerate(rules::E_REC_4, col, format!("group opened with '{{' ended in {other:?}"), "closed there")?;
            }
        }
        // as in TeX, an operator alone in braces becomes an ordinary atom: {,} gets no space
        Ok(match items.as_slice() {
            [Node::Operator { symbol, .. }] => Node::Operator { symbol: *symbol, class: Class::Ord },
            _ => Node::row(items),
        })
    }

    fn number(&mut self) -> Node {
        if let Some(node) = self.number_with_thousands() {
            return node;
        }
        let start = self.pos;
        let mut value = String::new();
        while let Some(c) = self.current() {
            let decimal = (c == '.' || (c == ',' && self.locale.comma_is_decimal())) && self.em(1).is_some_and(|d| d.is_ascii_digit()) && !value.contains('.');
            if c.is_ascii_digit() {
                value.push(c);
            } else if decimal {
                value.push('.');
            } else {
                break;
            }
            self.pos += 1;
        }
        let written: String = self.c[start..self.pos].iter().collect();
        if written.contains(',') {
            self.registry.note(rules::L_NUM_1, format!("'{written}': comma between digits read as decimal separator ({value})"));
        }
        Node::Number { value, group: false, thousands: None }
    }

    /// L-NUM-3: `1.000` and `12.345,67` in pt-BR, `1,000` and `12,345.67` in English: the dot (or comma)
    /// followed by groups of exactly three digits separates thousands and is kept in the drawing.
    fn number_with_thousands(&mut self) -> Option<Node> {
        let (thousands, decimal) = match self.locale {
            crate::locale::Locale::PtBr => ('.', ','),
            crate::locale::Locale::En => (',', '.'),
        };
        let mut i = self.pos;
        let digits = |i: &mut usize| -> String {
            let mut s = String::new();
            while let Some(c) = self.c.get(*i).filter(|c| c.is_ascii_digit()) {
                s.push(*c);
                *i += 1;
            }
            s
        };
        let first = digits(&mut i);
        if first.is_empty() || first.len() > 3 {
            return None;
        }
        let mut groups = vec![first];
        while self.c.get(i) == Some(&thousands) && self.c.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            let mut j = i + 1;
            let g = digits(&mut j);
            if g.len() != 3 {
                break;
            }
            groups.push(g);
            i = j;
        }
        if groups.len() < 2 {
            return None;
        }
        // after the last group no other digit may follow the thousands separator directly
        if self.c.get(i) == Some(&thousands) && self.c.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            return None;
        }
        let mut value: String = groups.concat();
        if self.c.get(i) == Some(&decimal) && self.c.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            let mut j = i + 1;
            let f = digits(&mut j);
            value.push('.');
            value.push_str(&f);
            i = j;
        }
        let written: String = self.c[self.pos..i].iter().collect();
        self.pos = i;
        self.registry.note(rules::L_NUM_3, format!("'{written}': '{thousands}' separa milhares ({value})"));
        Some(Node::Number { value, group: false, thousands: Some(thousands) })
    }

    /// Argument of a command or script: a group or a single token.
    fn argument(&mut self) -> R<Node> {
        self.skip_spaces();
        let col = self.column();
        match self.current() {
            None => {
                self.tolerate(rules::E_REC_7, col, "missing argument", "left empty")?;
                Ok(Node::row(vec![]))
            }
            Some('{') => Ok(self.group()?.simplify()),
            Some('}') => {
                self.tolerate(rules::E_REC_7, col, "missing argument", "left empty")?;
                Ok(Node::row(vec![]))
            }
            Some('\\') => {
                let mut items = Vec::new();
                if let Some(p) = self.structural_command(&mut items)? {
                    self.tolerate(rules::E_REC_7, col, format!("invalid argument: {p:?}"), "ignored")?;
                }
                Ok(Node::row(items).simplify())
            }
            Some(c) if c.is_ascii_digit() => {
                self.pos += 1;
                Ok(Node::Number { value: c.to_string(), group: false, thousands: None })
            }
            Some(_) => Ok(Node::row(self.atom()?).simplify()),
        }
    }

    /// Base of `\overset` and `\underset`: `{=}` stays a relation (the plain group would make it Ord).
    fn base_with_class(&mut self) -> R<Node> {
        self.skip_spaces();
        if self.current() == Some('{') {
            let raw = self.raw_text()?;
            return self.sub_reader(&raw, self.variant);
        }
        self.argument()
    }

    fn optional_argument(&mut self) -> R<Option<Node>> {
        match self.raw_optional() {
            Some(s) => Ok(Some(self.sub_reader(&s, self.variant)?)),
            None => Ok(None),
        }
    }

    /// `^`, `_` and `'` attach to the last atom.
    fn scripts(&mut self, items: &mut Vec<Node>) -> R<()> {
        self.nested(|this| this.scripts_unguarded(items))
    }

    fn scripts_unguarded(&mut self, items: &mut Vec<Node>) -> R<()> {
        // x_1_2_3…: each double script starts over on an empty base, in a loop (not by recursion)
        'index: loop {
            let previous = items.pop().unwrap_or(Node::row(vec![]));
            // the brace of \overbrace{…}^{label} takes the script as its label
            if let Node::Brace { above, label: None, .. } = &previous {
                let mark = if *above { '^' } else { '_' };
                if self.current() == Some(mark) {
                    self.pos += 1;
                    let label = self.argument()?;
                    if let Node::Brace { base, above, .. } = previous {
                        items.push(Node::Brace { base, above, label: Some(Box::new(label)) });
                    }
                    return Ok(());
                }
            }
            let (base, mut sup, mut sub) = match previous {
                Node::Scripts { base, sup, sub } => (*base, sup.map(|b| *b), sub.map(|b| *b)),
                other => (other, None, None),
            };
            let only_primes = |s: &Option<Node>| matches!(s, Some(Node::Operator { symbol: '′' | '″' | '‴', .. }));
            loop {
                self.skip_spaces();
                match self.current() {
                    Some('\'') => {
                        let mut n = 0;
                        while self.current() == Some('\'') {
                            n += 1;
                            self.pos += 1;
                        }
                        let prime = Node::Operator { symbol: ['′', '″', '‴'][n.min(3) - 1], class: Class::Ord };
                        sup = Some(match sup {
                            None => prime,
                            Some(s) => Node::row(std::iter::once(prime).chain(undo_line(s)).collect()),
                        });
                    }
                    Some('^') if sup.is_none() || only_primes(&sup) => {
                        self.pos += 1;
                        let arg = self.argument()?;
                        sup = Some(match sup {
                            None => arg,
                            Some(prime) => Node::row(vec![prime, arg]),
                        });
                    }
                    Some('_') if sub.is_none() => {
                        self.pos += 1;
                        sub = Some(self.argument()?);
                    }
                    Some(c @ ('^' | '_')) => {
                        let col = self.column();
                        let which = if c == '^' { "superscript" } else { "subscript" };
                        self.tolerate(rules::E_REC_3, col, format!("double {which}: use braces, as in x{c}{{a{c}b}}"), "the second went on an empty base")?;
                        // close the current script and start over with an empty base
                        items.push(Node::Scripts { base: Box::new(base), sup: sup.take().map(Box::new), sub: sub.take().map(Box::new) });
                        items.push(Node::row(vec![]));
                        continue 'index;
                    }
                    _ => break,
                }
            }
            // we only get here after reading at least one script
            items.push(Node::Scripts { base: Box::new(base), sup: sup.map(Box::new), sub: sub.map(Box::new) });
            return Ok(());
        }
    }
}

// ---------- commands ----------

impl Reader<'_> {
    /// Reads a command. Returns a stop when the command ends the list (`\\`, `\end`, `\right`…).
    fn structural_command(&mut self, items: &mut Vec<Node>) -> R<Option<Stop>> {
        self.nested(|this| this.command_unguarded(items))
    }

    fn command_unguarded(&mut self, items: &mut Vec<Node>) -> R<Option<Stop>> {
        let col = self.column();
        let start = self.pos;
        self.pos += 1;
        let name = self.read_command_name();
        if let Some(m) = self.macros.commands.get(&name).cloned() {
            self.expand(start, &m, col)?;
            return Ok(None);
        }
        let primitive = match name.as_str() {
            "bgroup" => Some("{"),
            "egroup" => Some("}"),
            "sp" => Some("^"),
            "sb" => Some("_"),
            "hfill" | "hss" | "quad" if name == "hfill" || name == "hss" => Some("\\quad "),
            "vrule" => Some("\\rule{0.04em}{1em}"),
            _ => None,
        };
        if let Some(text) = primitive {
            self.inject(start, text, col)?;
            return Ok(None);
        }
        if let Some(expansion) = self.package_expansion(&name, col)? {
            self.inject(start, &expansion, col)?;
            return Ok(None);
        }
        if self.definition(&name, col)? {
            return Ok(None);
        }
        let star = if self.current() == Some('*') && matches!(name.as_str(), "operatorname" | "tag" | "smash") {
            self.pos += 1;
            true
        } else {
            false
        };
        // each group of commands lives in its own function: in debug builds, a single match with every
        // arm reserved ~80 KB of stack per level of nested \frac{…}.
        // Each group returns None when the name is not its own; the last one handles any name.
        #[allow(clippy::type_complexity)]
        let groups: [fn(&mut Self, &str, usize, usize, bool, &mut Vec<Node>) -> R<Option<Option<Stop>>>; 11] = [
            Self::structure_command,
            Self::space_command,
            Self::style_command,
            Self::fraction_command,
            Self::text_command,
            Self::accent_command,
            Self::unit_command,
            Self::unicode_and_table_command,
            Self::box_command,
            Self::verbatim_command,
            Self::mod_command,
        ];
        for group in groups {
            if let Some(stop) = group(self, &name, col, start, star, items)? {
                return Ok(stop);
            }
        }
        self.symbol_command(&name, col, start, items)
    }

    fn structure_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            "\\" | "cr" | "newline" => {
                self.raw_optional();
                return Ok(Some(Some(Stop::NewRow)));
            }
            "end" => return Ok(Some(Some(Stop::End(self.raw_text()?.trim().to_string())))),
            "right" => return Ok(Some(Some(Stop::Right(self.delimiter()?)))),
            "middle" => return Ok(Some(Some(Stop::Middle(self.delimiter()?)))),
            "hline" | "midrule" | "toprule" | "bottomrule" => return Ok(Some(Some(Stop::Hline))),
            "cline" => {
                let range = self.raw_text()?;
                let (a, b) = range.split_once('-').unwrap_or((&range, &range));
                let n = |t: &str| t.trim().parse::<usize>().map(|v| v.saturating_sub(1));
                return match (n(a), n(b)) {
                    (Ok(a), Ok(b)) => Ok(Some(Some(Stop::Cline(a, b)))),
                    _ => self.error(col, format!("\\cline{{{range}}}: use the form \\cline{{1-2}}")),
                };
            }
            "left" => self.delimited(col)?,
            "begin" => {
                let saved = self.pos;
                let env_name = self.raw_text()?.trim().to_string();
                if let Some((open, close)) = self.macros.environments.get(&env_name).cloned() {
                    let content = self.environment_body(&env_name, col)?;
                    self.inject(start, &format!("{{{open}{content}{close}}}"), col)?;
                    return Ok(Some(None));
                }
                self.pos = saved;
                self.environment(col)?
            }
            "intertext" | "shortintertext" => return Ok(Some(Some(Stop::Intertext(self.text()?)))),
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn space_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let mu = |m: f32| Node::Space { em: m / 18.0 };
        let node: Node = match name {
            // spaces
            "," | "thinspace" => mu(3.0),
            ":" | ">" | "medspace" => mu(4.0),
            ";" | "thickspace" => mu(5.0),
            "!" | "negthinspace" => mu(-3.0),
            "negmedspace" => mu(-4.0),
            "negthickspace" => mu(-5.0),
            " " => Node::Space { em: 0.333 },
            "quad" => Node::Space { em: 1.0 },
            "qquad" => Node::Space { em: 2.0 },
            "enspace" | "enskip" => Node::Space { em: 0.5 },
            "hspace" | "mspace" | "kern" | "mkern" | "hskip" | "mskip" => {
                if self.current() == Some('*') {
                    self.pos += 1;
                }
                let measure = if self.c.get(self.pos).is_some_and(|c| *c == '{') || name.starts_with('h') && name != "hskip" { self.raw_text()? } else { self.raw_measure() };
                Node::Space { em: measure_em(&measure).ok_or(Error::Syntax { column: col, message: format!("unrecognized measure: {measure}") })? }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn style_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            // style and font for the rest of the group
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" => {
                let level = match name {
                    "displaystyle" => 0,
                    "textstyle" => 1,
                    "scriptstyle" => 2,
                    _ => 3,
                };
                let (remainder, p) = self.list()?;
                items.push(Node::Style { level, base: Box::new(Node::row(remainder)) });
                return Ok(Some(Some(p)));
            }
            "color" => {
                let color = self.color()?;
                let (remainder, p) = self.list()?;
                let base = Node::row(remainder);
                items.push(match color {
                    Some(color) => Node::Colored { color, base: Box::new(base) },
                    None => base,
                });
                return Ok(Some(Some(p)));
            }
            "over" | "choose" | "atop" | "brack" | "brace" => {
                let num = Node::row(std::mem::take(items));
                let (remainder, p) = self.list()?;
                let den = Node::row(remainder);
                let rule = name == "over";
                let fraction = Node::Fraction { num: Box::new(num), den: Box::new(den), style: FractionStyle::Auto, rule };
                items.push(match name {
                    "choose" => Node::Delimited { open: Some('('), close: Some(')'), parts: vec![fraction], middles: vec![] },
                    "brack" => Node::Delimited { open: Some('['), close: Some(']'), parts: vec![fraction], middles: vec![] },
                    "brace" => Node::Delimited { open: Some('{'), close: Some('}'), parts: vec![fraction], middles: vec![] },
                    _ => fraction,
                });
                return Ok(Some(Some(p)));
            }
            "bf" | "rm" | "it" | "sf" | "tt" | "cal" | "mit" | "boldmath" | "unboldmath" | "normalfont" => {
                self.variant = match name {
                    "bf" | "boldmath" => Variant::Bold,
                    "rm" | "normalfont" => Variant::Upright,
                    "sf" => Variant::SansSerif,
                    "tt" => Variant::Monospace,
                    "cal" => Variant::Calligraphic,
                    _ => Variant::Italic,
                };
                return Ok(Some(None));
            }
            "tiny" | "scriptsize" | "footnotesize" | "small" | "normalsize" | "large" | "Large" | "LARGE" | "huge" | "Huge" => {
                self.registry.note(rules::L_ENT_3, format!("\\{name} ignored: the size comes from whoever uses the SVG"));
                return Ok(Some(None));
            }
            "limits" | "nolimits" | "displaylimits" => {
                let wanted = match name {
                    "limits" => Some(true),
                    "nolimits" => Some(false),
                    _ => None,
                };
                if let Some(last_item) = items.last_mut() {
                    set_limits(last_item, wanted);
                }
                return Ok(Some(None));
            }
            "label" | "ref" | "eqref" => {
                let label = self.raw_text()?;
                if name == "label" {
                    return Ok(Some(None));
                }
                Node::Text { parts: vec![TextPart::Text(if name == "eqref" { format!("({label})") } else { label })] }
            }
            "nonumber" | "notag" | "nolinebreak" | "allowbreak" | "displaybreak" | "protect" | "relax" | "noindent" | "centering" | "ensuremath" => return Ok(Some(None)),
            "strut" | "mathstrut" => Node::Phantom { base: Box::new(Node::Operator { symbol: '(', class: Class::Open }), width: false, height: true },
            "tag" => {
                let label = self.raw_text()?;
                let text = if star { label } else { format!("({label})") };
                items.push(Node::Space { em: 2.0 });
                Node::Text { parts: vec![TextPart::Text(text)] }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn fraction_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            // fractions, roots, binomials
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                if name == "cfrac" {
                    self.raw_optional();
                }
                let num = self.argument()?;
                let den = self.argument()?;
                let style = match name {
                    "dfrac" | "cfrac" => FractionStyle::Display,
                    "tfrac" => FractionStyle::Row,
                    _ => FractionStyle::Auto,
                };
                Node::Fraction { num: Box::new(num), den: Box::new(den), style, rule: true }
            }
            "binom" | "dbinom" | "tbinom" => {
                let num = self.argument()?;
                let den = self.argument()?;
                let style = match name {
                    "dbinom" => FractionStyle::Display,
                    "tbinom" => FractionStyle::Row,
                    _ => FractionStyle::Auto,
                };
                let f = Node::Fraction { num: Box::new(num), den: Box::new(den), style, rule: false };
                Node::Delimited { open: Some('('), close: Some(')'), parts: vec![f], middles: vec![] }
            }
            "sqrt" | "root" => {
                let index = self.optional_argument()?;
                let radicand = self.argument()?;
                Node::Root { radicand: Box::new(radicand), index: index.map(Box::new) }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn text_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            // text and fonts
            "text" | "textrm" | "textnormal" | "textup" | "mbox" | "hbox" | "textit" | "textbf" | "textsf" | "texttt" | "emph" | "textmd" | "textsl" | "fbox" | "framebox" => {
                let text = self.text()?;
                if matches!(name, "fbox" | "framebox") {
                    Node::Framed { base: Box::new(text), shape: Frame::Rectangle }
                } else {
                    text
                }
            }
            "mathrm" | "mathup" | "mathit" | "mathbf" | "mathsf" | "mathtt" | "mathbb" | "mathcal" | "mathscr" | "mathfrak" | "boldsymbol" | "bm" | "pmb" | "mathbfit"
            | "mathnormal" | "symbf" | "symrm" | "symit" | "mathds" | "mathbbm" | "Bbb" => {
                let variant = match name {
                    "mathrm" | "mathup" | "symrm" => Variant::Upright,
                    "mathit" | "mathnormal" | "symit" => Variant::Italic,
                    "mathbf" | "symbf" => Variant::Bold,
                    "boldsymbol" | "bm" | "pmb" | "mathbfit" => Variant::BoldItalic,
                    "mathsf" => Variant::SansSerif,
                    "mathtt" => Variant::Monospace,
                    "mathbb" | "mathds" | "mathbbm" | "Bbb" => Variant::DoubleStruck,
                    "mathcal" | "mathscr" => Variant::Calligraphic,
                    _ => Variant::Fraktur,
                };
                let raw = self.raw_text()?;
                self.sub_reader(&raw, variant)?
            }
            "operatorname" => {
                let raw = self.raw_text()?;
                // in math mode TeX ignores spaces: \operatorname{ l i m } is "lim"
                let function_name: String = raw.chars().filter(|c| c.is_alphanumeric() || *c == '-').collect();
                Node::Function { name: function_name.trim().to_string(), limits: star }
            }
            "mathop" | "mathrel" | "mathbin" | "mathord" | "mathopen" | "mathclose" | "mathpunct" | "mathinner" => {
                let class = match name {
                    "mathop" => Class::Op,
                    "mathrel" => Class::Rel,
                    "mathbin" => Class::Bin,
                    "mathopen" => Class::Open,
                    "mathclose" => Class::Close,
                    "mathpunct" => Class::Punct,
                    "mathinner" => Class::Inner,
                    _ => Class::Ord,
                };
                Node::ForcedClass { class, base: Box::new(self.argument()?) }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn accent_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            // accents, bars, braces
            "overline" | "underline" => Node::Bar { base: Box::new(self.argument()?), above: name == "overline" },
            "overbrace" | "underbrace" => Node::Brace { base: Box::new(self.argument()?), above: name == "overbrace", label: None },
            "overset" | "stackrel" => {
                let above = self.argument()?;
                let base = self.base_with_class()?;
                Node::ForcedClass {
                    class: if name == "stackrel" { Class::Rel } else { natural_class(&base) },
                    base: Box::new(Node::Stacked { base: Box::new(base), above: Some(Box::new(above)), below: None }),
                }
            }
            "underset" => {
                let below = self.argument()?;
                let base = self.base_with_class()?;
                let class = natural_class(&base);
                Node::ForcedClass { class, base: Box::new(Node::Stacked { base: Box::new(base), above: None, below: Some(Box::new(below)) }) }
            }
            "xrightarrow" | "xleftarrow" | "xRightarrow" | "xLeftarrow" | "xleftrightarrow" | "xLeftrightarrow" | "xrightleftharpoons" | "xleftrightharpoons" | "xmapsto"
            | "xlongequal" | "xhookrightarrow" => {
                let symbol = match name {
                    "xrightarrow" => '→',
                    "xleftarrow" => '←',
                    "xRightarrow" => '⇒',
                    "xLeftarrow" => '⇐',
                    "xleftrightarrow" => '↔',
                    "xLeftrightarrow" => '⇔',
                    "xrightleftharpoons" => '⇌',
                    "xleftrightharpoons" => '⇋',
                    "xmapsto" => '↦',
                    "xhookrightarrow" => '↪',
                    _ => '=',
                };
                let below = self.optional_argument()?;
                let above = self.argument()?;
                let above = if empty(&above) { None } else { Some(Box::new(above)) };
                Node::Arrow { symbol, above, below: below.map(Box::new) }
            }
            "cancel" | "bcancel" | "xcancel" | "sout" => {
                let kind = match name {
                    "bcancel" => Cancel::Backslash,
                    "xcancel" => Cancel::X,
                    _ => Cancel::Bar,
                };
                Node::Canceled { base: Box::new(self.argument()?), shape: kind }
            }
            "cancelto" => {
                let value = self.argument()?;
                let base = self.argument()?;
                Node::Scripts { base: Box::new(Node::Canceled { base: Box::new(base), shape: Cancel::Bar }), sup: Some(Box::new(value)), sub: None }
            }
            "boxed" => Node::Framed { base: Box::new(self.argument()?), shape: Frame::Rectangle },
            "textcircled" | "circled" | "circledcirc" => Node::Framed { base: Box::new(self.argument()?), shape: Frame::Circle },
            "phantom" | "hphantom" | "vphantom" => Node::Phantom { base: Box::new(self.argument()?), width: name != "vphantom", height: name != "hphantom" },
            "textcolor" | "colorbox" => {
                let color = self.color()?;
                let base = if name == "colorbox" { self.text()? } else { self.argument()? };
                match color {
                    Some(color) => Node::Colored { color, base: Box::new(base) },
                    None => base,
                }
            }
            // \centernot (centernot package) is drawn like \not
            "not" | "centernot" => {
                self.skip_spaces();
                let mut following = Vec::new();
                if self.current() == Some('\\') {
                    if let Some(p) = self.structural_command(&mut following)? {
                        return self.error(col, format!("\\not before {p:?}"));
                    }
                } else if self.current().is_some() {
                    following.extend(self.atom()?);
                }
                match following.pop() {
                    Some(Node::Operator { symbol, class }) => match negate(symbol) {
                        Some(n) => Node::Operator { symbol: n, class },
                        None => Node::Canceled { base: Box::new(Node::Operator { symbol, class }), shape: Cancel::Bar },
                    },
                    Some(other) => Node::Canceled { base: Box::new(other), shape: Cancel::Bar },
                    None => return self.error(col, "\\not with no symbol after it"),
                }
            }
            "big" | "Big" | "bigg" | "Bigg" | "bigl" | "Bigl" | "biggl" | "Biggl" | "bigr" | "Bigr" | "biggr" | "Biggr" | "bigm" | "Bigm" | "biggm" | "Biggm" => {
                let size = match name.trim_end_matches(['l', 'r', 'm']) {
                    "big" => 1,
                    "Big" => 2,
                    "bigg" => 3,
                    _ => 4,
                };
                let Some(symbol) = self.delimiter()? else {
                    return Ok(Some(None));
                };
                let class = match name.chars().last() {
                    Some('l') => Class::Open,
                    Some('r') => Class::Close,
                    Some('m') => Class::Rel,
                    _ => symbols::character(symbol).map_or(Class::Ord, |s| s.class),
                };
                Node::FixedDelimiter { symbol, class, size }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn unit_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            // units, chemistry, arithmetic
            "SI" | "qty" => {
                let value = self.raw_text()?;
                let unit = self.raw_text()?;
                self.quantity(Some(value), &unit, col)?
            }
            "si" | "unit" => {
                let unit = self.raw_text()?;
                self.quantity(None, &unit, col)?
            }
            "num" => {
                let value = self.raw_text()?;
                self.raw_optional();
                siunitx_number(&value, true).ok_or(Error::Syntax { column: col, message: format!("\\num{{{value}}} is not a number") })?
            }
            "ang" => {
                let value = self.raw_text()?;
                let n = siunitx_number(&value, false).ok_or(Error::Syntax { column: col, message: format!("\\ang{{{value}}} is not a number") })?;
                Node::row(vec![n, Node::Operator { symbol: '°', class: Class::Ord }])
            }
            "ce" | "pu" => {
                let raw = self.raw_text()?;
                self.chemistry(&raw, col)?
            }
            "divchave" => {
                let dividend = self.raw_text()?.trim().to_string();
                let divisor = self.raw_text()?.trim().to_string();
                if !is_integer(&dividend) || !is_integer(&divisor) || divisor.trim_start_matches('0').is_empty() {
                    let problem = "\\divchave{dividend}{divisor} takes integers, with a nonzero divisor";
                    self.tolerate(rules::E_REC_9, col, problem, "written as inline division")?;
                    self.sub_reader(&format!("{{{dividend}}} \\div {{{divisor}}}"), self.variant)?
                } else {
                    Node::LongDivision { dividend, divisor }
                }
            }
            "contaarmada" => {
                let op = self.raw_text()?.trim().to_string();
                let operation = match op.as_str() {
                    "+" => Some('+'),
                    "-" => Some('−'),
                    "x" | "\\times" | "×" | "*" => Some('×'),
                    _ => None,
                };
                let a = self.raw_text()?.trim().to_string();
                let b = self.raw_text()?.trim().to_string();
                match operation {
                    Some(operation) if is_integer(&a) && is_integer(&b) => Node::ColumnArithmetic { operation, terms: vec![a, b] },
                    _ => {
                        let problem = if operation.is_none() { "\\contaarmada{op}{a}{b}: op is +, - or \\times" } else { "\\contaarmada takes integers" };
                        self.tolerate(rules::E_REC_9, col, problem, "written as an inline calculation")?;
                        let symbol = operation.map_or(op, String::from);
                        self.sub_reader(&format!("{{{a}}} {symbol} {{{b}}}"), self.variant)?
                    }
                }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn unicode_and_table_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            "unicode" => {
                let raw = self.raw_text()?;
                let code = raw.trim().trim_start_matches(['x', 'X', '#', 'U', '+']);
                let c = u32::from_str_radix(code, 16).ok().and_then(char::from_u32);
                match c {
                    Some(c) => Node::Operator { symbol: c, class: Class::Ord },
                    None => return self.error(col, format!("\\unicode{{{raw}}}: invalid code point")),
                }
            }
            "LaTeX" | "TeX" => Node::Text { parts: vec![TextPart::Text(name.to_string())] },
            "substack" => self.argument_table(TableKind::Substack, col)?,
            "cases" => self.argument_table(TableKind::Cases, col)?,
            "matrix" => self.argument_table(TableKind::Matrix, col)?,
            "pmatrix" => self.argument_table(TableKind::PMatrix, col)?,
            "bordermatrix" => self.argument_table(TableKind::BorderMatrix, col)?,
            "eqalign" | "eqalignno" | "displaylines" => self.argument_table(TableKind::Aligned, col)?,
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn box_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            "genfrac" => {
                let open = self.raw_text()?;
                let close = self.raw_text()?;
                let thickness = self.raw_text()?;
                let style = self.raw_text()?;
                let num = self.argument()?;
                let den = self.argument()?;
                let style = match style.trim() {
                    "0" => FractionStyle::Display,
                    "" => FractionStyle::Auto,
                    _ => FractionStyle::Row,
                };
                let rule = !thickness.trim().trim_start_matches(['0', '.']).trim_end_matches(char::is_alphabetic).is_empty() || thickness.trim().is_empty();
                let f = Node::Fraction { num: Box::new(num), den: Box::new(den), style, rule };
                let delim = |t: &str| -> Option<char> {
                    let t = t.trim();
                    match t.strip_prefix('\\') {
                        Some(n) => symbols::delimiter_command(n).flatten(),
                        None => t.chars().next().and_then(|c| symbols::delimiter_char(c).flatten()),
                    }
                };
                if open.trim().is_empty() && close.trim().is_empty() {
                    f
                } else {
                    Node::Delimited { open: delim(&open), close: delim(&close), parts: vec![f], middles: vec![] }
                }
            }
            "smash" => {
                let option = self.raw_optional().unwrap_or_default();
                let base = self.argument()?;
                Node::Smashed { base: Box::new(base), height: option != "b", depth: option != "t" }
            }
            "mathllap" | "mathrlap" | "mathclap" | "llap" | "rlap" | "clap" => {
                let side = match name.trim_start_matches("math") {
                    "llap" => 'l',
                    "rlap" => 'r',
                    _ => 'c',
                };
                let base = if name.starts_with("math") { self.argument()? } else { self.text()? };
                Node::Overlapped { base: Box::new(base), side }
            }
            "raisebox" => {
                let measure = self.raw_text()?;
                let em = measure_em(&measure).ok_or(Error::Syntax { column: col, message: format!("unrecognized measure: {measure}") })?;
                self.raw_optional();
                self.raw_optional();
                Node::Raised { em, base: Box::new(self.text()?) }
            }
            "scalebox" => {
                let factor: f32 = self.raw_text()?.trim().parse().map_err(|_| Error::Syntax { column: col, message: "\\scalebox{factor}{…} takes a number".into() })?;
                self.raw_optional();
                Node::Scaled { factor: factor.clamp(0.05, 20.0), base: Box::new(self.text()?) }
            }
            "rule" => {
                let raise = self.raw_optional().and_then(|m| measure_em(&m)).unwrap_or(0.0);
                let width = self.raw_text()?;
                let height = self.raw_text()?;
                match (measure_em(&width), measure_em(&height)) {
                    (Some(width), Some(height)) => Node::Rule { width, height, raise },
                    _ => return self.error(col, "\\rule{width}{height}: unrecognized measures"),
                }
            }
            "sideset" => {
                let before = self.raw_text()?;
                let after = self.raw_text()?;
                let pre = self.sub_reader(&format!("{{}}{before}"), self.variant)?;
                let pos = self.sub_reader(&format!("{{}}{after}"), self.variant)?;
                let op = self.argument()?;
                let mut tmp = vec![op];
                self.skip_spaces();
                if matches!(self.current(), Some('^' | '_')) {
                    self.scripts(&mut tmp)?;
                }
                items.push(pre);
                items.extend(tmp);
                pos
            }
            "textsuperscript" | "textsubscript" => {
                let t = self.text()?;
                if name == "textsuperscript" {
                    Node::Scripts { base: Box::new(Node::row(vec![])), sup: Some(Box::new(t)), sub: None }
                } else {
                    Node::Scripts { base: Box::new(Node::row(vec![])), sup: None, sub: Some(Box::new(t)) }
                }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn verbatim_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let node: Node = match name {
            "verb" => {
                if self.current() == Some('*') {
                    self.pos += 1;
                }
                let delim = self.current().ok_or(Error::Syntax { column: col, message: "\\verb without a delimiter".into() })?;
                self.pos += 1;
                let mut children = Vec::new();
                loop {
                    match self.current() {
                        None => return self.error(col, format!("\\verb{delim}… without the closing '{delim}'")),
                        Some(x) if x == delim => {
                            self.pos += 1;
                            break;
                        }
                        Some(x) => {
                            self.pos += 1;
                            children.push(if x.is_ascii_alphanumeric() {
                                Node::Identifier { name: x.to_string(), variant: Variant::Monospace }
                            } else if x == ' ' {
                                Node::Space { em: 0.525 }
                            } else {
                                Node::Operator { symbol: if x == '-' { '-' } else { x }, class: Class::Ord }
                            });
                        }
                    }
                }
                Node::row(children)
            }
            "multicolumn" => {
                let n: usize = self.raw_text()?.trim().parse().map_err(|_| Error::Syntax { column: col, message: "\\multicolumn{n}{…}{…} takes a number".into() })?;
                let spec = self.raw_text()?;
                let alignment = spec.chars().find(|c| matches!(c, 'l' | 'c' | 'r')).unwrap_or('c');
                Node::Multicolumn { columns: n.clamp(1, 50), alignment, base: Box::new(self.argument()?) }
            }
            "hdotsfor" | "dotsfor" => {
                self.raw_optional();
                let n: usize = self.raw_text()?.trim().parse().unwrap_or(1);
                let points = (0..n.clamp(1, 50) * 3).flat_map(|_| [Node::Operator { symbol: '.', class: Class::Ord }, Node::Space { em: 0.35 }]).collect();
                Node::Multicolumn { columns: n.clamp(1, 50), alignment: 'c', base: Box::new(Node::row(points)) }
            }
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    fn mod_command(&mut self, name: &str, col: usize, start: usize, star: bool, items: &mut Vec<Node>) -> R<Option<Option<Stop>>> {
        let _ = (col, start, star);
        let mu = |m: f32| Node::Space { em: m / 18.0 };
        let node: Node = match name {
            // modular arithmetic, with amsmath's spacing
            "pmod" | "mod" | "pod" | "bmod" => {
                let text_mod = || Node::Text { parts: vec![TextPart::Text("mod".into())] };
                if name == "bmod" {
                    Node::ForcedClass { class: Class::Bin, base: Box::new(text_mod()) }
                } else {
                    let arg = self.argument()?;
                    let mut children = vec![mu(if name == "mod" { 12.0 } else { 18.0 })];
                    if name != "mod" {
                        children.push(Node::Operator { symbol: '(', class: Class::Open });
                    }
                    if name != "pod" {
                        children.push(text_mod());
                        children.push(mu(6.0));
                    }
                    children.push(arg);
                    if name != "mod" {
                        children.push(Node::Operator { symbol: ')', class: Class::Close });
                    }
                    Node::row(children)
                }
            }
            "imath" | "jmath" => Node::Identifier { name: if name == "imath" { "ı" } else { "ȷ" }.into(), variant: self.variant },
            "lt" => Node::Operator { symbol: '<', class: Class::Rel },
            "gt" => Node::Operator { symbol: '>', class: Class::Rel },
            _ => return Ok(None),
        };
        items.push(node);
        Ok(Some(None))
    }

    /// Functions, large operators, letters and symbols; whatever is left is an unknown command.
    fn symbol_command(&mut self, other: &str, col: usize, start: usize, items: &mut Vec<Node>) -> R<Option<Stop>> {
        let node = if let Some((canonical, limits)) = symbols::function(other) {
            if canonical != other {
                self.registry.note(rules::L_FUN_1, format!("\\{other} recognized as the function {canonical}"));
            }
            Node::Function { name: canonical.to_string(), limits }
        } else if let Some((symbol, _)) = symbols::large_operator(other) {
            Node::LargeOperator { symbol, limits: None }
        } else if let Some(letter) = symbols::greek(other) {
            let variant = if symbols::greek_upright(other) && self.variant == Variant::Italic { Variant::Upright } else { self.variant };
            Node::Identifier { name: letter.to_string(), variant }
        } else if let Some((accent, stretchy)) = symbols::accent(other) {
            Node::Accent { base: Box::new(self.argument()?), accent, stretchy }
        } else if let Some(Symbol { symbol, class }) = symbols::command(other) {
            Node::Operator { symbol, class }
        } else {
            return self.unknown_command(other, start, col, items);
        };
        items.push(node);
        Ok(None)
    }

    fn raw_measure(&mut self) -> String {
        self.skip_spaces();
        let mut s = String::new();
        while let Some(c) = self.current() {
            if c.is_ascii_digit() || c == '.' || c == '-' || c == ',' || (c.is_ascii_alphabetic() && s.len() < 12) {
                s.push(c);
                self.pos += 1;
                if s.ends_with("em") || s.ends_with("ex") || s.ends_with("pt") || s.ends_with("mu") || s.ends_with("cm") || s.ends_with("mm") || s.ends_with("in") {
                    break;
                }
            } else {
                break;
            }
        }
        s
    }

    /// A valid color, or None in tolerant mode when it is unusable (the content comes out uncolored).
    fn color(&mut self) -> R<Option<String>> {
        let col = self.column();
        let model = self.raw_optional();
        let raw = self.raw_text()?.trim().to_string();
        let color = match model.as_deref() {
            Some("HTML") => format!("#{raw}"),
            Some("rgb") => format!("rgb({raw})"),
            _ => raw,
        };
        let valid = color.chars().all(|c| c.is_ascii_alphanumeric() || "#(),. ".contains(c)) && !color.is_empty() && color.len() < 40;
        if valid {
            return Ok(Some(color));
        }
        self.tolerate(rules::E_REC_9, col, format!("invalid color: {color}"), "drawn without color")?;
        Ok(None)
    }

    fn delimiter(&mut self) -> R<Option<char>> {
        self.skip_spaces();
        let col = self.column();
        match self.current() {
            Some('\\') => {
                self.pos += 1;
                let name = self.read_command_name();
                match symbols::delimiter_command(&name) {
                    Some(d) => Ok(d),
                    None => {
                        self.tolerate(rules::E_REC_9, col, format!("\\{name} is not a delimiter"), "used the empty one")?;
                        Ok(None)
                    }
                }
            }
            Some(c) => {
                self.pos += 1;
                match symbols::delimiter_char(c) {
                    Some(d) => Ok(d),
                    None => {
                        self.tolerate(rules::E_REC_9, col, format!("'{c}' is not a delimiter"), "drawn as is")?;
                        Ok(Some(c))
                    }
                }
            }
            None => {
                self.tolerate(rules::E_REC_7, col, "missing delimiter", "used the empty one")?;
                Ok(None)
            }
        }
    }

    fn delimited(&mut self, col: usize) -> R<Node> {
        let open = self.delimiter()?;
        let variant = self.variant;
        let mut parts = Vec::new();
        let mut middles = Vec::new();
        loop {
            let (items, stop) = self.list()?;
            parts.push(Node::row(items));
            match stop {
                Stop::Right(close) => {
                    self.variant = variant;
                    return Ok(Node::Delimited { open, close, parts, middles });
                }
                Stop::Middle(m) => middles.push(m.unwrap_or('|')),
                Stop::Eof => {
                    self.tolerate(rules::E_REC_5, col, "\\left without \\right", "closed with an empty delimiter")?;
                    self.variant = variant;
                    return Ok(Node::Delimited { open, close: None, parts, middles });
                }
                other => {
                    self.tolerate(rules::E_REC_5, col, format!("\\left opened here ended in {other:?}"), "closed there")?;
                    self.variant = variant;
                    return Ok(Node::Delimited { open, close: None, parts, middles });
                }
            }
        }
    }

    fn environment(&mut self, col: usize) -> R<Node> {
        let name = self.raw_text()?.trim().to_string();
        let base = name.trim_end_matches('*');
        let mut columns: Vec<char> = Vec::new();
        let mut verticals = Vec::new();
        let mut between = Vec::new();
        let kind = match base {
            "matrix" => TableKind::Matrix,
            "pmatrix" => TableKind::PMatrix,
            "bmatrix" => TableKind::BMatrix,
            "Bmatrix" => TableKind::BraceMatrix,
            "vmatrix" => TableKind::VMatrix,
            "Vmatrix" => TableKind::VVMatrix,
            "smallmatrix" => TableKind::SmallMatrix,
            "cases" | "dcases" | "numcases" => TableKind::Cases,
            "aligned" | "align" | "alignat" | "alignedat" | "split" | "eqnarray" | "flalign" | "flalignat" => TableKind::Aligned,
            "gathered" | "gather" | "multline" => TableKind::Centered,
            "array" | "tabular" => TableKind::Array,
            "subarray" => TableKind::Substack,
            "CD" => {
                let raw = self.environment_body("CD", col)?;
                return self.cd_diagram(&raw, col);
            }
            "subequations" => {
                let (items, stop) = self.list()?;
                return match stop {
                    Stop::End(end) if end == name => Ok(Node::row(items)),
                    _ => self.error(col, format!("\\begin{{{name}}} without \\end{{{name}}}")),
                };
            }
            "equation" | "displaymath" | "math" => {
                let (items, stop) = self.list()?;
                return match stop {
                    Stop::End(end) if end == name => Ok(Node::row(items)),
                    _ => self.error(col, format!("\\begin{{{name}}} without \\end{{{name}}}")),
                };
            }
            _ => {
                self.tolerate(rules::E_REC_6, col, format!("unsupported environment: {name}"), "read as a table")?;
                TableKind::Centered
            }
        };
        if matches!(base, "alignat" | "alignedat" | "flalignat") {
            self.raw_text()?;
        }
        if matches!(base, "pmatrix" | "bmatrix" | "matrix" | "vmatrix" | "Bmatrix" | "Vmatrix") && name.ends_with('*') {
            if let Some(a) = self.raw_optional() {
                columns = vec![a.trim().chars().next().unwrap_or('c'); 1];
            }
        }
        if matches!(kind, TableKind::Array | TableKind::Substack) {
            self.raw_optional();
            let spec = self.raw_text()?;
            self.array_columns(&spec, &mut columns, &mut verticals, &mut between, col)?;
        }

        let (rows, horizontals, intertexts) = self.table_body(Some(&name), col)?;
        if !columns.is_empty() && kind != TableKind::Array {
            // pmatrix*[r]: the same alignment in every column
            let a = columns[0];
            let n = rows.iter().map(Vec::len).max().unwrap_or(1);
            columns = vec![a; n];
        }
        Ok(Node::Table { shape: kind, rows, columns, verticals, horizontals, between, intertexts })
    }

    /// Rows and cells up to `\end{name}` (or up to the end of the text, when `name` is None).
    #[allow(clippy::type_complexity)]
    fn table_body(&mut self, name: Option<&str>, col: usize) -> R<(Vec<Vec<Node>>, Vec<TableRule>, Vec<usize>)> {
        let label = name.unwrap_or("tabela");
        let mut rows: Vec<Vec<Node>> = Vec::new();
        let mut row: Vec<Node> = Vec::new();
        let mut cell: Vec<Node> = Vec::new();
        let mut horizontals = Vec::new();
        let mut intertexts = Vec::new();
        loop {
            let (items, stop) = self.list()?;
            cell.extend(items);
            let body_end = match (&stop, name) {
                (Stop::End(end), Some(n)) if end == n => true,
                (Stop::Eof, None) => true,
                _ => false,
            };
            if body_end {
                if !cell.is_empty() || !row.is_empty() {
                    row.push(Node::row(std::mem::take(&mut cell)).simplify());
                    rows.push(row);
                }
                break;
            }
            match stop {
                Stop::E => row.push(Node::row(std::mem::take(&mut cell)).simplify()),
                Stop::NewRow => {
                    row.push(Node::row(std::mem::take(&mut cell)).simplify());
                    rows.push(std::mem::take(&mut row));
                }
                Stop::Hline => horizontals.push(TableRule { row: rows.len(), from: 0, to: usize::MAX }),
                Stop::Cline(from, to) => horizontals.push(TableRule { row: rows.len(), from, to }),
                Stop::Intertext(t) => {
                    if !cell.is_empty() || !row.is_empty() {
                        row.push(Node::row(std::mem::take(&mut cell)).simplify());
                        rows.push(std::mem::take(&mut row));
                    }
                    intertexts.push(rows.len());
                    rows.push(vec![t]);
                }
                Stop::End(other) => {
                    self.tolerate(rules::E_REC_5, col, format!("\\begin{{{label}}} closed with \\end{{{other}}}"), "closed there")?;
                    if !cell.is_empty() || !row.is_empty() {
                        row.push(Node::row(std::mem::take(&mut cell)).simplify());
                        rows.push(std::mem::take(&mut row));
                    }
                    break;
                }
                Stop::Eof => {
                    self.tolerate(rules::E_REC_5, col, format!("\\begin{{{label}}} without \\end{{{label}}}"), "closed at the end")?;
                    if !cell.is_empty() || !row.is_empty() {
                        row.push(Node::row(std::mem::take(&mut cell)).simplify());
                        rows.push(std::mem::take(&mut row));
                    }
                    break;
                }
                Stop::CloseBrace => self.tolerate(rules::E_REC_4, col, format!("stray '}}' inside {label}"), "ignored")?,
                other => self.tolerate(rules::E_REC_5, col, format!("{other:?} inside {label}"), "ignored")?,
            }
        }
        Ok((rows, horizontals, intertexts))
    }

    /// Table read from a braced argument (`\substack{…}`, plain TeX's `\cases{…}`).
    fn argument_table(&mut self, shape: TableKind, col: usize) -> R<Node> {
        let raw = self.raw_text()?;
        let mut sub = Reader { c: raw.chars().collect(), pos: 0, locale: self.locale, registry: self.registry, macros: self.macros, variant: self.variant, strict: self.strict };
        let (rows, horizontals, intertexts) = sub.table_body(None, col)?;
        Ok(Node::Table { shape, rows, columns: vec![], verticals: vec![], horizontals, between: vec![], intertexts })
    }

    /// amscd commutative diagram: `A @>f>> B \\ @VgVV @VVhV \\ C @>>k> D`.
    fn cd_diagram(&mut self, raw: &str, col: usize) -> R<Node> {
        let c: Vec<char> = raw.chars().collect();
        // split into rows at \\ outside braces
        let mut raw_rows: Vec<String> = vec![String::new()];
        let mut level = 0;
        let mut i = 0;
        while i < c.len() {
            match c[i] {
                '{' => level += 1,
                '}' => level -= 1,
                '\\' if level == 0 && c.get(i + 1) == Some(&'\\') => {
                    raw_rows.push(String::new());
                    i += 2;
                    continue;
                }
                _ => {}
            }
            raw_rows.last_mut().unwrap().push(c[i]);
            i += 1;
        }
        let to = |c: &[char], i: &mut usize, end: char| -> String {
            let mut level = 0;
            let mut s = String::new();
            while *i < c.len() {
                let ch = c[*i];
                *i += 1;
                match ch {
                    '{' => level += 1,
                    '}' => level -= 1,
                    x if x == end && level == 0 => return s,
                    _ => {}
                }
                s.push(ch);
            }
            s
        };
        let mut rows = Vec::new();
        for row in raw_rows.iter().filter(|l| !l.trim().is_empty()) {
            let c: Vec<char> = row.chars().collect();
            let mut objects: Vec<String> = vec![String::new()];
            let mut arrows: Vec<Node> = Vec::new();
            let mut vertical = false;
            let mut i = 0;
            while i < c.len() {
                if c[i] == '@' && i + 1 < c.len() {
                    let kind = c[i + 1];
                    i += 2;
                    let label = |s: String, this: &mut Self| -> R<Option<Box<Node>>> {
                        if s.trim().is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some(Box::new(this.sub_reader(&s, Variant::Italic)?)))
                        }
                    };
                    let arrow = match kind {
                        '>' | '<' => {
                            let a = to(&c, &mut i, kind);
                            let b = to(&c, &mut i, kind);
                            Node::Arrow { symbol: if kind == '>' { '→' } else { '←' }, above: label(a, self)?, below: label(b, self)? }
                        }
                        'V' | 'A' => {
                            vertical = true;
                            let a = to(&c, &mut i, kind);
                            let b = to(&c, &mut i, kind);
                            Node::VerticalArrow { symbol: if kind == 'V' { '↓' } else { '↑' }, left: label(a, self)?, right: label(b, self)? }
                        }
                        '=' => Node::Arrow { symbol: '=', above: None, below: None },
                        '|' => {
                            vertical = true;
                            Node::VerticalArrow { symbol: '‖', left: None, right: None }
                        }
                        '.' => Node::row(vec![]),
                        other => return self.error(col, format!("unknown diagram arrow: @{other}")),
                    };
                    arrows.push(arrow);
                    objects.push(String::new());
                } else {
                    objects.last_mut().unwrap().push(c[i]);
                    i += 1;
                }
            }
            let mut cells = Vec::new();
            if vertical {
                // row of vertical arrows: one per object column, with empty columns between them
                for (k, s) in arrows.into_iter().enumerate() {
                    if k > 0 {
                        cells.push(Node::row(vec![]));
                    }
                    cells.push(s);
                }
            } else {
                let mut arrows = arrows.into_iter();
                for (k, o) in objects.iter().enumerate() {
                    if k > 0 {
                        cells.push(arrows.next().unwrap_or(Node::row(vec![])));
                    }
                    cells.push(self.sub_reader(o, Variant::Italic)?);
                }
            }
            rows.push(cells);
        }
        Ok(Node::Table { shape: TableKind::Diagram, rows, columns: vec![], verticals: vec![], horizontals: vec![], between: vec![], intertexts: vec![] })
    }

    fn array_columns(&mut self, spec: &str, columns: &mut Vec<char>, verticals: &mut Vec<usize>, between: &mut Vec<(usize, Node)>, col: usize) -> R<()> {
        let c: Vec<char> = spec.chars().collect();
        let mut i = 0;
        let group = |i: &mut usize| -> String {
            // content of {…} starting at c[*i] == '{'
            let mut level = 0;
            let mut s = String::new();
            while *i < c.len() {
                let ch = c[*i];
                *i += 1;
                match ch {
                    '{' => {
                        level += 1;
                        if level == 1 {
                            continue;
                        }
                    }
                    '}' => {
                        level -= 1;
                        if level == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                s.push(ch);
            }
            s
        };
        while i < c.len() {
            match c[i] {
                'l' | 'c' | 'r' => {
                    columns.push(c[i]);
                    i += 1;
                }
                'p' | 'm' | 'b' | 'X' => {
                    i += 1;
                    if c.get(i) == Some(&'{') {
                        group(&mut i);
                    }
                    columns.push('l');
                }
                '|' => {
                    verticals.push(columns.len());
                    i += 1;
                }
                '@' | '!' => {
                    i += 1;
                    let content = group(&mut i);
                    between.push((columns.len(), self.sub_reader(&content, Variant::Italic)?));
                }
                '*' => {
                    i += 1;
                    let n: usize = group(&mut i).trim().parse().unwrap_or(1);
                    let repeat_n_times = group(&mut i);
                    // *{50}{*{50}{…}} multiplies: the total column count is capped too
                    for _ in 0..n.min(50) {
                        if columns.len() >= MAX_COLUMNS {
                            break;
                        }
                        self.nested(|this| this.array_columns(&repeat_n_times, columns, verticals, between, col))?;
                    }
                }
                ' ' => i += 1,
                other => {
                    self.tolerate(rules::E_REC_9, col, format!("unsupported array column: '{other}'"), "ignored")?;
                    i += 1;
                }
            }
        }
        Ok(())
    }

    /// Content of `\text{…}`: running text, old-style accents and `$…$` snippets.
    fn text(&mut self) -> R<Node> {
        let col = self.column();
        let raw = self.raw_text()?;
        let c: Vec<char> = raw.chars().collect();
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut i = 0;
        // bold: \textbf{…} closes at the group's brace; \bf and \bfseries last until the end of the current group
        let mut level = 0usize;
        let mut bold_until: Vec<usize> = Vec::new();
        while i < c.len() {
            match c[i] {
                '$' => {
                    let end = c[i + 1..].iter().position(|&x| x == '$').map(|p| p + i + 1);
                    let Some(end) = end else {
                        self.tolerate(rules::E_REC_9, col, "unpaired '$' inside \\text", "written as a dollar sign")?;
                        current.push('$');
                        i += 1;
                        continue;
                    };
                    let snippet: String = c[i + 1..end].iter().collect();
                    if !current.is_empty() {
                        parts.push(TextPart::Text(std::mem::take(&mut current)));
                    }
                    parts.push(TextPart::Math(self.sub_reader(&snippet, Variant::Italic)?));
                    i = end + 1;
                }
                '~' => {
                    current.push('\u{00A0}');
                    i += 1;
                }
                '{' => {
                    level += 1;
                    i += 1;
                }
                '}' => {
                    if bold_until.last() == Some(&level) {
                        bold_until.pop();
                        current.push(BOLD_END);
                    }
                    level = level.saturating_sub(1);
                    i += 1;
                }
                '\\' if matches!(c.get(i + 1), Some('(') | Some('[')) => {
                    // \(…\) and \[…\] inside text, like $…$
                    let close = if c[i + 1] == '(' { ')' } else { ']' };
                    let end = (i + 2..c.len().saturating_sub(1)).find(|&k| c[k] == '\\' && c[k + 1] == close);
                    let Some(end) = end else {
                        self.tolerate(rules::E_REC_9, col, format!("unpaired \\{} inside \\text", c[i + 1]), "written as is")?;
                        current.push(c[i + 1]);
                        i += 2;
                        continue;
                    };
                    let snippet: String = c[i + 2..end].iter().collect();
                    if !current.is_empty() {
                        parts.push(TextPart::Text(std::mem::take(&mut current)));
                    }
                    parts.push(TextPart::Math(self.sub_reader(&snippet, Variant::Italic)?));
                    i = end + 2;
                }
                '\\' => {
                    i += 1;
                    let mark = c.get(i).copied().unwrap_or(' ');
                    if "'`^~\"=.".contains(mark) || (matches!(mark, 'c' | 'u' | 'v') && c.get(i + 1).is_some_and(|x| *x == '{' || *x == ' ')) {
                        i += 1;
                        while c.get(i) == Some(&' ') || c.get(i) == Some(&'{') {
                            i += 1;
                        }
                        let letter = c.get(i).copied().unwrap_or(' ');
                        i += 1;
                        if c.get(i) == Some(&'}') {
                            i += 1;
                        }
                        match symbols::text_accent(mark, letter) {
                            Some(composed) => current.push(composed),
                            // no precomposed form: the letter followed by the combining accent
                            None => current.extend([letter].into_iter().chain(symbols::text_combining(mark))),
                        }
                        continue;
                    }
                    let mut name = String::new();
                    while let Some(x) = c.get(i).filter(|x| x.is_ascii_alphabetic()) {
                        name.push(*x);
                        i += 1;
                    }
                    if name.is_empty() {
                        i += 1;
                        match mark {
                            '%' | '$' | '&' | '#' | '_' | '{' | '}' => current.push(mark),
                            '|' => current.push('‖'),
                            ',' | ';' | ':' | ' ' | '!' => current.push(if mark == '!' { '\u{200B}' } else { ' ' }),
                            '\\' => current.push(' '),
                            other => {
                                self.tolerate(rules::E_REC_1, col, format!("unsupported text command: \\{other}"), "written as is")?;
                                current.push(other);
                            }
                        }
                        continue;
                    }
                    while c.get(i) == Some(&' ') {
                        i += 1;
                    }
                    match name.as_str() {
                        "textbf" | "mathbf" => {
                            current.push(BOLD_START);
                            bold_until.push(level + 1);
                        }
                        "bf" | "bfseries" => {
                            current.push(BOLD_START);
                            bold_until.push(level);
                        }
                        "textit" | "emph" | "textrm" | "textsf" | "texttt" | "text" | "mbox" | "textup" | "textnormal" | "it" | "rm" | "sf" | "tt" | "em" | "small"
                        | "scriptsize" | "footnotesize" | "tiny" | "large" | "normalsize" | "mdseries" => {}
                        "quad" => current.push('\u{2003}'),
                        "qquad" => current.push_str("\u{2003}\u{2003}"),
                        "ldots" | "dots" => current.push('…'),
                        "textdegree" | "degree" => current.push('°'),
                        "o" => current.push('ø'),
                        "ss" => current.push('ß'),
                        "aa" => current.push('å'),
                        "AA" => current.push('Å'),
                        "ae" => current.push('æ'),
                        "textendash" => current.push('–'),
                        "textemdash" => current.push('—'),
                        "textasciitilde" => current.push('~'),
                        "textbackslash" => current.push('\\'),
                        "times" => current.push('×'),
                        "LaTeX" => current.push_str("LaTeX"),
                        "TeX" => current.push_str("TeX"),
                        other => {
                            if let Some(symbols::Symbol { symbol, .. }) = symbols::command(other) {
                                current.push(symbol);
                            } else if let Some(g) = symbols::greek(other) {
                                current.push(g);
                            } else {
                                self.tolerate(rules::E_REC_1, col, format!("unsupported text command: \\{other}"), "written as is")?;
                                current.push('\\');
                                current.push_str(other);
                            }
                        }
                    }
                }
                other => {
                    current.push(other);
                    i += 1;
                }
            }
        }
        for _ in bold_until {
            current.push(BOLD_END);
        }
        if !current.is_empty() {
            parts.push(TextPart::Text(current));
        }
        Ok(Node::Text { parts })
    }

    /// L-UNI-1: `\SI{9,8}{\meter\per\second\squared}` → 9,8 m/s² in pt-BR.
    fn quantity(&mut self, value: Option<String>, unit: &str, col: usize) -> R<Node> {
        let unit_node = if unit.contains('\\') {
            let mut numerator: Vec<(String, i32)> = Vec::new();
            let mut denominator: Vec<(String, i32)> = Vec::new();
            let mut prefix = String::new();
            let mut per = false;
            let mut last_in_den = false;
            for token in unit.split('\\').map(str::trim).filter(|t| !t.is_empty()) {
                let (name, remainder) = token.split_at(token.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(token.len()));
                match name {
                    "per" => per = true,
                    "squared" | "cubed" => {
                        let target = if last_in_den { denominator.last_mut() } else { numerator.last_mut() };
                        if let Some(u) = target {
                            u.1 *= if name == "squared" { 2 } else { 3 };
                        }
                    }
                    "square" | "cubic" => prefix.push(if name == "square" { '²' } else { '³' }),
                    "tothe" | "raiseto" => {
                        let n: i32 = remainder.trim_matches(['{', '}', ' ']).parse().unwrap_or(1);
                        let target = if last_in_den { denominator.last_mut() } else { numerator.last_mut() };
                        if let Some(u) = target {
                            u.1 *= n;
                        }
                    }
                    _ => {
                        if let Some(p) = symbols::prefix(name) {
                            // \square\kilo\meter: the pending exponent still applies (km²)
                            prefix.insert_str(0, p);
                            continue;
                        }
                        let Some(symbol) = symbols::unit(name) else {
                            return self.error(col, format!("unsupported unit: \\{name}"));
                        };
                        let exponent = if prefix.contains('²') {
                            2
                        } else if prefix.contains('³') {
                            3
                        } else {
                            1
                        };
                        let text = format!("{}{symbol}", prefix.trim_end_matches(['²', '³']));
                        prefix.clear();
                        if per {
                            denominator.push((text, exponent));
                            last_in_den = true;
                            per = false;
                        } else {
                            numerator.push((text, exponent));
                            last_in_den = false;
                        }
                    }
                }
            }
            let mut children = Vec::new();
            let unit_node = |u: &(String, i32)| -> Node {
                let base = Node::Text { parts: vec![TextPart::Text(u.0.clone())] };
                if u.1 == 1 {
                    base
                } else {
                    Node::Scripts { base: Box::new(base), sup: Some(Box::new(Node::Number { value: u.1.to_string(), group: false, thousands: None })), sub: None }
                }
            };
            for (i, u) in numerator.iter().enumerate() {
                if i > 0 {
                    children.push(Node::Space { em: 3.0 / 18.0 });
                }
                children.push(unit_node(u));
            }
            if !denominator.is_empty() {
                if numerator.is_empty() {
                    children.push(Node::Number { value: "1".into(), group: false, thousands: None });
                }
                children.push(Node::Operator { symbol: '/', class: Class::Ord });
                let several = denominator.len() > 1;
                if several {
                    children.push(Node::Operator { symbol: '(', class: Class::Open });
                }
                for (i, u) in denominator.iter().enumerate() {
                    if i > 0 {
                        children.push(Node::Operator { symbol: '·', class: Class::Ord });
                    }
                    children.push(unit_node(u));
                }
                if several {
                    children.push(Node::Operator { symbol: ')', class: Class::Close });
                }
            }
            self.registry.note(rules::L_UNI_1, format!("unit {unit} written with a slash for negative exponents"));
            Node::row(children)
        } else {
            self.sub_reader(unit, Variant::Upright)?
        };
        let value = match value {
            Some(v) => Some(siunitx_number(&v, true).ok_or(Error::Syntax { column: col, message: format!("value {v} is not a number") })?),
            None => None,
        };
        let text_value = value.as_ref().map(|v| match v {
            Node::Number { value, .. } => value.clone(),
            _ => String::new(),
        });
        let mut children = Vec::new();
        if let Some(v) = value {
            children.push(v);
            children.push(Node::Space { em: 3.0 / 18.0 });
        }
        children.push(unit_node);
        Ok(Node::Quantity { value: text_value.filter(|v| !v.is_empty()), unit: Box::new(Node::row(children)) })
    }

    /// mhchem subset: formulas with automatic subscripts, charges, coefficients and arrows.
    fn chemistry(&mut self, raw: &str, col: usize) -> R<Node> {
        let mut children: Vec<Node> = Vec::new();
        for (n, token) in raw.split_whitespace().enumerate() {
            if n > 0 {
                children.push(Node::Space { em: 0.0 });
            }
            match token {
                "->" | "→" => children.push(Node::Operator { symbol: '→', class: Class::Rel }),
                "<-" => children.push(Node::Operator { symbol: '←', class: Class::Rel }),
                "<=>" | "<->" | "⇌" => children.push(Node::Operator { symbol: '⇌', class: Class::Rel }),
                "+" => children.push(Node::Operator { symbol: '+', class: Class::Bin }),
                "=" => children.push(Node::Operator { symbol: '=', class: Class::Rel }),
                "^" | "v" => children.push(Node::Operator { symbol: if token == "^" { '↑' } else { '↓' }, class: Class::Ord }),
                formula => children.push(self.chemical_formula(formula, col)?),
            }
        }
        Ok(Node::row(children))
    }

    fn chemical_formula(&mut self, f: &str, col: usize) -> R<Node> {
        let c: Vec<char> = f.chars().collect();
        let mut children: Vec<Node> = Vec::new();
        let mut i = 0;
        // stoichiometric coefficient
        let mut coef = String::new();
        while i < c.len() && c[i].is_ascii_digit() {
            coef.push(c[i]);
            i += 1;
        }
        if !coef.is_empty() {
            children.push(Node::Number { value: coef, group: false, thousands: None });
            children.push(Node::Space { em: 2.0 / 18.0 });
        }
        let upright = |t: String| Node::Text { parts: vec![TextPart::Text(t)] };
        while i < c.len() {
            let ch = c[i];
            if ch.is_ascii_uppercase() {
                let mut el = ch.to_string();
                i += 1;
                while i < c.len() && c[i].is_ascii_lowercase() {
                    el.push(c[i]);
                    i += 1;
                }
                children.push(upright(el));
            } else if ch.is_ascii_digit() {
                let mut d = String::new();
                while i < c.len() && c[i].is_ascii_digit() {
                    d.push(c[i]);
                    i += 1;
                }
                let base = children.pop().unwrap_or(Node::row(vec![]));
                children.push(Node::Scripts { base: Box::new(base), sup: None, sub: Some(Box::new(Node::Number { value: d, group: false, thousands: None })) });
            } else if ch == '^' {
                i += 1;
                let mut payload = String::new();
                if c.get(i) == Some(&'{') {
                    i += 1;
                    while i < c.len() && c[i] != '}' {
                        payload.push(c[i]);
                        i += 1;
                    }
                    i += 1;
                } else {
                    while i < c.len() && (c[i].is_ascii_digit() || c[i] == '+' || c[i] == '-') {
                        payload.push(c[i]);
                        i += 1;
                    }
                }
                append_payload(&mut children, &payload);
            } else if (ch == '+' || ch == '-') && i == c.len() - 1 && !children.is_empty() {
                append_payload(&mut children, &ch.to_string());
                i += 1;
            } else if let Some(state) = ["(aq)", "(s)", "(l)", "(g)"].into_iter().find(|e| c[i..].iter().take(e.len()).copied().eq(e.chars())) {
                children.push(upright(state.to_string()));
                i += state.len();
            } else {
                let class = match ch {
                    '(' | '[' => Class::Open,
                    ')' | ']' => Class::Close,
                    _ => Class::Ord,
                };
                let symbol = match ch {
                    '-' => '−',
                    '.' | '*' => '·',
                    other => other,
                };
                if ch.is_alphabetic() || ch.is_ascii_punctuation() || ch == '·' {
                    children.push(Node::Operator { symbol, class });
                } else {
                    return self.error(col, format!("unsupported character in \\ce: '{ch}'"));
                }
                i += 1;
            }
        }
        Ok(Node::row(children))
    }
}

fn append_payload(children: &mut Vec<Node>, payload: &str) {
    let text = payload.replace('-', "−");
    let sup = Node::Text { parts: vec![TextPart::Text(text)] };
    let last_item = children.pop().unwrap_or(Node::row(vec![]));
    children.push(match last_item {
        Node::Scripts { base, sub, sup: None } => Node::Scripts { base, sub, sup: Some(Box::new(sup)) },
        other => Node::Scripts { base: Box::new(other), sup: Some(Box::new(sup)), sub: None },
    });
}

fn set_limits(node: &mut Node, wanted: Option<bool>) {
    match node {
        Node::LargeOperator { limits, .. } => *limits = wanted,
        Node::Function { limits, .. } => *limits = wanted.unwrap_or(*limits),
        Node::Scripts { base, .. } => set_limits(base, wanted),
        Node::ForcedClass { base, .. } => set_limits(base, wanted),
        _ => {}
    }
}

fn natural_class(node: &Node) -> Class {
    match node {
        Node::Operator { class, .. } => *class,
        Node::LargeOperator { .. } | Node::Function { .. } => Class::Op,
        _ => Class::Ord,
    }
}

fn negate(c: char) -> Option<char> {
    Some(match c {
        '=' => '≠',
        '∈' => '∉',
        '⊂' => '⊄',
        '⊃' => '⊅',
        '⊆' => '⊈',
        '⊇' => '⊉',
        '≡' => '≢',
        '<' => '≮',
        '>' => '≯',
        '≤' => '≰',
        '≥' => '≱',
        '∼' => '≁',
        '≈' => '≉',
        '∃' => '∄',
        '∣' => '∤',
        '∥' => '∦',
        '→' => '↛',
        _ => return None,
    })
}

fn is_integer(s: &str) -> bool {
    !s.is_empty() && s.len() <= 30 && s.chars().all(|c| c.is_ascii_digit())
}

/// Converts TeX measures to em (1em = 10pt in the 10pt font).
fn measure_em(m: &str) -> Option<f32> {
    let m = m.trim().replace(',', ".");
    let units = [("em", 1.0), ("ex", 0.45), ("pt", 0.1), ("mu", 1.0 / 18.0), ("cm", 2.845), ("mm", 0.2845), ("in", 7.227), ("px", 0.075)];
    for (u, f) in units {
        if let Some(v) = m.strip_suffix(u) {
            return v.trim().parse::<f32>().ok().map(|v| v * f);
        }
    }
    None
}

/// A number in siunitx format: decimal comma or dot, spaces, exponent with `e`.
fn siunitx_number(raw: &str, group: bool) -> Option<Node> {
    let clean: String = raw.replace("\\,", "").replace(['{', '}', ' ', '~'], "");
    let (mantissa, exponent) = match clean.split_once(['e', 'E']) {
        Some((m, e)) => (m.to_string(), Some(e.to_string())),
        None => (clean, None),
    };
    let (sign, mantissa) = match mantissa.strip_prefix(['-', '+']) {
        Some(remainder) => (mantissa.chars().next(), remainder.to_string()),
        None => (None, mantissa),
    };
    let value = mantissa.replace(',', ".");
    if value.is_empty() || value.matches('.').count() > 1 || !value.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let mut children = Vec::new();
    if let Some(s) = sign {
        children.push(Node::Operator { symbol: if s == '-' { '−' } else { '+' }, class: Class::Ord });
    }
    children.push(Node::Number { value, group, thousands: None });
    if let Some(e) = exponent {
        let (sign_e, digits) = match e.strip_prefix(['-', '+']) {
            Some(d) => (e.starts_with('-'), d.to_string()),
            None => (false, e.clone()),
        };
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let mut exp = Vec::new();
        if sign_e {
            exp.push(Node::Operator { symbol: '−', class: Class::Ord });
        }
        exp.push(Node::Number { value: digits, group: false, thousands: None });
        children.push(Node::Operator { symbol: '×', class: Class::Bin });
        children.push(Node::Scripts {
            base: Box::new(Node::Number { value: "10".into(), group: false, thousands: None }),
            sup: Some(Box::new(Node::row(exp).simplify())),
            sub: None,
        });
    }
    Some(Node::row(children).simplify())
}

// ---------- macros ----------

impl Reader<'_> {
    /// Replaces `self.c[start..self.pos]` with `text` and resumes reading at `start`.
    /// In tolerant mode, an expansion beyond the limit disappears from the text with a warning.
    fn inject(&mut self, start: usize, text: &str, col: usize) -> R<()> {
        let replacement: Vec<char> = text.chars().collect();
        let end = self.pos;
        // what was already read is not read again: a text that fits in the replaced span goes to its
        // end, without moving the rest of the input (\bgroup repeated a thousand times does not go quadratic)
        if replacement.len() <= end - start {
            let start = end - replacement.len();
            self.c[start..end].copy_from_slice(&replacement);
            self.pos = start;
            return Ok(());
        }
        // growing moves the rest of the input: expansions that grow count against the limit too
        self.macros.expansions += 1;
        let excess = self.macros.expansions > EXPANSION_LIMIT || self.c.len() + replacement.len() > self.macros.max_size;
        let replacement = if excess {
            if !self.macros.warned_excess {
                self.tolerate(rules::E_REC_1, col, "too many macros: a macro probably calls itself", "expansions beyond the limit were discarded")?;
                self.macros.warned_excess = true;
            }
            Vec::new()
        } else {
            replacement
        };
        self.c.splice(start..end, replacement);
        self.pos = start;
        Ok(())
    }

    fn expand(&mut self, start: usize, m: &Macro, col: usize) -> R<()> {
        self.macros.expansions += 1;
        if let Some((open, close)) = &m.paired {
            // \abs{x} → \mathopen{|}x\mathclose{|};  \abs*{x} → \left|x\right|
            let star = self.current() == Some('*');
            if star {
                self.pos += 1;
            }
            let size = self.raw_optional();
            let x = self.raw_text()?;
            let text = match (star, size) {
                (true, _) => format!("\\left{open} {x} \\right{close}"),
                (false, Some(t)) => format!("{t}l{open} {x} {t}r{close}"),
                (false, None) => format!("\\mathopen{{{open}}} {x} \\mathclose{{{close}}}"),
            };
            return self.inject(start, &text, col);
        }
        let mut args = Vec::new();
        if let Some(default_arg) = &m.default_arg {
            args.push(self.raw_optional().unwrap_or_else(|| default_arg.clone()));
        }
        while args.len() < m.arguments {
            args.push(self.raw_text()?);
        }
        let mut body = String::new();
        let c: Vec<char> = m.body.chars().collect();
        let mut i = 0;
        while i < c.len() {
            if c[i] == '#' && i + 1 < c.len() {
                if let Some(d) = c[i + 1].to_digit(10) {
                    // #0 does not exist in TeX: it disappears, like a missing argument
                    if let Some(a) = (d as usize).checked_sub(1).and_then(|k| args.get(k)) {
                        body.push('{');
                        body.push_str(a);
                        body.push('}');
                    }
                    i += 2;
                    continue;
                }
            }
            body.push(c[i]);
            i += 1;
        }
        // a space keeps the macro from sticking to what follows (\foo x)
        body.push(' ');
        self.inject(start, &body, col)
    }

    /// Name being defined: `{\foo}` or `\foo`.
    fn defined_name(&mut self, col: usize) -> R<String> {
        self.skip_spaces();
        let raw = if self.current() == Some('{') {
            self.raw_text()?
        } else if self.current() == Some('\\') {
            self.pos += 1;
            format!("\\{}", self.read_command_name())
        } else {
            return self.error(col, "missing macro name");
        };
        let name = raw.trim().trim_start_matches('\\').to_string();
        if name.is_empty() {
            return self.error(col, "empty macro name");
        }
        Ok(name)
    }

    /// Commands that define macros. Returns `true` if it handled the command.
    fn definition(&mut self, name: &str, col: usize) -> R<bool> {
        match name {
            "newcommand" | "renewcommand" | "providecommand" | "DeclareRobustCommand" => {
                if self.current() == Some('*') {
                    self.pos += 1;
                }
                let target = self.defined_name(col)?;
                let arguments = match self.raw_optional() {
                    Some(n) => n.trim().parse::<usize>().map_err(|_| Error::Syntax { column: col, message: format!("invalid number of arguments: {n}") })?,
                    None => 0,
                };
                let default_arg = self.raw_optional();
                let body = self.raw_text()?;
                if name == "providecommand" && self.macros.commands.contains_key(&target) {
                    return Ok(true);
                }
                self.registry.note(rules::L_MAC_1, format!("macro \\{target} defined with {arguments} argument(s)"));
                self.macros.commands.insert(target, Macro { arguments, default_arg, body, paired: None });
                Ok(true)
            }
            "def" | "gdef" | "edef" | "xdef" => {
                let target = self.defined_name(col)?;
                let mut arguments = 0;
                while self.current() != Some('{') {
                    match (self.current(), self.em(1)) {
                        (Some('#'), Some(d)) if d.is_ascii_digit() => {
                            arguments += 1;
                            self.pos += 2;
                        }
                        (Some(c), _) if c.is_whitespace() => self.pos += 1,
                        (None, _) => return self.error(col, format!("\\def\\{target} without a body")),
                        _ => return self.error(col, "\\def with delimited parameters is not supported (only #1#2…)"),
                    }
                }
                let body = self.raw_text()?;
                self.registry.note(rules::L_MAC_1, format!("macro \\{target} defined with {arguments} argument(s)"));
                self.macros.commands.insert(target, Macro { arguments, default_arg: None, body, paired: None });
                Ok(true)
            }
            "let" => {
                let target = self.defined_name(col)?;
                self.skip_spaces();
                if self.current() == Some('=') {
                    self.pos += 1;
                    self.skip_spaces();
                }
                let body = if self.current() == Some('\\') {
                    self.pos += 1;
                    let n = self.read_command_name();
                    match self.macros.commands.get(&n).cloned() {
                        Some(m) => {
                            self.macros.commands.insert(target, m);
                            return Ok(true);
                        }
                        None => format!("\\{n}"),
                    }
                } else {
                    let c = self.current().ok_or(Error::Syntax { column: col, message: "\\let without a target".into() })?;
                    self.pos += 1;
                    c.to_string()
                };
                self.macros.commands.insert(target, Macro { arguments: 0, default_arg: None, body, paired: None });
                Ok(true)
            }
            "DeclareMathOperator" => {
                let star = self.current() == Some('*');
                if star {
                    self.pos += 1;
                }
                let target = self.defined_name(col)?;
                let text = self.raw_text()?;
                let body = format!("\\operatorname{}{{{text}}}", if star { "*" } else { "" });
                self.registry.note(rules::L_MAC_1, format!("operator \\{target} defined as '{text}'"));
                self.macros.commands.insert(target, Macro { arguments: 0, default_arg: None, body, paired: None });
                Ok(true)
            }
            "DeclarePairedDelimiter" => {
                let target = self.defined_name(col)?;
                let open = self.raw_text()?.trim().to_string();
                let close = self.raw_text()?.trim().to_string();
                self.macros.commands.insert(target, Macro { arguments: 1, default_arg: None, body: String::new(), paired: Some((open, close)) });
                Ok(true)
            }
            "newenvironment" | "renewenvironment" => {
                let target = self.raw_text()?.trim().to_string();
                if let Some(n) = self.raw_optional() {
                    if n.trim() != "0" {
                        return self.error(col, "\\newenvironment with arguments is not supported");
                    }
                }
                let open = self.raw_text()?;
                let close = self.raw_text()?;
                self.macros.environments.insert(target, (open, close));
                Ok(true)
            }
            "usepackage" | "RequirePackage" | "require" => {
                self.raw_optional();
                let package = self.raw_text()?;
                self.registry.note(rules::L_ENT_3, format!("\\{name}{{{package}}} ignored: the supported packages are already loaded"));
                Ok(true)
            }
            "makeatletter" | "makeatother" | "ignorespaces" | "leavevmode" | "sc" | "scshape" | "upshape" | "itshape" | "bfseries" | "mdseries" | "rmfamily" | "sffamily"
            | "ttfamily" => Ok(true),
            "aftergroup" => {
                // TeX primitive: defers the next token; here it is just discarded
                self.skip_spaces();
                if self.current() == Some('\\') {
                    self.pos += 1;
                    self.read_command_name();
                } else if self.current().is_some() {
                    self.pos += 1;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Raw content up to the matching `\end{name}` (which is consumed).
    fn environment_body(&mut self, name: &str, col: usize) -> R<String> {
        let open: Vec<char> = format!("\\begin{{{name}}}").chars().collect();
        let close: Vec<char> = format!("\\end{{{name}}}").chars().collect();
        let start = self.pos;
        let mut level = 1;
        let mut i = start;
        while i < self.c.len() {
            if self.c[i..].starts_with(&open) {
                level += 1;
                i += open.len();
            } else if self.c[i..].starts_with(&close) {
                level -= 1;
                if level == 0 {
                    self.pos = i + close.len();
                    return Ok(self.c[start..i].iter().collect());
                }
                i += close.len();
            } else {
                i += 1;
            }
        }
        self.tolerate(rules::E_REC_5, col, format!("\\begin{{{name}}} without \\end{{{name}}}"), "closed at the end")?;
        self.pos = self.c.len();
        Ok(self.c[start..].iter().collect())
    }

    /// Package commands defined through other commands (physics, braket, xfrac, units).
    fn package_expansion(&mut self, name: &str, col: usize) -> R<Option<String>> {
        let d = |p: bool| if p { "\\partial" } else { "\\mathrm{d}" };
        Ok(Some(match name {
            "dv" | "odv" | "pdv" | "derivative" | "partialderivative" | "pd" => {
                let partial = name.starts_with('p');
                let order = self.raw_optional();
                let f = self.raw_text()?;
                self.skip_spaces();
                let x = if self.current() == Some('{') { Some(self.raw_text()?) } else { None };
                self.skip_spaces();
                let y = if partial && x.is_some() && self.current() == Some('{') { Some(self.raw_text()?) } else { None };
                let dd = d(partial);
                match (x, y, order) {
                    (None, _, None) => format!("\\frac{{{dd}}}{{{dd} {f}}}"),
                    (None, _, Some(n)) => format!("\\frac{{{dd}^{{{n}}}}}{{{dd} {f}^{{{n}}}}}"),
                    (Some(x), None, None) => format!("\\frac{{{dd} {f}}}{{{dd} {x}}}"),
                    (Some(x), None, Some(n)) => {
                        format!("\\frac{{{dd}^{{{n}}} {f}}}{{{dd} {x}^{{{n}}}}}")
                    }
                    (Some(x), Some(y), _) => {
                        format!("\\frac{{{dd}^{{2}} {f}}}{{{dd} {x} \\, {dd} {y}}}")
                    }
                }
            }
            "ket" => format!("\\left| {} \\right\\rangle", self.raw_text()?),
            "bra" => format!("\\left\\langle {} \\right|", self.raw_text()?),
            "braket" | "Braket" => {
                let x = self.raw_text()?;
                format!("\\left\\langle {} \\right\\rangle", x.replacen('|', " \\middle| ", 2))
            }
            "Set" | "set" => {
                let x = self.raw_text()?;
                match x.split_once('|') {
                    Some((a, b)) => format!("\\left\\{{ {a} \\;\\middle|\\; {b} \\right\\}}"),
                    None => format!("\\left\\{{ {x} \\right\\}}"),
                }
            }
            "abs" => format!("\\left| {} \\right|", self.raw_text()?),
            "norm" => format!("\\left\\| {} \\right\\|", self.raw_text()?),
            "sfrac" | "nicefrac" => {
                let a = self.raw_text()?;
                let b = self.raw_text()?;
                format!("{{}}^{{{a}}}\\!/{{}}_{{{b}}}")
            }
            "unitfrac" => {
                let v = self.raw_optional();
                let a = self.raw_text()?;
                let b = self.raw_text()?;
                format!("{}\\,{{}}^{{\\mathrm{{{a}}}}}\\!/{{}}_{{\\mathrm{{{b}}}}}", v.unwrap_or_default())
            }
            "unit" if self.current() == Some('[') => {
                // \unit[3]{m} from the units package
                let v = self.raw_optional().unwrap_or_default();
                let u = self.raw_text()?;
                format!("{v}\\,\\mathrm{{{u}}}")
            }
            _ => {
                let _ = col;
                return Ok(None);
            }
        }))
    }
}

impl Reader<'_> {
    /// A command mathra does not know. In tolerant mode: a `\n` that came from JSON becomes a line break,
    /// `²` becomes the character, `\5` becomes the digit, and the rest shows up in red.
    fn unknown_command(&mut self, name: &str, start: usize, col: usize, items: &mut Vec<Node>) -> R<Option<Stop>> {
        if self.strict {
            return self.error(col, format!("unsupported command: \\{name}"));
        }
        if let Some(remainder) = name.strip_prefix('n').or_else(|| name.strip_prefix('r')).or_else(|| name.strip_prefix('t').filter(|_| name.len() <= 2)) {
            self.tolerate(rules::E_REC_2, col, format!("\\{name}"), "JSON '\\n' read as a line break")?;
            self.inject(start, &format!(" {remainder}"), col)?;
            return Ok(None);
        }
        if name == "u" {
            let hex: String = self.c[self.pos..].iter().take(4).collect();
            if let Some(c) = u32::from_str_radix(&hex, 16).ok().filter(|_| hex.len() == 4).and_then(char::from_u32) {
                self.tolerate(rules::E_REC_2, col, format!("\\u{hex} is a JSON escape"), &format!("read as '{c}'"))?;
                self.pos += 4;
                self.inject(start, &c.to_string(), col)?;
                return Ok(None);
            }
        }
        // \vertA, \alphax: a space forgotten after a known command
        // ASCII names only: a character like '∠' after the backslash has no prefix to cut
        for cut in (2..if name.is_ascii() { name.len() } else { 0 }).rev() {
            let (prefix, remainder) = name.split_at(cut);
            if known(prefix) {
                self.tolerate(rules::E_REC_1, col, format!("unsupported command: \\{name}"), &format!("read as \\{prefix} {remainder}"))?;
                self.inject(start, &format!("\\{prefix} {remainder}"), col)?;
                return Ok(None);
            }
        }
        if name.chars().count() == 1 && !name.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.tolerate(rules::E_REC_1, col, format!("'\\{name}'"), "backslash before a symbol ignored")?;
            self.inject(start, name, col)?;
            if matches!(name, "\"") {
                self.pos += 1;
            }
            return Ok(None);
        }
        self.tolerate(rules::E_REC_1, col, format!("unsupported command: \\{name}"), "drawn in red")?;
        items.push(Node::Colored { color: "#c0392b".into(), base: Box::new(Node::Text { parts: vec![TextPart::Text(format!("\\{name}"))] }) });
        Ok(None)
    }
}

/// Does mathra know this command? Used to recover `\vertA` as `\vert A`.
fn known(name: &str) -> bool {
    symbols::command(name).is_some()
        || symbols::greek(name).is_some()
        || symbols::function(name).is_some()
        || symbols::large_operator(name).is_some()
        || symbols::accent(name).is_some()
        || matches!(
            name,
            "begin"
                | "end"
                | "binom"
                | "dbinom"
                | "cfrac"
                | "textbf"
                | "textit"
                | "textrm"
                | "mathit"
                | "boldsymbol"
                | "bmatrix"
                | "pmatrix"
                | "vmatrix"
                | "overbrace"
                | "underbrace"
                | "widehat"
                | "tilde"
                | "dot"
                | "ddot"
                | "big"
                | "Big"
                | "bigg"
                | "Bigg"
                | "newline"
                | "not"
                | "centernot"
                | "notin"
                | "neq"
                | "nabla"
                | "frac"
                | "sqrt"
                | "text"
                | "mathrm"
                | "mathbf"
                | "mathbb"
                | "mathcal"
                | "left"
                | "right"
                | "cdot"
                | "times"
                | "quad"
                | "qquad"
                | "vec"
                | "hat"
                | "bar"
                | "cal"
                | "bf"
                | "rm"
                | "it"
                | "tt"
                | "sf"
                | "overline"
                | "underline"
                | "mathsf"
                | "mathtt"
                | "boxed"
                | "cancel"
                | "dfrac"
                | "tfrac"
        )
}

/// `\\frac{\\sqrt{3}}{2}`: every command comes with a doubled backslash. Undoes one level of escaping,
/// as long as there is no command with a single backslash (then the double backslashes are real line breaks).
pub fn undo_doubled_backslashes(s: &str) -> Option<String> {
    let c: Vec<char> = s.chars().collect();
    let mut doubles = 0;
    let mut i = 0;
    while i < c.len() {
        if c[i] == '\\' {
            let mut n = 0;
            while i < c.len() && c[i] == '\\' {
                n += 1;
                i += 1;
            }
            if c.get(i).is_some_and(|x| x.is_ascii_alphabetic()) {
                if n % 2 == 1 {
                    return None;
                }
                doubles += 1;
            }
        } else {
            i += 1;
        }
    }
    if doubles == 0 {
        return None;
    }
    let mut output = String::with_capacity(s.len());
    let mut i = 0;
    while i < c.len() {
        if c[i] == '\\' {
            let mut n: usize = 0;
            while i < c.len() && c[i] == '\\' {
                n += 1;
                i += 1;
            }
            output.extend(std::iter::repeat_n('\\', n.div_ceil(2)));
        } else {
            output.push(c[i]);
            i += 1;
        }
    }
    Some(output)
}

/// `\frac` stored in JSON without escaping the backslash becomes form feed + "rac"; `\begin` becomes backspace + "egin";
/// `\times`, `\right`, `\neq` become tab, CR and line feed. Rebuilds the command when the following
/// letters form a known command; otherwise the control character becomes a space.
fn repair_controls(s: &str) -> Option<String> {
    let letter_of = |c: char| match c {
        '\u{0C}' => Some('f'),
        '\u{08}' => Some('b'),
        '\u{0B}' => Some('v'),
        '\u{07}' => Some('a'),
        '\t' => Some('t'),
        '\r' => Some('r'),
        '\n' => Some('n'),
        _ => None,
    };
    if !s.chars().any(|c| letter_of(c).is_some()) {
        return None;
    }
    let c: Vec<char> = s.chars().collect();
    let mut output = String::with_capacity(s.len() + 8);
    let mut changed = false;
    for (i, ch) in c.iter().enumerate() {
        let Some(letter) = letter_of(*ch) else {
            output.push(*ch);
            continue;
        };
        let following: String = c[i + 1..].iter().take_while(|x| x.is_ascii_alphabetic()).collect();
        let name = format!("{letter}{following}");
        let rebuild = (1..=name.len()).rev().any(|n| known(&name[..n]) && n > 1);
        if rebuild {
            output.push('\\');
            output.push(letter);
            changed = true;
        } else {
            output.push(' ');
        }
    }
    changed.then_some(output)
}
