mod any_launchpad;
mod embedded_gfx;
mod navigator;
mod payload;
mod preferences;

use cond::cond;
use embedded_gfx::{draw_text, text_width};
use hex_color::HexColor;
use payload::{Action, ButtonSpec};
use std::{collections::HashSet, pin::Pin, time::Duration};

use any_launchpad::{
    discover, rgb_to_palette, Button, ButtonStyle, Buttons, InputMessage, Launchpad, PaletteColor,
    RgbColor,
};
use anyhow::bail;
use clap::{command, Parser};
use disk_persist::DiskPersist;
use navigator::Navigator;
use preferences::Preferences;
use tokio::{
    select,
    time::{sleep, Sleep},
};

static GRID_WIDTH: u32 = 8;
static GRID_HEIGHT: u32 = 8;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL (or file name) to start navigation
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Plugging in the LaunchPad after we've started the program doesn't detect
    // it, at least not on macOS, without further work.
    let Some(lp) = discover() else {
        bail!("No LaunchPad found; please plug it in and restart the app.");
    };

    let mut restpad = RestPad::new(lp)?;
    restpad.init()?;
    restpad.navigate(&args.url).await?;
    restpad.main_loop().await?;

    Ok(())
}

struct RestPad {
    prefs: Preferences,
    navigator: Navigator,
    lp: Box<dyn Launchpad>,
    pressed_buttons: HashSet<Button>,
    counter: i32,
    timer: Option<Pin<Box<Sleep>>>,
    refresh_timer: Option<Pin<Box<Sleep>>>,
    y_scroll: u32,
}

impl RestPad {
    pub fn new(lp: Box<dyn Launchpad>) -> anyhow::Result<Self> {
        let prefs = DiskPersist::init("restpad")?.read()?.unwrap_or_default();

        Ok(RestPad {
            prefs,
            navigator: Navigator::new()?,
            lp,
            pressed_buttons: Default::default(),
            counter: 0,
            timer: None,
            refresh_timer: None,
            y_scroll: 0,
        })
    }

    fn save_settings(&self) -> anyhow::Result<()> {
        DiskPersist::init("restpad")?.write(&self.prefs)?;
        Ok(())
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        print_error(self.lp.clear());
        self.flush_brightness()?;
        Ok(())
    }

    fn flush_brightness(&mut self) -> anyhow::Result<()> {
        if self.lp.supports_brightness() {
            println!("Brightness {}", self.prefs.brightness);
            print_error(self.lp.set_brightness(self.prefs.brightness));
        }
        Ok(())
    }

    pub async fn navigate(&mut self, url: &str) -> anyhow::Result<()> {
        print_error(self.navigator.navigate(url).await);
        self.on_page_load();
        self.update_buttons()?;
        Ok(())
    }

    pub async fn main_loop(&mut self) -> anyhow::Result<()> {
        loop {
            select! {
                Some(m) = self.lp.receiver().recv() => {
                    print_error(self.handle_message(m).await);
                }
                Some(_) = await_optional(&mut self.timer) => {
                    self.counter += 1;
                    print_error(self.update_buttons());
                }
                Some(_) = await_optional(&mut self.refresh_timer) => {
                    print_error(self.refresh_on_timer().await);
                }
            };
        }
    }

    async fn handle_message(&mut self, message: InputMessage) -> anyhow::Result<()> {
        match message {
            InputMessage::Press(button) => {
                self.pressed_buttons.insert(button);
            }
            InputMessage::Release(button) => {
                self.pressed_buttons.remove(&button);

                let scrollable = self.scrollable_y_height() > 0;
                match button {
                    Button::UP => {
                        if self.prefs.brightness < 8 {
                            self.prefs.brightness += 1;
                        }
                        self.flush_brightness()?;
                        self.save_settings()?;
                    }
                    Button::DOWN => {
                        if self.prefs.brightness > 0 {
                            self.prefs.brightness -= 1;
                        }
                        self.flush_brightness()?;
                        self.save_settings()?;
                    }
                    Button::LEFT => {
                        self.navigator.back().await?;
                        self.on_page_load();
                    }
                    Button::RIGHT => {
                        self.navigator.forward().await?;
                        self.on_page_load();
                    }
                    Button::MIXER => {
                        self.navigator.refresh().await?;
                        self.on_page_load();
                    }
                    Button::GridButton { x, y: 0 } if x == GRID_WIDTH as u8 && scrollable => {
                        // The way we do pressed buttons, a pressed button will never be released
                        // if we adjust the scroll. So scrolling only works if no other buttons
                        // are pressed ^^
                        if self.y_scroll > 0 && self.pressed_buttons.is_empty() {
                            self.y_scroll -= 1;
                        }
                    }
                    Button::GridButton { x, y }
                        if x == GRID_WIDTH as u8 && y == GRID_HEIGHT as u8 - 1 && scrollable =>
                    {
                        if self.y_scroll < self.scrollable_y_height()
                            && self.pressed_buttons.is_empty()
                        {
                            self.y_scroll += 1;
                        }
                    }
                    Button::GridButton { x, y } => {
                        // Find the button that was pressed
                        if let Some(button) = self.find_button(Button::grid(x, y)) {
                            if let Some(action) = button.on_press {
                                match action {
                                    Action::Navigate { href } => {
                                        print_error(self.navigate(&href).await);
                                        return Ok(());
                                    }
                                    Action::Browser { href } => {
                                        print_error(webbrowser::open(&href));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        self.update_buttons()?;
        Ok(())
    }

    fn on_page_load(&mut self) {
        self.timer = None;
        self.refresh_timer = None;

        // Only start the timer if there are texts to scroll
        if let Some(payload) = self.navigator.current() {
            if !payload.text.is_empty() {
                self.timer = Some(Box::pin(sleep(Duration::from_millis(100))));
            }

            if let Some(refresh_secs) = payload.refresh_after_secs {
                self.refresh_timer =
                    Some(Box::pin(sleep(Duration::from_secs(refresh_secs as u64))));
            }
        }
        self.y_scroll = 0;
    }

    async fn refresh_on_timer(&mut self) -> anyhow::Result<()> {
        let mut buttons = self.calculate_buttons();
        buttons.insert(Button::MIXER, PaletteColor::YELLOW.into());
        print_error(self.lp.set_all(buttons));

        self.navigator.refresh().await?;
        self.on_page_load();
        self.update_buttons()?;

        Ok(())
    }

    fn calculate_buttons(&self) -> Buttons {
        let Some(payload) = self.navigator.current() else {
            return Default::default();
        };

        let mut buttons = Buttons::new();

        // UP and DOWN buttons control brightness
        buttons.insert(
            Button::UP,
            cond! {
                self.pressed_buttons.contains(&Button::UP) => PaletteColor::RED,
                self.prefs.brightness < 8 => PaletteColor::WHITE,
                _ => PaletteColor::BLACK
            }
            .into(),
        );
        buttons.insert(
            Button::DOWN,
            cond! {
                self.pressed_buttons.contains(&Button::DOWN) => PaletteColor::RED,
                self.prefs.brightness > 0 => PaletteColor::WHITE,
                _ => PaletteColor::BLACK
            }
            .into(),
        );
        // LEFT and RIGHT are for the browser history
        buttons.insert(
            Button::LEFT,
            cond! {
                self.pressed_buttons.contains(&Button::LEFT) => PaletteColor::YELLOW,
                self.navigator.has_history() => PaletteColor::WHITE,
                _ => PaletteColor::BLACK
            }
            .into(),
        );
        buttons.insert(
            Button::RIGHT,
            cond! {
                self.pressed_buttons.contains(&Button::RIGHT) => PaletteColor::YELLOW,
                self.navigator.has_future() => PaletteColor::WHITE,
                _ => PaletteColor::BLACK
            }
            .into(),
        );

        // The "refresh" button is always on
        buttons.insert(
            Button::MIXER,
            cond! {
            self.pressed_buttons.contains(&Button::MIXER) => PaletteColor::YELLOW,
            _ => PaletteColor::WHITE }
            .into(),
        );

        // The right-hand column is for vertical scroll
        let invis_height = self.scrollable_y_height();
        if invis_height > 0 {
            let up_button = Button::GridButton {
                x: GRID_WIDTH as u8,
                y: 0,
            };
            let down_button = Button::GridButton {
                x: GRID_WIDTH as u8,
                y: GRID_HEIGHT as u8 - 1,
            };
            buttons.insert(
                up_button,
                cond! {
                    self.pressed_buttons.contains(&up_button) => PaletteColor::YELLOW,
                    _ => PaletteColor::WHITE
                }
                .into(),
            );
            buttons.insert(
                down_button,
                cond! {
                    self.pressed_buttons.contains(&down_button) => PaletteColor::YELLOW,
                    _ => PaletteColor::WHITE
                }
                .into(),
            );

            // Indicate the scroll position
            let scroll_pos = self.y_scroll * (GRID_HEIGHT - 2 - 1) / invis_height;
            buttons.insert(
                Button::GridButton {
                    x: GRID_WIDTH as u8,
                    y: 1 + scroll_pos as u8,
                },
                PaletteColor::DARK_GRAY.into(),
            );
        }

        for button in &payload.buttons {
            // Reserve the right-hand column for the scroll bar
            if button.x >= GRID_WIDTH {
                continue;
            }

            // This adjusts the button in virtual-space to the pad in real-space
            let pads = self
                .pads_from_buttonspec(button)
                .into_iter()
                .filter(|p| !matches!(p, Button::GridButton { x, .. } if *x >= GRID_WIDTH as u8))
                .collect::<Vec<_>>();

            let is_pressed = pads.iter().any(|p| self.pressed_buttons.contains(p));
            let press_color = button
                .press_color
                .or_else(|| self.navigator.current().and_then(|p| p.default_press_color));

            let style = match press_color {
                Some(press_color) if is_pressed => ButtonStyle::Rgb(hex_to_rgb(press_color)),
                _ => parse_button_style(button),
            };

            for pad in pads {
                buttons.insert(pad, style);
            }
        }

        for text in &payload.text {
            let color = hex_to_rgb(text.color);
            let pos = (text.x as i32, text.y as i32 - self.y_scroll as i32);
            let size = (
                text.width
                    .unwrap_or(GRID_WIDTH)
                    .min(GRID_WIDTH - pos.0 as u32),
                6,
            );

            let invis_width = (text_width(&text.text) as i32 - size.0 as i32).max(0);
            let wait_margin = 10;

            let mut offset = self.counter % (wait_margin + invis_width);
            offset = (offset - wait_margin).max(0);

            // On odd iterations, we go backwards
            let iter = self.counter / (wait_margin + invis_width);
            if iter % 2 == 1 {
                offset = invis_width - offset;
            }

            let x_shift = -offset;
            draw_text(&mut buttons, &text.text, pos, size, x_shift, color);
        }
        buttons
    }

    fn update_buttons(&mut self) -> anyhow::Result<()> {
        let buttons = self.calculate_buttons();

        self.lp.set_all(buttons)?;
        Ok(())
    }

    fn y_max(&self) -> u32 {
        let Some(payload) = self.navigator.current() else {
            return 0;
        };

        let y_max_buttons = payload.buttons.iter().map(|b| b.y).max().unwrap_or(0);
        let y_max_text = payload.text.iter().map(|t| t.y + 6).max().unwrap_or(0);
        y_max_buttons.max(y_max_text)
    }

    fn scrollable_y_height(&self) -> u32 {
        (self.y_max() as i32 + 1 - GRID_HEIGHT as i32).max(0) as u32
    }

    fn find_button(&self, pad: Button) -> Option<ButtonSpec> {
        let Some(payload) = self.navigator.current() else {
            return None;
        };
        for button in &payload.buttons {
            let pads = self.pads_from_buttonspec(button);
            if pads.contains(&pad) {
                return Some(button.clone());
            }
        }
        None
    }

    fn pads_from_buttonspec(&self, button: &ButtonSpec) -> Vec<Button> {
        (0..button.width.unwrap_or(1).max(1))
            .filter_map(|k| {
                let adjusted_y = button.y as i32 - self.y_scroll as i32;
                if 0 <= adjusted_y && adjusted_y < GRID_HEIGHT as i32 {
                    Some(Button::grid(button.x as u8 + k, adjusted_y as u8))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}

fn print_error<A, E: std::fmt::Display>(e: Result<A, E>) -> Option<A> {
    match e {
        Ok(x) => Some(x),
        Err(x) => {
            eprintln!("{}", x);
            None
        }
    }
}

fn parse_button_style(b: &ButtonSpec) -> ButtonStyle {
    match b.style {
        payload::Style::Plain => ButtonStyle::Rgb(hex_to_rgb(b.color)),
        payload::Style::Pulse => {
            ButtonStyle::Pulse(PaletteColor(rgb_to_palette(hex_to_rgb(b.color))))
        }
        payload::Style::Flash(color2) => ButtonStyle::Flash(
            PaletteColor(rgb_to_palette(hex_to_rgb(b.color))),
            PaletteColor(rgb_to_palette(hex_to_rgb(color2.unwrap_or_default()))),
        ),
    }
}

fn hex_to_rgb(color: HexColor) -> RgbColor {
    RgbColor {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

async fn await_optional(t: &mut Option<Pin<Box<tokio::time::Sleep>>>) -> Option<()> {
    match t.as_mut() {
        Some(timer) => {
            timer.await;
            Some(())
        }
        None => None,
    }
}
