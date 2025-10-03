//! Global state placeholders.
//! Apply Toy 3 pattern (unsafe static mut + helpers) when wiring real state.

// TODO: Replace with real types (screen, config, current_session, etc.)
// pub struct Screen { /* fields */ }
// static mut SCREEN: Option<Screen> = None;
// pub fn screen() -> &'static mut Screen { unsafe { SCREEN.as_mut().unwrap() } }

