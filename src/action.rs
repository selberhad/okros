// Action - Trigger/replacement/gag system with regex matching
//
// Ported from mcl-cpp-reference/h/Action.h and mcl-cpp-reference/Alias.cc

use crate::plugins::stack::Interpreter;
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Trigger,     // Pattern match → execute commands
    Replacement, // Pattern match → substitute text
    Gag,         // Pattern match → suppress line
}

pub struct Action {
    pub pattern: String,
    pub commands: String,
    pub action_type: ActionType,
    compiled: Option<Box<dyn Any>>,
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("pattern", &self.pattern)
            .field("commands", &self.commands)
            .field("action_type", &self.action_type)
            .field("compiled", &self.compiled.is_some())
            .finish()
    }
}

impl Action {
    pub fn new(
        pattern: impl Into<String>,
        commands: impl Into<String>,
        action_type: ActionType,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            commands: commands.into(),
            action_type,
            compiled: None,
        }
    }

    /// Compile the pattern/replacement using the interpreter
    /// Must be called before check_match or check_replacement
    pub fn compile(&mut self, interp: &mut dyn Interpreter) {
        self.compiled = match self.action_type {
            ActionType::Trigger => interp.match_prepare(&self.pattern, &self.commands),
            ActionType::Replacement | ActionType::Gag => {
                let replacement = if self.action_type == ActionType::Gag {
                    "" // Gag = replace with empty string
                } else {
                    &self.commands
                };
                interp.substitute_prepare(&self.pattern, replacement)
            }
        };
    }

    /// Check if this action matches the text and run commands (for Trigger type)
    /// Returns Some(commands) if matched
    pub fn check_match(&self, text: &str, interp: &mut dyn Interpreter) -> Option<String> {
        if self.action_type != ActionType::Trigger {
            return None;
        }

        if let Some(compiled) = &self.compiled {
            interp.match_exec(compiled.as_ref(), text)
        } else {
            None
        }
    }

    /// Check if this action should replace text (for Replacement/Gag types)
    /// Returns Some(new_text) if matched and replaced
    pub fn check_replacement(&self, text: &str, interp: &mut dyn Interpreter) -> Option<String> {
        if self.action_type == ActionType::Trigger {
            return None;
        }

        if let Some(compiled) = &self.compiled {
            interp.match_exec(compiled.as_ref(), text)
        } else {
            None
        }
    }

    /// Parse action from command line format: "pattern" commands
    /// Returns None if parsing fails
    pub fn parse(input: &str, action_type: ActionType) -> Result<Self, String> {
        let input = input.trim_start();

        // Extract pattern (quoted or first word)
        let (pattern, rest) = if input.starts_with('"') {
            // Quoted pattern
            let end = input[1..]
                .find('"')
                .ok_or_else(|| format!("Incomplete action: missing closing quote in {}", input))?;
            let pattern = &input[1..=end];
            let rest = input[end + 2..].trim_start();
            (pattern, rest)
        } else {
            // Unquoted pattern (first word)
            let end = input.find(char::is_whitespace).unwrap_or(input.len());
            let pattern = &input[..end];
            let rest = input[end..].trim_start();
            (pattern, rest)
        };

        // For Replacement/Gag, commands can be empty
        if rest.is_empty() && action_type == ActionType::Trigger {
            return Err(format!("Missing action string for trigger: {}", input));
        }

        Ok(Self::new(pattern, rest, action_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_parse_quoted() {
        let action = Action::parse("\"^You hit\" say ouch!", ActionType::Trigger).unwrap();
        assert_eq!(action.pattern, "^You hit");
        assert_eq!(action.commands, "say ouch!");
        assert_eq!(action.action_type, ActionType::Trigger);
    }

    #[test]
    fn test_action_parse_unquoted() {
        let action = Action::parse("^prompt> look", ActionType::Trigger).unwrap();
        assert_eq!(action.pattern, "^prompt>");
        assert_eq!(action.commands, "look");
    }

    #[test]
    fn test_action_parse_replacement() {
        let action = Action::parse("\"stupid\" smart", ActionType::Replacement).unwrap();
        assert_eq!(action.pattern, "stupid");
        assert_eq!(action.commands, "smart");
        assert_eq!(action.action_type, ActionType::Replacement);
    }

    #[test]
    fn test_action_parse_gag() {
        let action = Action::parse("^spam.*message", ActionType::Gag).unwrap();
        assert_eq!(action.pattern, "^spam.*message");
        assert_eq!(action.commands, "");
    }

    #[test]
    fn test_action_parse_missing_quote() {
        let result = Action::parse("\"pattern commands", ActionType::Trigger);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing closing quote"));
    }

    #[test]
    fn test_action_parse_missing_commands() {
        let result = Action::parse("^pattern", ActionType::Trigger);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing action string"));
    }
}
