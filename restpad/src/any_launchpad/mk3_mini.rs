use std::collections::{HashMap, HashSet};

use super::{
    launchpad::{Buttons, InputMessage, Launchpad},
    Button, ButtonStyle, PaletteColor, RgbColor,
};
use launchy::{
    mini_mk3::{Input, Message, Output},
    InputDevice, InputDeviceHandler, OutputDevice,
};
use tokio::sync::mpsc;

pub struct Mk3 {
    prev_state: Buttons,
    _input: InputDeviceHandler,
    output: Output,
    receiver: mpsc::Receiver<InputMessage>,
}

impl Mk3 {
    pub fn open() -> Option<Mk3> {
        let (sender, receiver) = mpsc::channel(32);
        let output = Output::guess().ok()?;
        let input = Input::guess(move |message| {
            if let Some(converted) = convert_message(message) {
                // If this fails the receiver went away, but it's not like we can
                // stop the input loop anyway.
                let _ = sender.blocking_send(converted);
            }
        })
        .ok()?;

        Some(Mk3 {
            _input: input,
            output,
            receiver,
            prev_state: Default::default(),
        })
    }
}

static BLACK: ButtonStyle = ButtonStyle::Palette(PaletteColor::BLACK);

impl Launchpad for Mk3 {
    fn supports_brightness(&self) -> bool {
        true
    }

    fn set_brightness(&mut self, brightness: u8) -> anyhow::Result<()> {
        let brightness = (brightness * 16).min(127);
        self.output.set_brightness(brightness)?;
        Ok(())
    }

    fn clear(&mut self) -> anyhow::Result<()> {
        self.output.clear()?;
        self.prev_state.clear();
        Ok(())
    }

    fn set_all(&mut self, buttons: HashMap<Button, ButtonStyle>) -> anyhow::Result<()> {
        let candidates = buttons
            .keys()
            .cloned()
            .chain(self.prev_state.keys().cloned())
            .collect::<HashSet<_>>();

        self.output.set_buttons(
            candidates
                .iter()
                .map(|button| (button, buttons.get(button).unwrap_or(&BLACK)))
                .filter(|(button, style)| self.prev_state.get(button).unwrap_or(&BLACK) != *style)
                .filter(|(button, _)| is_valid_button(button))
                .map(|(button, style)| (*button, convert_button_style(style))),
        )?;
        self.prev_state = buttons;
        Ok(())
    }

    fn receiver(&mut self) -> &mut mpsc::Receiver<InputMessage> {
        &mut self.receiver
    }
}

fn convert_button_style(bs: &ButtonStyle) -> launchy::mini_mk3::ButtonStyle {
    match bs {
        ButtonStyle::Palette(x) => launchy::mini_mk3::ButtonStyle::Palette {
            color: convert_palette(x),
        },
        ButtonStyle::Rgb(x) => launchy::mini_mk3::ButtonStyle::Rgb {
            color: convert_rgb(x),
        },
        ButtonStyle::Flash(c1, c2) => launchy::mini_mk3::ButtonStyle::Flash {
            color1: convert_palette(c1),
            color2: convert_palette(c2),
        },
        ButtonStyle::Pulse(x) => launchy::mini_mk3::ButtonStyle::Pulse {
            color: convert_palette(x),
        },
    }
}

fn convert_palette(bs: &PaletteColor) -> launchy::mini_mk3::PaletteColor {
    launchy::mini_mk3::PaletteColor::new(bs.0)
}

fn convert_rgb(bs: &RgbColor) -> launchy::mini_mk3::RgbColor {
    launchy::mini_mk3::RgbColor::new(bs.r >> 4, bs.g >> 4, bs.b >> 4)
}

fn convert_message(m: Message) -> Option<InputMessage> {
    match m {
        Message::Press { button } => Some(InputMessage::Press(button)),
        Message::Release { button } => Some(InputMessage::Release(button)),
        _ => None,
    }
}

fn is_valid_button(button: &Button) -> bool {
    match *button {
        Button::ControlButton { index } => index < 8,
        Button::GridButton { x, y } => x < 9 && y < 8,
    }
}
