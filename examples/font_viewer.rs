use anyhow::Result;
use eg_fontdue::FontdueTextStyle;
use embedded_graphics::{
    geometry::AnchorPoint,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{renderer::TextRenderer, Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:#}");
    }
}

/// Draws a text and its bounding box.
fn draw_text<S: TextRenderer<Color = Rgb888>>(
    display: &mut SimulatorDisplay<Rgb888>,
    text: &Text<S>,
) -> Point {
    text.bounding_box()
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DARK_ORANGE, 1))
        .draw(display)
        .unwrap();

    text.draw(display).unwrap()
}

fn draw<S: TextRenderer<Color = Rgb888> + Copy>(
    display: &mut SimulatorDisplay<Rgb888>,
    style: S,
    line_height: u32,
    text: Option<&String>,
) {
    let abc = ('a'..='z').collect::<String>();
    let digits = ('0'..='9').collect::<String>();
    let string = if text.is_none() {
        format!(
            "The quick brown fox jumps over the lazy dog\n{}\n{}\n{}",
            abc,
            abc.to_ascii_uppercase(),
            digits
        )
        .to_string()
    } else {
        text.unwrap().clone()
    };

    let position = Point::new(5, 5 + line_height as i32);

    Line::with_delta(position.y_axis(), Point::zero() + display.size().x_axis())
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DIM_GRAY, 1))
        .draw(display)
        .unwrap();

    let text = Text::new(&string, position, style);
    draw_text(display, &text);

    let position = display.bounding_box().anchor_point(AnchorPoint::BottomLeft)
        + Point::new(5, -(line_height as i32) * 3 / 2);

    Line::with_delta(position.y_axis(), Point::zero() + display.size().x_axis())
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DIM_GRAY, 1))
        .draw(display)
        .unwrap();

    let p = draw_text(
        display,
        &Text::with_baseline("Top ", position, style, Baseline::Top),
    );

    let p = draw_text(
        display,
        &Text::with_baseline(
            "Middle ",
            Point::new(p.x, position.y),
            style,
            Baseline::Middle,
        ),
    );

    let p = draw_text(
        display,
        &Text::with_baseline(
            "Bottom ",
            Point::new(p.x, position.y),
            style,
            Baseline::Bottom,
        ),
    );

    draw_text(
        display,
        &Text::with_baseline(
            "Alphabetic",
            Point::new(p.x, position.y),
            style,
            Baseline::Alphabetic,
        ),
    );
}

fn try_main() -> Result<()> {
    use std::io::prelude::*;
    let argv: Vec<String> = std::env::args().collect();
    let file = argv.get(1).expect("Usage: fontfile [text] [size]");
    let text = argv.get(2);
    let size: u16 = argv
        .get(3)
        .or(Some(&"15".to_string()))
        .unwrap()
        .parse::<u16>()?;

    let mut file = std::fs::File::open(file)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let fdfont =
        fontdue::Font::from_bytes(data.as_ref(), fontdue::FontSettings::default()).unwrap();
    let style = FontdueTextStyle::new(&fdfont, Rgb888::WHITE, size);

    let hints_style = MonoTextStyle::new(&FONT_6X10, Rgb888::CSS_DIM_GRAY);
    let bottom_right = TextStyleBuilder::new()
        .baseline(Baseline::Bottom)
        .alignment(Alignment::Right)
        .build();

    let line_height = style.line_height();
    eprintln!("{}", style.line_height());
    let display_height = line_height * 8;
    let display_width = (line_height * 25).max(display_height);
    let display_size = Size::new(display_width, display_height);

    let mut display = SimulatorDisplay::<Rgb888>::new(display_size);

    let scale = (1200 / display_size.width).max(1);

    let settings = OutputSettingsBuilder::new().scale(scale).build();
    let mut window = Window::new("Font viewer", &settings);

    'main_loop: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'main_loop,
                _ => {}
            }
        }

        let mut hint = "Press M to toggle".to_string();

        display.clear(Rgb888::BLACK).unwrap();
        draw(&mut display, style, line_height, text);
        hint.insert_str(0, "TTF | ");

        let corner = display
            .bounding_box()
            .offset(-3)
            .anchor_point(AnchorPoint::BottomRight);
        Text::with_text_style(&hint, corner, hints_style, bottom_right)
            .draw(&mut display)
            .unwrap();
    }

    Ok(())
}
