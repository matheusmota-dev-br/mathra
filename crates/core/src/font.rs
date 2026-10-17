//! Math font embedded in the binary. Nothing is read from the user's system.

use std::fmt::Write;
use std::sync::OnceLock;

use ttf_parser::math::{Constants, MathValue};
pub use ttf_parser::GlyphId;
use ttf_parser::{Face, OutlineBuilder};

static DATA: &[u8] = include_bytes!("../fonts/STIXTwoMath-Regular.otf");
pub const NAME: &str = "STIX Two Math";

pub struct Font {
    face: Face<'static>,
}

pub fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| Font { face: Face::parse(DATA, 0).expect("valid embedded font") })
}

/// One part of an assembled glyph (tall parenthesis, long brace).
#[derive(Debug, Clone, Copy)]
pub struct Part {
    pub id: GlyphId,
    pub start: f32,
    pub end: f32,
    pub advance: f32,
    pub extender: bool,
}

impl Font {
    pub fn units_per_em(&self) -> f32 {
        f32::from(self.face.units_per_em())
    }

    pub fn glyph(&self, c: char) -> Option<GlyphId> {
        self.face.glyph_index(c)
    }

    pub fn advance(&self, id: GlyphId) -> f32 {
        f32::from(self.face.glyph_hor_advance(id).unwrap_or(0))
    }

    /// (x_min, y_min, x_max, y_max) of the outline, in font units.
    pub fn limits(&self, id: GlyphId) -> (f32, f32, f32, f32) {
        match self.face.glyph_bounding_box(id) {
            Some(r) => (f32::from(r.x_min), f32::from(r.y_min), f32::from(r.x_max), f32::from(r.y_max)),
            None => (0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Glyph outline as the `d` of an SVG `<path>`, with y pointing up (as in the font).
    pub fn outline(&self, id: GlyphId) -> String {
        let mut path = PathWriter(String::new());
        self.face.outline_glyph(id, &mut path);
        path.0.trim_end().to_string()
    }

    fn constants(&self) -> Constants<'static> {
        self.face.tables().math.and_then(|m| m.constants).expect("the embedded font has a MATH table")
    }

    /// A constant from the MATH table, in font units.
    pub fn c(&self, f: impl Fn(&Constants<'static>) -> MathValue<'static>) -> f32 {
        f32::from(f(&self.constants()).value)
    }

    pub fn script_scale(&self) -> f32 {
        f32::from(self.constants().script_percent_scale_down()) / 100.0
    }

    pub fn script_script_scale(&self) -> f32 {
        f32::from(self.constants().script_script_percent_scale_down()) / 100.0
    }

    pub fn display_operator_min_height(&self) -> f32 {
        f32::from(self.constants().display_operator_min_height())
    }

    pub fn radical_degree_raise(&self) -> f32 {
        f32::from(self.constants().radical_degree_bottom_raise_percent()) / 100.0
    }

    pub fn italic_correction(&self, id: GlyphId) -> f32 {
        self.face.tables().math.and_then(|m| m.glyph_info).and_then(|g| g.italic_corrections).and_then(|c| c.get(id)).map_or(0.0, |v| f32::from(v.value))
    }

    pub fn accent_point(&self, id: GlyphId) -> Option<f32> {
        self.face.tables().math.and_then(|m| m.glyph_info).and_then(|g| g.top_accent_attachments).and_then(|c| c.get(id)).map(|v| f32::from(v.value))
    }

    pub fn min_overlap(&self) -> f32 {
        self.face.tables().math.and_then(|m| m.variants).map_or(0.0, |v| f32::from(v.min_connector_overlap))
    }

    /// Ready-made size variants, smallest to largest: (glyph, measure).
    pub fn variants(&self, id: GlyphId, vertical: bool) -> Vec<(GlyphId, f32)> {
        self.face
            .tables()
            .math
            .and_then(|m| m.variants)
            .and_then(|v| if vertical { v.vertical_constructions } else { v.horizontal_constructions }.get(id))
            .map(|c| c.variants.into_iter().map(|g| (g.variant_glyph, f32::from(g.advance_measurement))).collect())
            .unwrap_or_default()
    }

    /// Parts to assemble a glyph larger than every variant, bottom to top
    /// (vertical) or left to right (horizontal).
    pub fn assembly(&self, id: GlyphId, vertical: bool) -> Option<Vec<Part>> {
        let v = self.face.tables().math.and_then(|m| m.variants)?;
        let constructions = if vertical { v.vertical_constructions } else { v.horizontal_constructions };
        let assembly = constructions.get(id)?.assembly?;
        Some(
            assembly
                .parts
                .into_iter()
                .map(|p| Part {
                    id: p.glyph_id,
                    start: f32::from(p.start_connector_length),
                    end: f32::from(p.end_connector_length),
                    advance: f32::from(p.full_advance),
                    extender: p.part_flags.extender(),
                })
                .collect(),
        )
    }
}

struct PathWriter(String);

fn num(v: f32) -> String {
    let rounded = (v * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded}")
    }
}

impl OutlineBuilder for PathWriter {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "M{} {} ", num(x), num(y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "L{} {} ", num(x), num(y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let _ = write!(self.0, "Q{} {} {} {} ", num(x1), num(y1), num(x), num(y));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let _ = write!(self.0, "C{} {} {} {} {} {} ", num(x1), num(y1), num(x2), num(y2), num(x), num(y));
    }
    fn close(&mut self) {
        self.0.push_str("Z ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_without_outline_has_zero_bounds() {
        let f = font();
        assert_eq!(f.limits(f.glyph(' ').unwrap()), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn quadratic_curve_becomes_q() {
        // STIX Two Math is CFF (cubics only), but the outline writer also accepts TrueType fonts
        let mut c = PathWriter(String::new());
        c.quad_to(1.0, 2.25, 3.0, 4.0);
        assert_eq!(c.0, "Q1 2.3 3 4 ");
    }
}
