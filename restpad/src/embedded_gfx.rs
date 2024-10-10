use embedded_graphics::mono_font::ascii::FONT_4X6;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Baseline;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb888,
    prelude::{OriginDimensions, Point, RgbColor, Size},
    text::Text,
    Drawable, Pixel,
};

use crate::any_launchpad::{Button, Buttons, RgbColor as OurRgbColor};

pub struct PadTarget<'a> {
    buttons: &'a mut Buttons,
    origin: (i32, i32),
    size: (u32, u32),
}

impl<'a> PadTarget<'a> {
    pub fn new(buttons: &'a mut Buttons, origin: (i32, i32), size: (u32, u32)) -> Self {
        Self {
            buttons,
            origin,
            size,
        }
    }
}

pub fn draw_text(
    buttons: &mut Buttons,
    text: &str,
    origin: (i32, i32),
    size: (u32, u32),
    x_shift: i32,
    color: OurRgbColor,
) {
    let mut target = PadTarget::new(buttons, origin, size);
    let char_style = MonoTextStyle::new(&FONT_4X6, color.into());
    let _ = Text::with_baseline(text, Point::new(x_shift, 0), char_style, Baseline::Top)
        .draw(&mut target);
}

pub fn text_width(text: &str) -> usize {
    text.chars().count() * 4
}

impl<'a> OriginDimensions for PadTarget<'a> {
    fn size(&self) -> Size {
        Size::from(self.size)
    }
}

impl<'a> DrawTarget for PadTarget<'a> {
    type Color = Rgb888;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x < self.size.0 as i32 && coord.y < self.size.1 as i32 {
                self.buttons.insert(
                    Button::grid(
                        (coord.x + self.origin.0) as u8,
                        (coord.y + self.origin.1) as u8,
                    ),
                    OurRgbColor::new(color.r(), color.g(), color.b()).into(),
                );
            }
        }
        Ok(())
    }
}

impl From<Rgb888> for OurRgbColor {
    fn from(value: Rgb888) -> Self {
        OurRgbColor {
            r: value.r(),
            g: value.g(),
            b: value.b(),
        }
    }
}

impl From<OurRgbColor> for Rgb888 {
    fn from(value: OurRgbColor) -> Self {
        Rgb888::new(value.r, value.g, value.b)
    }
}
