//! `.oft` bitmap fonts (unit `mc_font`).

use omsi_cfg::CfgFile;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontChar {
    pub ch: char,
    pub x0: i32,
    pub x1: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Font {
    pub path: PathBuf,
    pub name: String,
    pub bitmap: String,
    pub alpha: String,
    pub height: i32,
    pub gap: i32,
    pub chars: Vec<FontChar>,
}

impl Font {
    /// One `.oft` may define several fonts (`[newfont]` blocks).
    pub fn load_all(path: &Path) -> Result<Vec<Font>, omsi_cfg::CfgError> {
        let f = CfgFile::read(path)?;
        let mut out: Vec<Font> = Vec::new();
        let mut r = f.reader();
        while let Some(k) = r.next_keyword() {
            match k.as_str() {
                "newfont" => {
                    let name = r.str().to_string();
                    let bitmap = r.str().to_string();
                    let alpha = r.str().to_string();
                    let height = r.i32();
                    let gap = r.i32();
                    // Most stock .oft files document the format with a dummy block whose
                    // lines are placeholders ("{name}", "{Höhe in Pixeln …}"), so they read
                    // as a font of no height. Loading those as fonts only gives a lookup
                    // something wrong to land on.
                    if height > 0 && !name.starts_with('{') {
                        out.push(Font { path: f.path.clone(), name, bitmap, alpha, height, gap, chars: Vec::new() });
                    }
                }
                "char" => {
                    let c = r.line();
                    let ch = c.chars().next().unwrap_or(' ');
                    let x0 = r.i32();
                    let x1 = r.i32();
                    let y = r.i32();
                    if let Some(font) = out.last_mut() {
                        font.chars.push(FontChar { ch, x0, x1, y });
                    }
                }
                _ => {}
            }
        }
        Ok(out)
    }

    /// Width of a space character in pixels: the font's own space glyph if defined,
    /// else the width of '0' (for digit-only fonts), else the font's first glyph
    /// (the font's default advance), or half the font height.
    pub fn space_width(&self) -> i32 {
        self.exact_glyph(' ')
            .map(|g| (g.x1 - g.x0).max(0))
            .or_else(|| self.exact_glyph('0').map(|g| (g.x1 - g.x0).max(0)))
            .or_else(|| self.chars.first().map(|g| (g.x1 - g.x0).max(0)))
            .unwrap_or_else(|| (self.height / 2).max(1))
    }

    /// The glyph Omsi.exe draws for `c` (0x5d66a4): the character itself (or the same
    /// character read in another code page: a font and the text it shows need not have
    /// been read in the same one - a Russian font's `Л` is the byte 0xCB, which a font file
    /// without other Cyrillic reads as `Ë`), else for a small Latin letter a-z its capital,
    /// else nothing: its lookup (0x5d660c) gives -1 and the text is drawn and measured
    /// without it (its callers skip a negative index), only the font's gap moves on.
    /// Whitespace the font lacks is still given a width ([`Font::space_width`]) so words
    /// keep their gaps - the font's first glyph, drawn in its place before, put a `|` in
    /// front of the MAN Lion's City's odometer (#360).
    pub fn glyph(&self, c: char) -> Option<&FontChar> {
        if c.is_whitespace() {
            return self.exact_glyph(' ').or_else(|| {
                omsi_cfg::codepage::char_variants(c)
                    .into_iter()
                    .find_map(|v| self.chars.iter().find(|g| g.ch == v))
            });
        }
        self.exact_glyph(c)
            .or_else(|| c.is_ascii_lowercase().then(|| self.exact_glyph(c.to_ascii_uppercase())).flatten())
    }

    /// Whether the font has a glyph of its own for `c`.
    pub fn has_glyph(&self, c: char) -> bool {
        self.exact_glyph(c).is_some()
    }

    fn exact_glyph(&self, c: char) -> Option<&FontChar> {
        self.chars.iter().find(|g| g.ch == c).or_else(|| {
            omsi_cfg::codepage::char_variants(c)
                .into_iter()
                .find_map(|v| self.chars.iter().find(|g| g.ch == v))
        })
    }

    /// A text's width as Omsi.exe measures it (0x5d6c00, the scripts' `TextLength` too):
    /// its glyphs' widths and the font's gap between each two of them.
    pub fn text_width(&self, text: &str) -> i32 {
        let n = text.chars().count() as i32;
        let w: i32 = text
            .chars()
            .map(|c| {
                if c.is_whitespace() {
                    self.space_width()
                } else {
                    self.glyph(c).map(|g| (g.x1 - g.x0).max(0)).unwrap_or(0)
                }
            })
            .sum();
        w + (n - 1).max(0) * self.gap
    }
}

/// Horizontal placement of a text in its texture (`[texttexture_enh]` orientation and
/// grid; a plain `[texttexture]` is centred).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextAlign {
    /// 0 and 4 centred over the letters (the gap behind the last one left out), leaning left
    /// when the centre falls between two pixels; 1 left, 2 right, 3 centred and rounded,
    /// 5 centred leaning right.
    pub orientation: i32,
    /// The text starts on a multiple of this many pixels (0 and 1: anywhere).
    pub grid: i32,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign { orientation: 0, grid: 1 }
    }
}

impl TextAlign {
    /// Left edge of a text `advance` pixels wide (gaps included) in a `width`-pixel texture.
    pub fn offset(&self, width: i32, advance: i32, gap: i32) -> i32 {
        // "over the spacing": the gap after the last letter is not part of the text
        let visible = (advance - gap.max(0)).max(0);
        let x = match self.orientation {
            1 => 0.0,
            2 => (width - visible) as f32,
            3 => ((width - visible) as f32 / 2.0).round(),
            5 => ((width - visible) as f32 / 2.0).ceil(),
            // a plain [texttexture] (0) is centred like 4: over the letters without the gap
            // behind the last one, halved downwards. The LiAZ 5292's line display maps its
            // three digit cells at u 0.078/0.306/0.535 of a 166 px texture (13, 51, 89 px)
            // and its letter cell at u 0.554 of the 512 px one (283.6 px) - exactly where
            // "092 " and "092D" land this way; centring the gap too put every cell 5-6 px
            // (2 px) to the left, the "0" lost its left side and read as "D92".
            _ => ((width - visible) as f32 / 2.0).floor(),
        };
        let g = self.grid.max(1) as f32;
        let x = match self.orientation {
            2 | 5 => (x / g).ceil() * g,
            3 => (x / g).round() * g,
            _ => (x / g).floor() * g,
        };
        (x as i32).max(0)
    }
}

/// A loaded font with its glyph bitmaps, ready to draw text into RGBA images.
pub struct FontAtlas {
    pub font: Font,
    pub width: u32,
    pub height: u32,
    /// Colour bitmap (RGBA8) and alpha bitmap (RGBA8; the red channel is the coverage).
    pub color: Vec<u8>,
    pub alpha: Vec<u8>,
}

impl FontAtlas {
    /// `color`/`alpha` are the decoded font bitmaps (same size). When the font has no
    /// separate colour bitmap, pass the alpha image for both.
    pub fn new(font: Font, width: u32, height: u32, color: Vec<u8>, alpha: Vec<u8>) -> FontAtlas {
        FontAtlas { font, width, height, color, alpha }
    }

    /// Pixel width of `text` in this font (glyph advances including the gap after each).
    pub fn text_width(&self, text: &str) -> i32 {
        text.chars()
            .map(|ch| {
                let w = if ch.is_whitespace() {
                    self.font.space_width()
                } else {
                    self.font.glyph(ch).map(|g| (g.x1 - g.x0).max(0)).unwrap_or(0)
                };
                w + self.font.gap
            })
            .sum()
    }

    /// Render `text` centred into a `w`×`h` RGBA image (a text wider than the image is
    /// clipped at its edge, as OMSI's text textures are).
    /// `full_color` uses the font's colour bitmap, otherwise glyphs are filled with `rgb`;
    /// the alpha channel holds the coverage.
    pub fn render(&self, text: &str, w: u32, h: u32, full_color: bool, rgb: [u8; 3]) -> Vec<u8> {
        self.render_aligned(text, w, h, full_color, rgb, TextAlign::default())
    }

    /// `render` with the horizontal placement of `[texttexture_enh]`.
    pub fn render_aligned(&self, text: &str, w: u32, h: u32, full_color: bool, rgb: [u8; 3], align: TextAlign) -> Vec<u8> {
        // '@' breaks the text into lines, one glyph height each, from the top: the SD202's
        // matrix hands over "   NORDSPITZE   @   BAUERNHOF    @NORDSP.BAUERNH. " for a
        // 512x128 texture whose meshes map the lines separately. Drawn as one line and
        // squeezed to fit, that came out as a row of unreadable dots.
        //
        // Lines stay top-aligned (Omsi.exe): IBIS/ibox listboxes build a tall `@`-separated
        // string and map a fixed "selected row" mesh onto a band of that texture. Centring
        // the block shifted every row, so the highlight sat one stop behind the announcement.
        //
        // When the natural block is taller than the texture (Aachen ibox `ibox_hstverlauf`:
        // DIN height 56 × ~13 Fahrplan / ~17 Route lines into 550×600), the block is scaled
        // vertically to fit. Clipping the bottom cut off the current stop that the white
        // selection mesh samples at v≈0.82..1.0, so only ~3 names showed and the highlight
        // was wrong.
        //
        // Route mode ends with `@@@` after the current stop (`iboxVerlaufRoute`). Those
        // empty lines must not keep a slot after scale-to-fit: they would occupy the white
        // UV band while the name sits just above it (v≈0.76..0.82). Drop trailing empty
        // `@` segments so the last painted line lands at the bottom of the texture.
        if text.contains('@') {
            let lh = self.font.height.max(1) as u32;
            let mut lines: Vec<&str> = text.split('@').collect();
            while lines.last().is_some_and(|l| l.is_empty()) {
                lines.pop();
            }
            if lines.is_empty() {
                return vec![0u8; (w * h * 4) as usize];
            }
            let natural_h = (lines.len() as u32).saturating_mul(lh).max(1);
            let mut tall = vec![0u8; (w * natural_h * 4) as usize];
            for (i, line) in lines.iter().enumerate() {
                let y0 = i as u32 * lh;
                let img = self.render_aligned(line, w, lh, full_color, rgb, align);
                for y in 0..lh as usize {
                    let src = y * w as usize * 4;
                    let dst = (y0 as usize + y) * w as usize * 4;
                    tall[dst..dst + w as usize * 4].copy_from_slice(&img[src..src + w as usize * 4]);
                }
            }
            let mut out = vec![0u8; (w * h * 4) as usize];
            if natural_h <= h {
                out[..tall.len()].copy_from_slice(&tall);
            } else {
                for y in 0..h as usize {
                    let src_y = (y as u64 * natural_h as u64 / h as u64) as usize;
                    let src = src_y * w as usize * 4;
                    let dst = y * w as usize * 4;
                    out[dst..dst + w as usize * 4].copy_from_slice(&tall[src..src + w as usize * 4]);
                }
            }
            return out;
        }
        // (a text wider than the texture runs off its edge, as Omsi.exe draws it: the
        // start is not left of the texture and the rest is clipped, 0x5fb79c / 0x5d67bc)
        self.render_unscaled(text, w, h, full_color, rgb, align)
    }

    fn render_unscaled(&self, text: &str, w: u32, h: u32, full_color: bool, rgb: [u8; 3], align: TextAlign) -> Vec<u8> {
        let mut out = vec![0u8; (w * h * 4) as usize];
        let glyph_h = self.font.height.max(1) as i32;
        let y0 = (h as i32 - glyph_h) / 2;
        let mut x = align.offset(w as i32, self.text_width(text), self.font.gap);
        for ch in text.chars() {
            if ch.is_whitespace() {
                x += self.font.space_width() + self.font.gap;
                continue;
            }
            let Some(g) = self.font.glyph(ch) else {
                x += self.font.gap;
                continue;
            };
            let gw = (g.x1 - g.x0).max(0);
            for gy in 0..glyph_h {
                let sy = g.y + gy;
                let dy = y0 + gy;
                if sy < 0 || sy >= self.height as i32 || dy < 0 || dy >= h as i32 {
                    continue;
                }
                for gx in 0..gw {
                    let sx = g.x0 + gx;
                    let dx = x + gx;
                    if sx < 0 || sx >= self.width as i32 || dx < 0 || dx >= w as i32 {
                        continue;
                    }
                    let si = ((sy as u32 * self.width + sx as u32) * 4) as usize;
                    let a = self.alpha[si];
                    if a == 0 {
                        continue;
                    }
                    let di = ((dy as u32 * w + dx as u32) * 4) as usize;
                    let (r, gcol, b) = if full_color { (self.color[si], self.color[si + 1], self.color[si + 2]) } else { (rgb[0], rgb[1], rgb[2]) };
                    // alpha-over compositing onto the transparent target
                    let af = a as f32 / 255.0;
                    let inv = 1.0 - af;
                    out[di] = (r as f32 * af + out[di] as f32 * inv) as u8;
                    out[di + 1] = (gcol as f32 * af + out[di + 1] as f32 * inv) as u8;
                    out[di + 2] = (b as f32 * af + out[di + 2] as f32 * inv) as u8;
                    out[di + 3] = out[di + 3].max(a);
                }
            }
            x += gw + self.font.gap;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_does_not_draw_visible_first_glyph() {
        // A font like MAN New Lion's City odometer font: first character is '|',
        // digits follow, and there is no space character defined in the font.
        let font = Font {
            path: PathBuf::new(),
            name: "LCD_Test".into(),
            bitmap: "lcd.bmp".into(),
            alpha: "lcd_alpha.bmp".into(),
            height: 10,
            gap: 1,
            chars: vec![
                FontChar { ch: '|', x0: 0, x1: 2, y: 0 },
                FontChar { ch: '0', x0: 2, x1: 10, y: 0 },
                FontChar { ch: '1', x0: 10, x1: 18, y: 0 },
            ],
        };
        // Alpha bitmap where '|' has opaque pixels
        let mut alpha = vec![0u8; 18 * 10 * 4];
        for y in 0..10 {
            for x in 0..2 {
                let idx = (y * 18 + x) * 4;
                alpha[idx] = 255;
            }
        }
        let atlas = FontAtlas::new(font.clone(), 18, 10, alpha.clone(), alpha);

        // Leading spaces (as used in odometer padding e.g. "  1") must not draw '|'
        let rendered = atlas.render_aligned("  1", 40, 10, false, [255, 255, 255], TextAlign { orientation: 1, grid: 1 });
        // The first 10 pixels horizontally (where spaces sit) must have 0 alpha!
        for y in 0..10 {
            for x in 0..10 {
                let idx = (y * 40 + x) * 4;
                assert_eq!(rendered[idx + 3], 0, "space pixel at ({x}, {y}) must be transparent");
            }
        }

        // glyph(' ') must not return the '|' character
        assert_eq!(font.glyph(' '), None);
        // space_width should fall back to '0' width (8)
        assert_eq!(font.space_width(), 8);
    }

    /// `@` lines start at the top of the texture. An IBIS listbox maps its selection band
    /// onto a fixed row; a centred block made that band show the previous stop.
    #[test]
    fn at_lines_are_drawn_from_the_top() {
        let font = Font {
            path: PathBuf::new(),
            name: "List".into(),
            bitmap: "list.bmp".into(),
            alpha: "list_alpha.bmp".into(),
            height: 8,
            gap: 0,
            chars: vec![FontChar { ch: 'X', x0: 0, x1: 4, y: 0 }],
        };
        let mut alpha = vec![0u8; 4 * 8 * 4];
        for y in 0..8 {
            for x in 0..4 {
                let i = (y * 4 + x) * 4;
                alpha[i] = 255;
                alpha[i + 3] = 255;
            }
        }
        let atlas = FontAtlas::new(font, 4, 8, alpha.clone(), alpha);
        // two lines in a 32-high texture: first line must occupy y=0..8, not a centred band
        let img = atlas.render_aligned("X@X", 8, 32, false, [255, 255, 255], TextAlign::default());
        let opaque = |y: u32| (0..8u32).any(|x| img[((y * 8 + x) * 4 + 3) as usize] > 0);
        assert!(opaque(0), "first @ line starts at the top");
        assert!(opaque(8), "second @ line follows immediately");
        assert!(!opaque(16), "no centred gap below the lines");
        assert!(!opaque(24), "bottom of a tall list texture stays empty");
    }

    /// Aachen ibox `ibox_hstverlauf` is 600 px with DIN height 56; the Fahrplan string has
    /// ~13 `@` lines (time+name for current and three ahead). Without scaling, the current
    /// stop (bottom of the string, sampled by the white mesh at v≈0.82..1) was clipped off.
    #[test]
    fn tall_at_lists_scale_so_the_last_line_stays_in_the_texture() {
        let font = Font {
            path: PathBuf::new(),
            name: "List".into(),
            bitmap: "list.bmp".into(),
            alpha: "list_alpha.bmp".into(),
            height: 56,
            gap: 0,
            chars: vec![FontChar { ch: 'X', x0: 0, x1: 4, y: 0 }],
        };
        let mut alpha = vec![0u8; 4 * 56 * 4];
        for y in 0..56 {
            for x in 0..4 {
                let i = (y * 4 + x) * 4;
                alpha[i] = 255;
                alpha[i + 3] = 255;
            }
        }
        let atlas = FontAtlas::new(font, 4, 56, alpha.clone(), alpha);
        // 13 lines like iboxVerlaufFahrplan: natural height 728 > 600
        let text = "@X@X@@X@X@@X@X@@X@X@";
        assert_eq!(text.split('@').count(), 13);
        let img = atlas.render_aligned(text, 8, 600, false, [255, 255, 255], TextAlign::default());
        let opaque = |y: u32| (0..8u32).any(|x| img[((y * 8 + x) * 4 + 3) as usize] > 0);
        // white selection band of ibox_text_3weiss.o3d: v 0.819..1.0 → y ≈ 491..600
        assert!(
            (491..600).any(opaque),
            "current-stop band at the bottom of the texture must stay painted"
        );
        // main list band v 0..0.82 → y 0..492 still has stop names (not only the bottom)
        assert!(
            (0..400).any(opaque),
            "scaled block keeps earlier stops in the upper list band"
        );
        // after trimming the trailing empty `@`, the last painted line fills the bottom
        assert!(opaque(580), "last content line sits near the texture bottom");
    }

    /// `iboxVerlaufRoute` pads with `@@@` after the current cabin stop. Those empty lines
    /// must not claim the white UV band once the block is scaled into 600 px.
    #[test]
    fn route_list_trailing_empty_at_lines_do_not_steal_the_white_band() {
        let font = Font {
            path: PathBuf::new(),
            name: "List".into(),
            bitmap: "list.bmp".into(),
            alpha: "list_alpha.bmp".into(),
            height: 56,
            gap: 0,
            chars: vec![FontChar { ch: 'X', x0: 0, x1: 4, y: 0 }],
        };
        let mut alpha = vec![0u8; 4 * 56 * 4];
        for y in 0..56 {
            for x in 0..4 {
                let i = (y * 4 + x) * 4;
                alpha[i] = 255;
                alpha[i + 3] = 255;
            }
        }
        let atlas = FontAtlas::new(font, 4, 56, alpha.clone(), alpha);
        // @@@@stop4@@@stop3@@@stop2@@@stop1@@@ — current stop is the last non-empty line
        let text = "@@@@X@@@X@@@X@@@X@@@";
        let img = atlas.render_aligned(text, 8, 600, false, [255, 255, 255], TextAlign::default());
        let opaque = |y: u32| (0..8u32).any(|x| img[((y * 8 + x) * 4 + 3) as usize] > 0);
        // Without trimming trailing empties, the current stop landed at y≈459..494 (mostly
        // above the white mesh). It must sit well inside v 0.819..1.0.
        assert!(opaque(560), "current stop must be inside the white selection band");
        assert!(opaque(590), "current stop reaches the bottom of the white box");
        assert!(
            (100..450).any(opaque),
            "earlier stops remain visible in the main list band"
        );
    }
}
