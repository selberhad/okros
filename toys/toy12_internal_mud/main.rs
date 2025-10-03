mod game;
mod parser;

use game::World;
use std::io::{self, Write};

fn main() {
    let mut world = World::new();

    println!("\x1b[1;33m=== okros Internal MUD Demo ===\x1b[0m");
    println!("Type 'help' for commands, 'quit' to exit.\n");

    // Show initial room
    print!("{}", world.execute(parser::Command::Look));

    loop {
        // Prompt
        print!("\x1b[37m> \x1b[0m");
        io::stdout().flush().unwrap();

        // Read input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        // Parse command
        let cmd = match parser::parse(&input) {
            Ok(cmd) => cmd,
            Err(e) => {
                println!("\x1b[31m{}\x1b[0m", e);
                continue;
            }
        };

        // Check for quit
        if matches!(cmd, parser::Command::Quit) {
            print!("{}", world.execute(cmd));
            break;
        }

        // Execute and display output
        print!("{}", world.execute(cmd));
    }
}
