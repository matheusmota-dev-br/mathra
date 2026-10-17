use std::process::ExitCode;
use std::{env, fs, io};

use mathra::{convert, Locale, Options};

const HELP: &str = "\
mathra — equations to SVG, no TeX install

USAGE
  mathra <equation> [options]
  echo '<equation>' | mathra [options]

OPTIONS
  -o, --output <file>     write the SVG to a file instead of printing it
  -l, --locale <code>     en (default) or pt-BR
      --inline            inline style, as in $…$; the default is display style
      --strict            refuse invalid LaTeX, like TeX (the default recovers and warns)
      --explain           show the tree, the speech and the rules applied
      --tree              print only the tree of meaning as JSON
      --speech            print only the screen reader phrase
      --text              print only the linear text
      --batch             read one equation per input line; with --output <folder>,
                          write 1.svg, 2.svg…; print ok, warning<TAB>… or error<TAB>… per line
  -V, --version           show the version
  -h, --help              show this help

EXAMPLES
  mathra '3.14' -o pi.svg
  mathra '3,14' --locale pt-BR --explain
  mathra '\\sin^2 x' --speech
";

enum Mode {
    Batch,
    Svg,
    Explain,
    Tree,
    Speech,
    Text,
}

struct Args {
    tex: Option<String>,
    output: Option<String>,
    locale: Locale,
    display: bool,
    strict: bool,
    mode: Mode,
}

fn read_args() -> Result<Option<Args>, String> {
    let mut args = Args { tex: None, output: None, locale: Locale::default(), display: true, strict: false, mode: Mode::Svg };
    let mut it = env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("mathra {}", mathra::VERSION);
                return Ok(None);
            }
            "-o" | "--output" => args.output = Some(it.next().ok_or("missing file after --output")?),
            "-l" | "--locale" => args.locale = it.next().ok_or("missing code after --locale")?.parse()?,
            "--explain" => args.mode = Mode::Explain,
            "--inline" => args.display = false,
            "--strict" => args.strict = true,
            "--tree" => args.mode = Mode::Tree,
            "--speech" => args.mode = Mode::Speech,
            "--text" => args.mode = Mode::Text,
            "--batch" => args.mode = Mode::Batch,
            option if option.starts_with("--") => return Err(format!("unknown option: {option} (see mathra --help)")),
            _ if args.tex.is_none() => args.tex = Some(a),
            _ => return Err("pass a single equation, in quotes".into()),
        }
    }
    Ok(Some(args))
}

fn main() -> ExitCode {
    let args = match read_args() {
        Ok(Some(args)) => args,
        Ok(None) => return ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mathra: {e}");
            return ExitCode::from(2);
        }
    };

    if let Mode::Batch = args.mode {
        return batch(&args);
    }

    let tex = match args.tex {
        Some(t) => t,
        None => match io::read_to_string(io::stdin()) {
            Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => {
                eprint!("{HELP}");
                return ExitCode::from(2);
            }
        },
    };

    let result = match convert(&tex, &Options { locale: args.locale, display: args.display, strict: args.strict }) {
        Ok(r) => {
            if !matches!(args.mode, Mode::Explain) {
                for a in &r.warnings {
                    eprintln!("mathra: warning {}: {}", a.rule, a.description);
                }
            }
            r
        }
        Err(e) => {
            eprintln!("mathra: {e}");
            return ExitCode::from(1);
        }
    };

    let text = match args.mode {
        Mode::Svg | Mode::Batch => result.svg,
        Mode::Tree => result.tree.to_json(),
        Mode::Speech => result.speech,
        Mode::Text => result.text,
        Mode::Explain => {
            let mut s = format!("input    {tex}\nlocale   {}\ntext     {}\nspeech   {}\ntree     {}\n", args.locale, result.text, result.speech, result.tree.to_json());
            if result.applied.is_empty() {
                s.push_str("rules    no rule changed the result\n");
            } else {
                s.push_str("rules\n");
                for a in &result.applied {
                    s.push_str(&format!("  {:<9} {}\n", a.rule, a.description));
                }
            }
            if !result.warnings.is_empty() {
                s.push_str("warnings\n");
                for a in &result.warnings {
                    s.push_str(&format!("  {:<9} {}\n", a.rule, a.description));
                }
            }
            print!("{s}");
            return ExitCode::SUCCESS;
        }
    };

    match args.output {
        Some(path) => {
            if let Err(e) = fs::write(&path, text) {
                eprintln!("mathra: could not write {path}: {e}");
                return ExitCode::from(1);
            }
        }
        None => println!("{text}"),
    }
    ExitCode::SUCCESS
}

fn batch(args: &Args) -> ExitCode {
    use std::io::{BufRead, Write};
    let options = Options { locale: args.locale, display: args.display, strict: args.strict };
    let mut output = io::BufWriter::new(io::stdout().lock());
    for (i, line) in io::stdin().lock().split(b'\n').enumerate() {
        let Ok(bytes) = line else { break };
        let status = match String::from_utf8(bytes) {
            Err(_) => "error\tthe line is not UTF-8".to_string(),
            Ok(tex) => batch_status(&tex, i, args, &options),
        };
        let _ = writeln!(output, "{status}");
    }
    ExitCode::SUCCESS
}

/// Status of one batch line: `ok`, `warning<TAB>…` or `error<TAB>…`.
fn batch_status(tex: &str, i: usize, args: &Args, options: &Options) -> String {
    let tex = tex.strip_suffix('\r').unwrap_or(tex);
    let r = match convert(tex, options) {
        Ok(r) => r,
        Err(e) => return format!("error\t{e}"),
    };
    if let Some(folder) = &args.output {
        let path = format!("{folder}/{}.svg", i + 1);
        if let Err(e) = fs::write(&path, &r.svg) {
            return format!("error\tcould not write {path}: {e}");
        }
    }
    match r.warnings.first() {
        Some(a) => format!("warning\t{} {}", a.rule, a.description),
        None => "ok".to_string(),
    }
}
