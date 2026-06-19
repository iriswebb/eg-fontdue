//! # eg-femtofont - A TTF/OTF renderer for `embedded_graphics`
//!
//! `eg-femtofont` implements `embedded_graphics`'s [`TextRenderer`](https://docs.rs/embedded-graphics/latest/embedded_graphics/text/renderer/trait.TextRenderer.html) and
//! [`CharacterStyle`](https://docs.rs/embedded-graphics/latest/embedded_graphics/text/renderer/trait.CharacterStyle.html) traits over the
//! [`femtofont`](https://github.com/iriswebb/femtofont) crate. Allowing for the rendering of arbitrary TTF/OTF fonts at any size.
//!
//! Basic anti-aliasing is implemented, the anti-aliasing engine automatically chooses the
//! inverse of the text color as the background color, if you do not want this, specify an
//! anti-aliasing color with `FemtoFontTextStyle::with_aa_color`.
//!
//! Since glyphs have to be manually rasterized, rendering times may vary,
//! `alloc` is also required
//!
//! ```rust
//! // load the font from raw data
//! let font = include_bytes!("assets/path_to_font.ttf") as &[u8];
//! let font = femtofont::Font::from_bytes_with_weight(font, 600.0, femtofont::FontSettings::default()).unwrap();
//!
//! // Red text anti-aliased as if it were on a blue background
//! let style = eg_femtofont::FemtoFontTextStyle::with_aa_color(&font, Rgb888::RED, Rgb888::BLUE, 20);
//! let rendered_text = Text::new("FemtoFont", Point::new(100, 100), style);
//!
//! // Draw
//! rendered_text.draw(&mut display).unwrap();
//! ```
#![no_std]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

use embedded_graphics::{
    pixelcolor::{Gray8, Rgb888},
    prelude::*,
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Alignment, Baseline,
    },
};
use femtofont::layout::{Layout, TextStyle, WrapStyle};

/// Text vertical alignment
#[derive(Debug, Clone, Copy, Default)]
pub enum VerticalAlign {
    #[default]
    /// Aligns to top of max height
    Top,
    /// Aligns to bottom of max height
    Bottom,
    /// Aligns to middle of max height
    Middle,
}

fn alpha_composite(background: Rgb888, foreground: Rgb888, alpha: u8) -> Rgb888 {
    let (r1, g1, b1) = (
        foreground.r() as u16,
        foreground.g() as u16,
        foreground.b() as u16,
    );
    let (r2, g2, b2) = (
        background.r() as u16,
        background.g() as u16,
        background.b() as u16,
    );

    let alpha = alpha as u16;
    let p = 255 - alpha;

    Rgb888::new(
        ((r1 * alpha + r2 * p) / 255) as u8,
        ((g1 * alpha + g2 * p) / 255) as u8,
        ((b1 * alpha + b2 * p) / 255) as u8,
    )
}

fn inverse(col: Rgb888) -> Rgb888 {
    let (r, g, b) = (col.r(), col.g(), col.b());
    Rgb888::new(Rgb888::MAX_R - r, Rgb888::MAX_G - g, Rgb888::MAX_B - b)
}

/// A text renderer for TTF and OTF fonts
#[derive(Debug, Clone, Copy)]
pub struct FemtoFontTextStyle<'a, C: PixelColor + From<Gray8> + From<Rgb888> + Into<Rgb888>> {
    /// A SFNT font
    pub font: &'a femtofont::Font<'a>,
    /// The color the text will be rendered in
    pub color: C,
    /// The color the font anti-aliases towards
    pub antialias_color: C,
    /// Size in pixels
    pub size: u16,
    /// Maximum Width
    pub max_width: Option<f32>,
    /// Maximum Height
    pub max_height: Option<f32>,
    /// Horizontal Alignment
    pub horiz_align: Alignment,
    /// Vertical Alignment
    pub vert_align_not_center: VerticalAlign,
    /// Line Height
    pub line_height: f32,
    /// Wraps words (if false, wraps letters)
    pub word_wrap: bool,
    /// Wrap hard breaks
    pub wrap_hard_breaks: bool,
}

impl<'a, C: PixelColor + From<Gray8> + From<Rgb888> + Into<Rgb888>> FemtoFontTextStyle<'a, C> {
    fn ascent(&self) -> u16 {
        self.font.horizontal_line_metrics(self.size as f32).ascent as u16
    }

    fn descent(&self) -> u16 {
        self.font.horizontal_line_metrics(self.size as f32).descent as u16
    }

    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => self.ascent().saturating_sub(1) as i32,
            Baseline::Bottom => -(self.descent() as i32),
            Baseline::Middle => (self.ascent() as i32 - self.descent() as i32) / 2,
            Baseline::Alphabetic => 0,
        }
    }
}

impl<'a, C: PixelColor + From<Gray8> + From<Rgb888> + Into<Rgb888>> FemtoFontTextStyle<'a, C>
where
    Rgb888: From<C>,
{
    /// Constructs a new text style
    pub fn new(font: &'a femtofont::Font, color: C, size: u16) -> Self {
        Self {
            font,
            color,
            antialias_color: inverse(Rgb888::from(color)).into(),
            size,
            max_width: None,
            max_height: None,
            horiz_align: Alignment::Left,
            vert_align_not_center: VerticalAlign::Top,
            line_height: 1.0 * size as f32,
            word_wrap: true,
            wrap_hard_breaks: true,
        }
    }

    /// Constructs a new text style with an antialiasing color
    pub fn with_aa_color(font: &'a femtofont::Font, color: C, aa_color: C, size: u16) -> Self {
        Self {
            font,
            color,
            antialias_color: aa_color,
            size,
            max_width: None,
            max_height: None,
            horiz_align: Alignment::Left,
            vert_align_not_center: VerticalAlign::Top,
            line_height: 1.0 * size as f32,
            word_wrap: true,
            wrap_hard_breaks: true,
        }
    }

    /// Renders a glyph at a certain location
    pub fn render_glyph_at<D: DrawTarget<Color = C>>(
        &self,
        idx: u16,
        x: f32,
        y: f32,
        target: &mut D,
    ) -> Result<Point, D::Error> {
        let (m, d) = self.font.rasterize_indexed(idx, self.size as f32);

        let bbx = Rectangle::new(
            Point {
                x: x as i32,
                y: y as i32,
            },
            Size {
                width: m.width as u32,
                height: m.height as u32,
            },
        );

        let mut data_iter = d.iter();

        let c8: Rgb888 = self.color.into();
        let bc8: Rgb888 = self.antialias_color.into();

        bbx.points()
            .filter_map(|p| {
                let l = *(data_iter.next()?);
                if l != 0 {
                    Some(Pixel(p, alpha_composite(bc8, c8, l).into()))
                } else {
                    None
                }
            })
            .draw(target)?;

        Ok(Point::new(m.advance_width as i32, m.advance_height as i32))
    }

    /// Generates a font layout from the text style
    pub fn generate_layout(&self, text: &str, position: Point) -> Layout {
        let mut layout = Layout::new(femtofont::layout::CoordinateSystem::PositiveYDown);
        let settings = femtofont::layout::LayoutSettings {
            x: position.x as f32,
            y: position.y as f32,
            line_height: self.line_height,
            max_height: self.max_height,
            max_width: self.max_width,
            wrap_style: match self.word_wrap {
                true => WrapStyle::Word,
                false => WrapStyle::Letter,
            },
            wrap_hard_breaks: self.wrap_hard_breaks,
            horizontal_align: match self.horiz_align {
                Alignment::Center => femtofont::layout::HorizontalAlign::Center,
                Alignment::Left => femtofont::layout::HorizontalAlign::Left,
                Alignment::Right => femtofont::layout::HorizontalAlign::Right,
            },
            vertical_align: match self.vert_align_not_center {
                VerticalAlign::Middle => femtofont::layout::VerticalAlign::Middle,
                VerticalAlign::Top => femtofont::layout::VerticalAlign::Top,
                VerticalAlign::Bottom => femtofont::layout::VerticalAlign::Bottom,
            },
        };

        layout.reset(&settings);

        layout.append(&[self.font], &TextStyle::new(text, self.size as f32, 0));

        layout
    }
}

impl<'a, C: PixelColor + From<Gray8> + From<Rgb888> + Into<Rgb888>> CharacterStyle
    for FemtoFontTextStyle<'a, C>
{
    type Color = C;

    fn set_text_color(&mut self, text_color: Option<C>) {
        // TODO: support transparent text
        if let Some(color) = text_color {
            self.color = color;
        }
    }

    // TODO: implement additional methods
}

impl<'a, C: PixelColor + From<Gray8> + From<Rgb888> + Into<Rgb888>> TextRenderer
    for FemtoFontTextStyle<'a, C>
where
    Rgb888: From<C>,
{
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        Rgb888: From<C>,
        D: DrawTarget<Color = Self::Color>,
    {
        let mut position = position + Point::new(0, self.baseline_offset(baseline));
        let layout = self.generate_layout(text, position);

        for glyph in layout.glyphs() {
            position += self.render_glyph_at(
                glyph.key.glyph_index,
                glyph.x,
                glyph.y - (self.baseline_offset(Baseline::Middle) as f32 * 2.0),
                target,
            )?;
        }

        Ok(position)
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        baseline: Baseline,
        _: &mut D,
    ) -> Result<Point, D::Error>
    where
        Rgb888: From<C>,
        D: DrawTarget<Color = Self::Color>,
    {
        let position = position + Point::new(0, self.baseline_offset(baseline));

        Ok(position + Size::new(width, 0))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        let position = position + Point::new(0, self.baseline_offset(baseline));
        let layout = self.generate_layout(text, position);

        let mut dx = 0.0;
        let mut dy = 0.0;
        for met in layout.glyphs().iter().map(|g| {
            self.font
                .metrics_indexed(g.key.glyph_index, self.size as f32)
        }) {
            dy += met.advance_height;
            dx += met.advance_width;
        }

        let bounding_box = Rectangle::new(
            position - Size::new(0, self.ascent().saturating_sub(1) as u32 + (dy as u32)),
            Size::new(dx as u32, self.line_height()),
        );

        TextMetrics {
            bounding_box,
            next_position: position + Size::new(dx as u32, 0),
        }
    }

    fn line_height(&self) -> u32 {
        self.line_height as u32
    }
}
