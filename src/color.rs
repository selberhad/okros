use bitflags::bitflags;

// Minimal placeholder for color/attribute constants.
bitflags! {
    pub struct Attr: u32 {
        const BOLD      = 1 << 0;
        const UNDERLINE = 1 << 1;
        const REVERSE   = 1 << 2;
        const DIM       = 1 << 3;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8);

impl Color {
    pub const BLACK: Self = Self(0);
    pub const RED: Self = Self(1);
    pub const GREEN: Self = Self(2);
    pub const YELLOW: Self = Self(3);
    pub const BLUE: Self = Self(4);
    pub const MAGENTA: Self = Self(5);
    pub const CYAN: Self = Self(6);
    pub const WHITE: Self = Self(7);
}
