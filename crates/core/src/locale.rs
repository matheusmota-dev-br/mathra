//! Writing conventions per language. The tree does not change; only how it is written and read.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    PtBr,
    #[default]
    En,
}

impl Locale {
    pub fn code(self) -> &'static str {
        match self {
            Locale::PtBr => "pt-BR",
            Locale::En => "en",
        }
    }

    /// L-NUM-2: decimal separator in the output.
    pub fn decimal_separator(self) -> char {
        match self {
            Locale::PtBr => ',',
            Locale::En => '.',
        }
    }

    /// L-NUM-1: is a comma between digits a decimal separator in the input?
    pub fn comma_is_decimal(self) -> bool {
        matches!(self, Locale::PtBr)
    }

    /// L-FUN-1: how the function is written, from its canonical name.
    pub fn function_name(self, canonical: &str) -> String {
        match (self, canonical) {
            (Locale::PtBr, "sin") => "sen".into(),
            (Locale::PtBr, "tan") => "tg".into(),
            (Locale::PtBr, "cot") => "cotg".into(),
            (Locale::PtBr, "csc") => "cossec".into(),
            (Locale::PtBr, "arcsin") => "arcsen".into(),
            (Locale::PtBr, "arctan") => "arctg".into(),
            (Locale::PtBr, "sinh") => "senh".into(),
            (Locale::PtBr, "gcd") => "mdc".into(),
            (Locale::PtBr, "lcm") => "mmc".into(),
            (_, other) => other.into(),
        }
    }

    /// How the function is read aloud.
    pub fn speak_function(self, canonical: &str) -> String {
        let name = match (self, canonical) {
            (Locale::PtBr, "sin") => "seno",
            (Locale::PtBr, "cos") => "cosseno",
            (Locale::PtBr, "tan") => "tangente",
            (Locale::PtBr, "cot") => "cotangente",
            (Locale::PtBr, "sec") => "secante",
            (Locale::PtBr, "csc") => "cossecante",
            (Locale::PtBr, "log") => "logaritmo",
            (Locale::PtBr, "ln") => "logaritmo natural",
            (Locale::PtBr, "gcd") => "mdc",
            (Locale::PtBr, "lcm") => "mmc",
            (Locale::En, "sin") => "sine",
            (Locale::En, "cos") => "cosine",
            (Locale::En, "tan") => "tangent",
            (Locale::En, "cot") => "cotangent",
            (Locale::En, "sec") => "secant",
            (Locale::En, "csc") => "cosecant",
            (Locale::En, "log") => "log",
            (Locale::En, "ln") => "natural log",
            (_, other) => return self.function_name(other),
        };
        match self {
            Locale::PtBr => format!("{name} de"),
            Locale::En => format!("{name} of"),
        }
    }
}

impl FromStr for Locale {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "pt-br" | "pt" | "pt_br" => Ok(Locale::PtBr),
            "en" | "en-us" | "en_us" => Ok(Locale::En),
            other => Err(format!("unknown locale: {other} (use en or pt-BR)")),
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
