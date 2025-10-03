// Internal MUD for offline demo and testing
// Ported from toys/toy12_internal_mud/

pub mod game;
pub mod parser;

pub use game::World;
pub use parser::{Command, parse};
