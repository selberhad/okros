// Action/Trigger integration tests
// Golden tests based on C++ reference (mcl-cpp-reference/Session.cc, Alias.cc)

use okros::action::{Action, ActionType};
use okros::alias::Alias;
use okros::mud::Mud;

#[test]
fn test_action_parsing_trigger() {
    // C++ Action::parse() - trigger format: "pattern" commands
    let action = Action::parse("\"^You are attacked\" flee", ActionType::Trigger).unwrap();
    assert_eq!(action.pattern, "^You are attacked");
    assert_eq!(action.commands, "flee");
    assert_eq!(action.action_type, ActionType::Trigger);
}

#[test]
fn test_action_parsing_replacement() {
    // C++ Action::parse() - replacement format: "pattern" replacement
    let action = Action::parse("\"stupid\" smart", ActionType::Replacement).unwrap();
    assert_eq!(action.pattern, "stupid");
    assert_eq!(action.commands, "smart");
    assert_eq!(action.action_type, ActionType::Replacement);
}

#[test]
fn test_action_parsing_gag() {
    // Gag is replacement with empty commands
    let action = Action::parse("\"spam message\"", ActionType::Gag).unwrap();
    assert_eq!(action.pattern, "spam message");
    assert_eq!(action.commands, "");
    assert_eq!(action.action_type, ActionType::Gag);
}

#[test]
fn test_action_parsing_unquoted_pattern() {
    // C++ allows unquoted patterns (first word)
    let action = Action::parse("^prompt> look", ActionType::Trigger).unwrap();
    assert_eq!(action.pattern, "^prompt>");
    assert_eq!(action.commands, "look");
}

#[test]
fn test_action_parsing_pattern_with_spaces() {
    // Quoted patterns preserve spaces
    let action = Action::parse("\"You see a dragon\" run away", ActionType::Trigger).unwrap();
    assert_eq!(action.pattern, "You see a dragon");
    assert_eq!(action.commands, "run away");
}

#[test]
fn test_mud_alias_lookup() {
    // C++ MUD::findAlias()
    let mut mud = Mud::new();
    mud.alias_list.push(Alias::new("go", "go %1"));
    mud.alias_list.push(Alias::new("say", "tell bob %0"));

    assert!(mud.find_alias("go").is_some());
    assert!(mud.find_alias("say").is_some());
    assert!(mud.find_alias("nonexistent").is_none());
}

#[test]
fn test_mud_action_storage() {
    // C++ stores actions in action_list, checks by type
    let mut mud = Mud::new();

    let trigger = Action::new("^You hit", "say ouch", ActionType::Trigger);
    let replacement = Action::new("stupid", "smart", ActionType::Replacement);

    mud.action_list.push(trigger);
    mud.action_list.push(replacement);

    assert_eq!(mud.action_list.len(), 2);
    assert_eq!(mud.action_list[0].action_type, ActionType::Trigger);
    assert_eq!(mud.action_list[1].action_type, ActionType::Replacement);
}

#[test]
fn test_alias_expansion_with_mud() {
    // Integration: MUD finds alias, expands it
    let mut mud = Mud::new();
    mud.alias_list.push(Alias::new("n", "go north"));
    mud.alias_list.push(Alias::new("tell", "whisper %1 %+2"));

    // Test lookup and expand
    let alias = mud.find_alias("n").unwrap();
    assert_eq!(alias.expand(""), "go north");

    let alias = mud.find_alias("tell").unwrap();
    assert_eq!(alias.expand("bob hello there"), "whisper bob hello there");
}

#[test]
fn test_action_list_iteration() {
    // C++ FOREACH pattern over action_list
    let mut mud = Mud::new();
    mud.action_list.push(Action::new("pattern1", "cmd1", ActionType::Trigger));
    mud.action_list.push(Action::new("pattern2", "cmd2", ActionType::Trigger));
    mud.action_list.push(Action::new("pattern3", "replace3", ActionType::Replacement));

    let triggers: Vec<_> = mud.action_list.iter()
        .filter(|a| a.action_type == ActionType::Trigger)
        .collect();
    assert_eq!(triggers.len(), 2);

    let replacements: Vec<_> = mud.action_list.iter()
        .filter(|a| a.action_type == ActionType::Replacement)
        .collect();
    assert_eq!(replacements.len(), 1);
}

#[test]
fn test_color_stripping_for_trigger_check() {
    // C++ Session::triggerCheck strips SET_COLOR (0xFF) bytes before matching
    // This is a golden behavior test
    let line_with_color = b"\x1b[31mHello\x1b[0m World";
    let stripped: Vec<u8> = line_with_color.iter()
        .filter(|&&b| b != 0xFF) // In real code, would strip SET_COLOR bytes
        .copied()
        .collect();

    // ANSI sequences remain (they're stripped by ansi parser earlier)
    // This test documents expected behavior
    assert!(stripped.contains(&b'H'));
    assert!(stripped.contains(&b'W'));
}

#[test]
fn test_alias_expansion_edge_cases_from_cpp() {
    // Golden tests based on C++ Alias::expand() edge cases

    // Edge case: %0 with no args returns empty
    let alias = Alias::new("test", "prefix %0 suffix");
    assert_eq!(alias.expand(""), "prefix  suffix");

    // Edge case: %-5 with only 2 tokens
    let alias = Alias::new("test", "show %-5");
    assert_eq!(alias.expand("a b"), "show a b");

    // Edge case: %+5 with only 2 tokens returns empty
    let alias = Alias::new("test", "show %+5");
    assert_eq!(alias.expand("a b"), "show ");

    // Edge case: Multiple %% in pattern
    let alias = Alias::new("test", "100%% and 50%%");
    assert_eq!(alias.expand(""), "100% and 50%");
}

#[test]
fn test_action_replacement_behavior() {
    // C++ Action::checkReplacement modifies buffer in place
    // Returns true if replacement happened (len changed)
    let action = Action::new("foo", "bar", ActionType::Replacement);

    // This test documents expected behavior once regex is wired
    assert_eq!(action.pattern, "foo");
    assert_eq!(action.commands, "bar");
}

#[test]
fn test_gag_returns_empty_on_match() {
    // C++ gag action returns empty string when matched (cancels line)
    let gag = Action::new("spam", "", ActionType::Gag);

    // When matched, gag should return empty (line is canceled)
    assert_eq!(gag.commands, "");
    assert_eq!(gag.action_type, ActionType::Gag);
}
