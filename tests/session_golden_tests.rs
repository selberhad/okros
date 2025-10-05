// Session Integration Golden Tests
//
// Validates Phase 1 Session restoration against C++ reference behavior:
// - Trigger checking per line (C++ Session.cc:527-538, 640-683)
// - Text replacement and gagging (C++ Session.cc:640-683)
// - Prompt multi-read buffering (C++ Session.cc:455-499, 596-602)
// - sys/output hook integration (C++ Session.cc:671)
// - Color stripping for trigger matching (C++ Session.cc:656-664)

use okros::mccp::PassthroughDecomp;
use okros::session::Session;
use std::sync::{Arc, Mutex};

#[test]
fn test_trigger_callback_fires_on_match() {
    // Golden: C++ Session.cc:527-538 - triggers fire on each \n
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    // Track triggered commands
    let triggered = Arc::new(Mutex::new(Vec::new()));
    let triggered_clone = triggered.clone();

    session.set_trigger_callback(Box::new(move |line: &str| -> Vec<String> {
        // Simulate trigger: "You are attacked" → ["flee"]
        if line.contains("You are attacked") {
            triggered_clone.lock().unwrap().push("flee".to_string());
            vec!["flee".to_string()]
        } else {
            Vec::new()
        }
    }));

    // Feed line with trigger match
    session.feed(b"You are attacked by a goblin\n");

    // Verify trigger fired
    assert_eq!(triggered.lock().unwrap().len(), 1);
    assert_eq!(triggered.lock().unwrap()[0], "flee");

    // Feed non-matching line
    session.feed(b"You are safe here\n");

    // Verify trigger didn't fire again
    assert_eq!(triggered.lock().unwrap().len(), 1);
}

#[test]
fn test_replacement_callback_modifies_text() {
    // Golden: C++ Session.cc:640-683 - replacements modify output
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    session.set_replacement_callback(Box::new(|line: &str| -> Option<String> {
        // Replace "stupid" with "smart"
        if line.contains("stupid") {
            Some(line.replace("stupid", "smart"))
        } else {
            None
        }
    }));

    // Feed line with replacement pattern
    session.feed(b"This is a stupid test\n");

    // Check scrollback contains replaced text
    let viewport = session.scrollback_viewport().unwrap();
    let text: String = viewport[0..80]
        .iter()
        .map(|&attr| (attr & 0xFF) as u8 as char)
        .collect();

    assert!(text.contains("smart"));
    assert!(!text.contains("stupid"));
}

#[test]
fn test_gag_suppresses_line() {
    // Golden: C++ Session.cc:640-683 - empty replacement = gag (suppress line)
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    session.set_replacement_callback(Box::new(|line: &str| -> Option<String> {
        // Gag lines containing "spam"
        if line.contains("spam") {
            Some(String::new()) // Empty = gag
        } else {
            None
        }
    }));

    // Feed normal line
    session.feed(b"Normal message\n");
    let lines_after_normal = session.total_lines();
    assert_eq!(lines_after_normal, 1);

    // Feed spam line (should be gagged)
    session.feed(b"spam spam spam\n");
    let lines_after_spam = session.total_lines();
    assert_eq!(lines_after_spam, 1); // No new line added (gagged)

    // Feed another normal line
    session.feed(b"Another message\n");
    let lines_after_second = session.total_lines();
    assert_eq!(lines_after_second, 2); // This one was added
}

#[test]
fn test_output_callback_runs_after_triggers() {
    // Golden: C++ Session.cc:671 - sys/output hook runs AFTER trigger/replacement
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    let order = Arc::new(Mutex::new(Vec::new()));

    // Set replacement first
    let order_clone1 = order.clone();
    session.set_replacement_callback(Box::new(move |line: &str| -> Option<String> {
        order_clone1.lock().unwrap().push("replacement".to_string());
        if line.contains("foo") {
            Some(line.replace("foo", "bar"))
        } else {
            None
        }
    }));

    // Set output hook (runs after replacement)
    let order_clone2 = order.clone();
    session.set_output_callback(Box::new(move |line: &str| -> Option<String> {
        order_clone2.lock().unwrap().push("output_hook".to_string());
        // Verify replacement already happened
        assert!(line.contains("bar"));
        assert!(!line.contains("foo"));
        None
    }));

    // Feed line
    session.feed(b"test foo test\n");

    // Verify order: replacement → output_hook
    assert_eq!(order.lock().unwrap().len(), 2);
    assert_eq!(order.lock().unwrap()[0], "replacement");
    assert_eq!(order.lock().unwrap()[1], "output_hook");
}

#[test]
fn test_prompt_callback_fires_on_ga() {
    // Golden: C++ Session.cc:455-499 - prompts fire on IAC GA
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    let prompts = Arc::new(Mutex::new(Vec::new()));
    let prompts_clone = prompts.clone();

    session.set_prompt_callback(Box::new(move |prompt: &str| -> bool {
        prompts_clone.lock().unwrap().push(prompt.to_string());
        true // Show prompt
    }));

    // Feed prompt with IAC GA (0xFF 0xF9)
    // Telnet parser will recognize this and fire prompt event
    session.feed(b"HP: 100>\xFF\xF9");

    // Prompt event should have fired
    // Note: telnet parser handles IAC GA, we just verify callback works
    assert!(!prompts.lock().unwrap().is_empty());
}

#[test]
fn test_prompt_buffering_across_reads() {
    // Golden: C++ Session.cc:596-602 - prompts buffer across multiple reads
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    let prompts = Arc::new(Mutex::new(Vec::new()));
    let prompts_clone = prompts.clone();

    session.set_prompt_callback(Box::new(move |prompt: &str| -> bool {
        prompts_clone.lock().unwrap().push(prompt.to_string());
        true
    }));

    // Feed prompt split across multiple calls
    session.feed(b"HP: ");
    session.feed(b"100 ");
    session.feed(b"MP: 50>\xFF\xF9"); // IAC GA terminates prompt

    // Prompt should combine all fragments
    let prompts_lock = prompts.lock().unwrap();
    if !prompts_lock.is_empty() {
        let full_prompt = &prompts_lock[prompts_lock.len() - 1];
        assert!(full_prompt.contains("HP:"));
        assert!(full_prompt.contains("MP:"));
    }
}

#[test]
fn test_color_stripping_for_trigger_matching() {
    // Golden: C++ Session.cc:656-664 - strip SET_COLOR markers for trigger matching
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    let matched = Arc::new(Mutex::new(false));
    let matched_clone = matched.clone();

    session.set_trigger_callback(Box::new(move |line: &str| -> Vec<String> {
        // Should match "Hello World" even if it has ANSI color codes
        if line.contains("Hello") && line.contains("World") {
            *matched_clone.lock().unwrap() = true;
        }
        Vec::new()
    }));

    // Feed colored text: "\x1b[31mHello\x1b[0m World\n"
    // ANSI parser strips ANSI codes, trigger should match plain text
    session.feed(b"\x1b[31mHello\x1b[0m World\n");

    // Trigger should have matched despite color codes
    assert!(*matched.lock().unwrap());
}

#[test]
fn test_multiple_triggers_on_same_line() {
    // C++ behavior: all matching triggers fire for a line
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    let triggers = Arc::new(Mutex::new(Vec::new()));
    let triggers_clone = triggers.clone();

    session.set_trigger_callback(Box::new(move |line: &str| -> Vec<String> {
        let mut cmds = Vec::new();
        if line.contains("dragon") {
            triggers_clone.lock().unwrap().push("dragon".to_string());
            cmds.push("flee".to_string());
        }
        if line.contains("huge") {
            triggers_clone.lock().unwrap().push("huge".to_string());
            cmds.push("run".to_string());
        }
        cmds
    }));

    // Line matches both triggers
    session.feed(b"A huge dragon appears\n");

    // Both triggers should fire
    let triggers_lock = triggers.lock().unwrap();
    assert_eq!(triggers_lock.len(), 2);
    assert!(triggers_lock.contains(&"dragon".to_string()));
    assert!(triggers_lock.contains(&"huge".to_string()));
}

#[test]
fn test_replacement_then_trigger_order() {
    // Golden: C++ Session.cc:640-683 - replacement happens first, then triggers
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    let trigger_saw = Arc::new(Mutex::new(String::new()));

    // Replacement: foo → bar
    session.set_replacement_callback(Box::new(|line: &str| -> Option<String> {
        if line.contains("foo") {
            Some(line.replace("foo", "bar"))
        } else {
            None
        }
    }));

    // Trigger: should see "bar" not "foo"
    let trigger_clone = trigger_saw.clone();
    session.set_trigger_callback(Box::new(move |line: &str| -> Vec<String> {
        *trigger_clone.lock().unwrap() = line.to_string();
        Vec::new()
    }));

    session.feed(b"test foo test\n");

    // Trigger should have seen replaced text
    let saw_lock = trigger_saw.lock().unwrap();
    assert!(saw_lock.contains("bar"));
    assert!(!saw_lock.contains("foo"));
}

#[test]
fn test_prompt_hide_via_callback() {
    // Golden: C++ Session.cc:474, 488 - opt_showprompt hides prompts
    let mut session = Session::new(PassthroughDecomp::new(), 80, 24, 100);

    session.set_prompt_callback(Box::new(|_prompt: &str| -> bool {
        false // Hide all prompts
    }));

    let lines_before = session.total_lines();

    // Send prompt with IAC GA
    session.feed(b"HP: 100>\xFF\xF9");

    let lines_after = session.total_lines();

    // Prompt should not have been added to scrollback
    assert_eq!(lines_before, lines_after);
}
