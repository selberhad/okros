// Terminfo ACS capability fetching via ncurses (read-only use).

#[derive(Default, Debug, Clone)]
pub struct AcsCaps {
    pub smacs: Option<String>,
    pub rmacs: Option<String>,
}

pub fn get_acs_caps() -> AcsCaps {
    // For safety in early bring-up, avoid terminfo calls here.
    // A future pass can call tigetstr after initializing the terminal properly.
    AcsCaps::default()
}
