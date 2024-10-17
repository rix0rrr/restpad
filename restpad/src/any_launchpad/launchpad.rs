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
    pub const WHITE: PaletteColor = Self(3);
    pub const RED: PaletteColor = Self(5);
    pub const YELLOW: PaletteColor = Self(13);
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
