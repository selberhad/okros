// Integration with okros Session pipeline
// This demonstrates feeding MUD output through the full pipeline:
// MUD → ANSI → Telnet → MCCP → Scrollback

// NOTE: This file uses types from the main okros crate, so it needs to be
// compiled with access to ../../src/*. For now, we'll test manually or
// create a test in the main crate's tests/ directory.

use crate::game::World;
use crate::parser;

/// Wrapper that integrates internal MUD with okros Session
pub struct MudSession<D> {
    pub world: World,
    pub session: okros::session::Session<D>,
}

impl<D: okros::mccp::Decompressor> MudSession<D> {
    pub fn new(decomp: D, width: usize, height: usize, lines: usize) -> Self {
        let world = World::new();
        let session = okros::session::Session::new(decomp, width, height, lines);

        Self { world, session }
    }

    /// Execute a command and feed output to Session pipeline
    pub fn execute(&mut self, input: &str) -> Result<(), String> {
        let cmd = parser::parse(input)?;
        let output = self.world.execute(cmd);

        // Feed MUD output through Session pipeline
        // This goes through: MCCP → Telnet → ANSI → Scrollback
        self.session.feed(output.as_bytes());

        Ok(())
    }

    /// Get current scrollback content as text
    pub fn get_scrollback_text(&self) -> String {
        let viewport = self.session.scrollback.viewport_slice();
        let bytes: Vec<u8> = viewport
            .iter()
            .map(|&attr| (attr & 0xFF) as u8)
            .collect();
        String::from_utf8_lossy(&bytes).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use okros::mccp::PassthroughDecomp;

    #[test]
    fn test_mud_through_session_pipeline() {
        let mut mud_session = MudSession::new(PassthroughDecomp::new(), 80, 24, 200);

        // Execute "look" command
        mud_session.execute("look").unwrap();

        // Verify scrollback contains room description
        let text = mud_session.get_scrollback_text();
        assert!(text.contains("Forest Clearing"), "Should show room name");
        assert!(text.contains("forest clearing"), "Should show description");
        assert!(text.contains("rusty sword"), "Should show item");
    }

    #[test]
    fn test_navigation_through_pipeline() {
        let mut mud_session = MudSession::new(PassthroughDecomp::new(), 80, 24, 200);

        // Go north to forest
        mud_session.execute("go north").unwrap();

        let text = mud_session.get_scrollback_text();
        assert!(text.contains("Dense Forest"), "Should be in forest now");
    }

    #[test]
    fn test_item_management_through_pipeline() {
        let mut mud_session = MudSession::new(PassthroughDecomp::new(), 80, 24, 200);

        // Take sword
        mud_session.execute("take rusty sword").unwrap();

        let text = mud_session.get_scrollback_text();
        assert!(text.contains("You take the rusty sword"), "Should confirm taking sword");

        // Check inventory
        mud_session.execute("inventory").unwrap();

        let text = mud_session.get_scrollback_text();
        assert!(text.contains("rusty sword"), "Should show sword in inventory");
    }

    #[test]
    fn test_deterministic_sequence() {
        let mut mud_session = MudSession::new(PassthroughDecomp::new(), 80, 24, 200);

        // Run deterministic command sequence
        let commands = vec![
            "look",
            "take rusty sword",
            "go north",
            "go south",
            "go east",
            "inventory",
        ];

        for cmd in commands {
            mud_session.execute(cmd).unwrap();
        }

        let text = mud_session.get_scrollback_text();

        // Should be in cave now (east from clearing)
        assert!(text.contains("Dark Cave"), "Should end in cave");

        // Should have sword in inventory
        assert!(text.contains("rusty sword"), "Should carry sword");
    }
}
