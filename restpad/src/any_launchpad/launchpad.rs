use std::collections::HashMap;

// ButtonStyle has the superset of capabilities, no point in duplicating it
pub use launchy::s::Button;
use tokio::sync::mpsc;

/// LaunchPad Abstraction Layer :)
pub trait Launchpad {
    fn supports_brightness(&self) -> bool;

    /// Set brightness on a scale from 0..8
    fn set_brightness(&mut self, brightness: u8) -> anyhow::Result<()>;

    fn clear(&mut self) -> anyhow::Result<()>;

    fn set_all(&mut self, buttons: Buttons) -> anyhow::Result<()>;

    fn receiver(&mut self) -> &mut mpsc::Receiver<InputMessage>;
}

pub type Buttons = HashMap<Button, ButtonStyle>;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ButtonStyle {
    Palette(PaletteColor),
    Rgb(RgbColor),
    Flash(PaletteColor, PaletteColor),
    Pulse(PaletteColor),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// An RGB color. Each component may go up to 255.
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        RgbColor { r, g, b }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct PaletteColor(pub u8);

impl PaletteColor {
    pub const BLACK: PaletteColor = Self(0);
    pub const DARK_GRAY: PaletteColor = Self(1);
    pub const LIGHT_GRAY: PaletteColor = Self(2);
    pub const WHITE: PaletteColor = Self(3);
    pub const LIGHT_RED: PaletteColor = Self(4);
    pub const RED: PaletteColor = Self(5);
    pub const ORANGE: PaletteColor = Self(9);
    pub const YELLOW: PaletteColor = Self(13);
    pub const LIME_GREEN: PaletteColor = Self(17);
    pub const GREEN: PaletteColor = Self(21);
    pub const SLIGHTLY_LIGHT_GREEN: PaletteColor = Self(29);
    pub const LIGHT_BLUE: PaletteColor = Self(37);
    pub const BLUE: PaletteColor = Self(45);
    pub const PURPLE: PaletteColor = Self(49);
    pub const MAGENTA: PaletteColor = Self(53);
    pub const PINK: PaletteColor = Self(57);
    pub const BROWN: PaletteColor = Self(61);
    pub const CYAN: PaletteColor = Self(90);
}

impl From<PaletteColor> for ButtonStyle {
    fn from(color: PaletteColor) -> Self {
        ButtonStyle::Palette(color)
    }
}

impl From<&PaletteColor> for ButtonStyle {
    fn from(color: &PaletteColor) -> Self {
        Self::from(*color)
    }
}

impl From<RgbColor> for ButtonStyle {
    fn from(color: RgbColor) -> Self {
        ButtonStyle::Rgb(color)
    }
}

impl From<&RgbColor> for ButtonStyle {
    fn from(color: &RgbColor) -> Self {
        Self::from(*color)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum InputMessage {
    Press(Button),
    Release(Button),
}
