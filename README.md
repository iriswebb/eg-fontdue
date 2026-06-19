# eg-femtofont - A TTF/OTF renderer for `embedded_graphics`

`eg-femtofont` implements `embedded_graphics`'s [`TextRenderer`](https://docs.rs/embedded-graphics/latest/embedded_graphics/text/renderer/trait.TextRenderer.html) and
[`CharacterStyle`](https://docs.rs/embedded-graphics/latest/embedded_graphics/text/renderer/trait.CharacterStyle.html) traits over the
[`femtofont`](https://github.com/iriswebb/femtofont) crate. Allowing for the rendering of arbitrary TTF/OTF fonts at any size.

Basic anti-aliasing is implemented, the anti-aliasing engine automatically chooses the
inverse of the text color as the background color, if you do not want this, specify an
anti-aliasing color with `FemtoFontTextStyle::with_aa_color`.

Since glyphs have to be manually rasterized, rendering times may vary,
`alloc` is also required

````rust
// load the font from raw data
let font = include_bytes!("assets/path_to_font.ttf") as &[u8];
let font = femtofont::Font::from_bytes_with_weight(font, 600.0, femtofont::FontSettings::default()).unwrap();
//!
// Red text anti-aliased as if it were on a blue background
let style = eg_femtofont::FemtoFontTextStyle::with_aa_color(&font, Rgb888::RED, Rgb888::BLUE, 20);
let rendered_text = Text::new("FemtoFont", Point::new(100, 100), style);
//!
// Draw
rendered_text.draw(&mut display).unwrap();! ```
````
