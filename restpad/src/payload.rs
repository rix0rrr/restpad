use chrono::{DateTime, Local};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};

/// The payload of a page load
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Payload {
    /// A list of button bindings
    #[serde(default)]
    pub buttons: Vec<Button>,

    /// A date time for when the page needs to be automatically refreshed.
    ///
    /// The string must be strictly in ISO8601 format, or page loading will fail.
    pub refresh: Option<DateTime<Local>>,

    /// For any of the buttons in this page, if no press color is set this color will be used
    pub default_press_color: Option<HexColor>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Button {
    /// X coordinate of this button
    pub x: u32,

    /// Y coordinate of this button
    pub y: u32,

    /// The color of this button
    pub color: HexColor,

    /// The style of this button
    #[serde(default)]
    pub style: Style,

    /// The color this button should have if it is being pressed
    pub press_color: Option<HexColor>,

    /// Action to perform when this button is pressed
    pub on_press: Option<Action>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
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
pub enum Action {
    /// Navigate to the given URL
    Navigate(String),
    /// Open a browser at the given URL
    Browser(String),
}
