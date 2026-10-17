//! Glyphs that grow: tall parentheses, radicals, braces and long arrows.
//! First try the font's ready-made variants; if none is enough, assemble from parts.

use crate::font::{font, Part};
use crate::Error;

use super::{glyph_id, k, Item, MathBox, Style};

type R<T> = Result<T, Error>;

/// Vertical glyph with a total height of at least `target` (already at the style's scale).
pub fn vertical(c: char, target: f32, e: Style) -> R<MathBox> {
    let f = font();
    let s = e.scale();
    let id = f.glyph(c).ok_or(Error::MissingGlyph { character: c })?;
    let base = glyph_id(id, s);
    if base.total() >= target {
        return Ok(base);
    }
    let variants = f.variants(id, true);
    for (v, measure) in &variants {
        if measure * s >= target {
            return Ok(glyph_id(*v, s));
        }
    }
    if let Some(parts) = f.assembly(id, true) {
        return Ok(assemble(&parts, target / s, s, true));
    }
    Ok(variants.last().map_or(base, |(v, _)| glyph_id(*v, s)))
}

/// Horizontal glyph with a width of at least `target`.
pub fn horizontal(c: char, target: f32, e: Style) -> R<MathBox> {
    let f = font();
    let s = e.scale();
    let id = f.glyph(c).ok_or(Error::MissingGlyph { character: c })?;
    let width = |g| {
        let (x0, _, x1, _) = f.limits(g);
        (x1 - x0) * s
    };
    if width(id) >= target {
        return Ok(glyph_id(id, s));
    }
    let variants = f.variants(id, false);
    for (v, _) in &variants {
        if width(*v) >= target {
            return Ok(glyph_id(*v, s));
        }
    }
    if let Some(parts) = f.assembly(id, false) {
        return Ok(assemble(&parts, target / s, s, false));
    }
    Ok(variants.last().map_or(glyph_id(id, s), |(v, _)| glyph_id(*v, s)))
}

/// Delimiter centered on the math axis, covering `target` of total height.
pub fn delimiter(c: char, target: f32, e: Style) -> R<MathBox> {
    let mut math_box = vertical(c, target, e)?;
    let axis = k(e, |c| c.axis_height());
    let offset = axis - (math_box.height - math_box.depth) / 2.0;
    math_box.shift_vertical(offset);
    math_box.single_glyph = false;
    Ok(math_box)
}

/// Assembles the parts (bottom to top or left to right), repeating the extenders
/// as few times as needed to reach the size, with overlap between connectors.
fn assemble(parts: &[Part], target: f32, s: f32, vertical: bool) -> MathBox {
    let f = font();
    let overlap = f.min_overlap();
    let sequence = |n: usize| -> Vec<Part> { parts.iter().flat_map(|p| std::iter::repeat_n(*p, if p.extender { n } else { 1 })).collect() };
    let size = |seq: &[Part], ov: f32| -> f32 { seq.iter().map(|p| p.advance).sum::<f32>() - ov * seq.len().saturating_sub(1) as f32 };
    let mut n = 0;
    let mut seq = sequence(n);
    while size(&seq, overlap) < target && n < 200 {
        n += 1;
        seq = sequence(n);
    }
    // spread the leftover by increasing the overlap, never beyond the shortest connector
    let joined = seq.len().saturating_sub(1).max(1) as f32;
    let slack = seq.windows(2).map(|w| w[0].end.min(w[1].start)).fold(f32::INFINITY, f32::min);
    let excess = (size(&seq, overlap) - target).max(0.0);
    let ov = (overlap + excess / joined).min(slack.max(overlap));

    let mut math_box = MathBox::empty();
    let mut cursor = 0.0;
    for p in &seq {
        let (x0, y0, _, _) = f.limits(p.id);
        let (x, y) = if vertical { (0.0, (cursor - y0) * s) } else { ((cursor - x0) * s, 0.0) };
        math_box.items.push(Item::Glyph { id: p.id, x, y, scale: s, color: None });
        let g = glyph_id(p.id, s);
        if vertical {
            math_box.width = math_box.width.max(g.width);
        } else {
            math_box.height = math_box.height.max(g.height);
            math_box.depth = math_box.depth.max(g.depth);
        }
        cursor += p.advance - ov;
    }
    let total = (cursor + ov) * s;
    if vertical {
        math_box.height = total;
        math_box.depth = 0.0;
    } else {
        math_box.width = total;
    }
    math_box
}
