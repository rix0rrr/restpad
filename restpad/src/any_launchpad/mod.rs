mod launchpad;
mod mk3_mini;
pub use launchpad::*;
mod colors;
pub use colors::rgb_to_palette;

pub fn discover() -> Option<Box<dyn Launchpad>> {
    if let Some(x) = mk3_mini::Mk3::open() {
        return Some(Box::new(x));
    }
    None
}
