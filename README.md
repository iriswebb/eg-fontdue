# eg-fontdue # A TTF/OTF renderer for `embedded_graphics`

`eg-fontdue` implements `embedded_graphics`'s [`TextRenderer`](https://docs.rs/embedded-graphics/latest/embedded_graphics/text/renderer/trait.TextRenderer.html) and [`CharacterStyle`](https://docs.rs/embedded-graphics/latest/embedded_graphics/text/renderer/trait.CharacterStyle.html) traits over the [`fontdue`](https://github.com/mooman219/fontdue) crate. Allowing for the rendering of arbitrary TTF/OTF fonts at any size.

Basic anti-aliasing is implemented, the anti-aliasing engine automatically chooses the inverse of the text color as the background color, if you do not want this, specify an anti-aliasing color with `FontdueTextStyle::with_aa_color`.

Since glyphs have to be manually rasterized, rendering times may vary, `alloc` is also required

```rust
use embedded_graphics::{pixelcolor::BinaryColor, text::Text};

// Load a font using `fontdue`
let ttf_font_data = include_bytes!("assets/font.ttf");
let font = fontdue::Font::from_bytes(ttf_font_data, fontdue::FontSettings::default())?;

// Specify color and location
let style = eg_fontdue::FontdueTextStyle::new(&font, BinaryColor::On, 40);
let rendered_text = Text::new("Hello!", Point::new(100, 100), style);

// Render
rendered_text.draw(display)?;
```
