use chrono::{DateTime, Local};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};

/// The payload of a page load
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    /// A list of button bindings
    #[serde(default)]
    pub buttons: Vec<ButtonSpec>,

    /// A list of texts
    #[serde(default)]
    pub text: Vec<TextSpec>,

    /// A date time for when the page needs to be automatically refreshed.
    ///
    /// The string must be strictly in ISO8601 format, or page loading will fail.
    pub refresh: Option<DateTime<Local>>,

    /// For any of the buttons in this page, if no press color is set this color will be used
    pub default_press_color: Option<HexColor>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ButtonSpec {
    /// X coordinate of this button
    pub x: u32,

    /// Y coordinate of this button
    pub y: u32,

    /// The color of this button
    pub color: HexColor,

    /// The style of this button
    #[serde(default)]
    pub style: Style,

    /// The width of the button
    pub width: Option<u8>,

    /// The color this button should have if it is being pressed
    pub press_color: Option<HexColor>,

    /// Action to perform when this button is pressed
    pub on_press: Option<Action>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextSpec {
    /// X coordinate of this button
    pub x: u32,

    /// Y coordinate of this button
    pub y: u32,

    /// The text to print
    pub text: String,

    /// The color of this button
    pub color: HexColor,

    /// Maximum width of the text
    pub width: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Style {
    /// The button just lights up
    #[default]
    Plain,

    /// The button pulses the given color
    Pulse,

    /// The button flashes between its primary color and the given color
    ///
    /// A missing color is interpreted as "black".
    Flash(Option<HexColor>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    /// Navigate to the given URL
    #[serde(rename = "navigate")]
    Navigate { href: String },
    /// Open a browser at the given URL
    #[serde(rename = "open")]
    Browser { href: String },
}
