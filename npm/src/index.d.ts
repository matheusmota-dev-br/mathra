/** A language code: "en" (default), "pt-BR", "es", "fr", "de", "it", "ru", "zh", "ja", "ko", "hi", "ar", "id", "tr", "nl" or "pl". A region is accepted ("es-MX", "zh-CN"). */
export type Locale = "en" | "pt-BR" | "es" | "fr" | "de" | "it" | "ru" | "zh" | "ja" | "ko" | "hi" | "ar" | "id" | "tr" | "nl" | "pl" | (string & {});

export interface Options {
  /** How the equation is read, written and spoken. Default "en". */
  locale?: Locale;
  /** Display style, as in `$$…$$` (default), or inline with the text, as in `$…$` (false). */
  display?: boolean;
  /** Refuse invalid LaTeX, like TeX does, instead of recovering with warnings. Default false. */
  strict?: boolean;
}

/** A numbered rule from spec/rules.md or spec/tolerance.md, and what it did to this equation. */
export interface Rule {
  rule: string;
  description: string;
}

export interface Result {
  /** Standalone SVG with the speech in `<title>` and the source and tree in `<metadata>`. */
  svg: string;
  /** The phrase a screen reader says, in the locale's language. */
  speech: string;
  /** Linear text, close to UnicodeMath. */
  text: string;
  /** The tree of meaning. */
  tree: unknown;
  /** Rules that changed something in this equation, in the order they acted. */
  applied: Rule[];
  /** Input problems the tolerant mode worked around (E-REC-*); empty in strict mode. */
  warnings: Rule[];
}

/** Loads the WebAssembly module: await it once before converting in browsers and bundlers. In Node it does nothing. */
export function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<void>;

/** Converts math LaTeX to SVG. Throws on input it cannot read (or on any invalid input with `strict`). */
export function texToSvg(tex: string, options?: Options | Locale): string;

/** Converts math LaTeX and returns every output at once. */
export function render(tex: string, options?: Options | Locale): Result;

/** The supported languages, English first. */
export function locales(): { code: string; name: string }[];

/** The version of mathra inside this package. */
export const version: string;
