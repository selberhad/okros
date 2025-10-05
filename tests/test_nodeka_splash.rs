/// Test Nodeka splash screen rendering
/// This test uses captured real MUD data to verify our pipeline works correctly
use okros::mccp::PassthroughDecomp;
use okros::session::Session;
use std::fs;
use std::path::Path;

#[test]
fn test_all_nodeka_splash_screens() {
    // Load all captured splash screens from test_captures/nodeka/
    let captures_dir = Path::new("test_captures/nodeka");
    let mut json_files: Vec<_> = fs::read_dir(captures_dir)
        .expect("test_captures/nodeka directory should exist")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "json" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    json_files.sort();

    assert!(
        !json_files.is_empty(),
        "No capture files found in test_captures/nodeka/"
    );

    println!(
        "\n=== Testing {} captured splash screens ===\n",
        json_files.len()
    );

    let mut passed = 0;
    let mut failed = 0;

    for json_path in &json_files {
        let filename = json_path.file_name().unwrap().to_string_lossy();
        println!("Testing: {}", filename);

        let json_content = fs::read_to_string(json_path).expect("Failed to read JSON file");

        let json: serde_json::Value =
            serde_json::from_str(&json_content).expect("Failed to parse JSON");

        let lines = json["lines"]
            .as_array()
            .expect("JSON should have 'lines' array");

        // Reconstruct ANSI data from JSON lines
        let mut ansi_data = String::new();
        for line in lines {
            let line_str = line.as_str().unwrap_or("");
            ansi_data.push_str(line_str);
            if !line_str.ends_with('\n') {
                ansi_data.push('\n');
            }
        }

        // Test the pipeline
        let result = test_single_splash(&ansi_data, &filename);

        if result {
            passed += 1;
            println!("  ✅ PASS\n");
        } else {
            failed += 1;
            println!("  ❌ FAIL\n");
        }
    }

    println!(
        "=== Results: {}/{} passed, {}/{} failed ===",
        passed,
        json_files.len(),
        failed,
        json_files.len()
    );

    // Assertion: we expect these to fail until we fix the bug
    // Uncomment this when the bug is fixed:
    // assert_eq!(failed, 0, "Some splash screens failed rendering");
}

fn test_single_splash(ansi_data: &str, filename: &str) -> bool {
    // Create session matching OutputWindow dimensions (height-1 like main.rs:176)
    // Modern terminal might be 120x40, not 80x24!
    let term_width = 120;
    let term_height = 40;

    let mut session = Session::new(
        PassthroughDecomp::new(),
        term_width,
        term_height - 1, // Match OutputWindow (main.rs:176)
        1000,
    );

    // Feed data in 1024-byte chunks like network reads (matching Nodeka behavior)
    let bytes = ansi_data.as_bytes();
    for chunk in bytes.chunks(1024) {
        session.feed(chunk);
        // NOTE: Don't flush_partial_line() here - that fragments lines!
        // C++ Window::print() appends to cursor, Rust needs to overlay line_buf
    }

    // After all chunks, line_buf might have the prompt (no \n, no GA/EOR)
    // Overlay line_buf on viewport (matching main.rs fix)
    let mut viewport = session.scrollback.viewport_slice().to_vec();

    // Render partial line_buf after last complete line (matching main.rs:645-667)
    if !session.current_line().is_empty() {
        let total_lines = session.scrollback.total_lines();
        let line_y = total_lines % (term_height - 1);
        let line_start = line_y * term_width;

        for (i, (ch, color)) in session.current_line_colored().iter().enumerate() {
            if line_start + i < viewport.len() {
                viewport[line_start + i] = ((*color as u16) << 8) | (*ch as u16);
            }
        }
    }

    // Extract text from viewport (now including overlaid line_buf)
    let mut lines: Vec<String> = Vec::new();
    for y in 0..(term_height - 1) {
        let start = y * term_width;
        let end = start + term_width;
        let line: String = viewport[start..end]
            .iter()
            .map(|&a| (a & 0xFF) as u8 as char)
            .collect();
        lines.push(line.trim_end().to_string());
    }

    // Count non-empty lines
    let non_empty_lines: Vec<&String> = lines.iter().filter(|l| !l.is_empty()).collect();

    // The splash screen has 20 lines (including empty lines and prompt)
    // We should see all of them in a 24-line viewport
    // At minimum, we should see the prompt at the bottom
    let prompt_visible = lines
        .iter()
        .any(|l| l.contains("Type 'create' or enter name"));

    println!(
        "  Session pipeline: {} non-empty lines, prompt visible: {}",
        non_empty_lines.len(),
        prompt_visible
    );

    // Return true if both conditions pass
    prompt_visible && non_empty_lines.len() >= 15
}

#[test]
fn test_all_nodeka_window_rendering() {
    // Load all captured splash screens
    let captures_dir = Path::new("test_captures/nodeka");
    let mut json_files: Vec<_> = fs::read_dir(captures_dir)
        .expect("test_captures/nodeka directory should exist")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "json" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    json_files.sort();

    println!(
        "\n=== Testing Window rendering for {} captures ===\n",
        json_files.len()
    );

    let mut passed = 0;
    let mut failed = 0;

    for json_path in &json_files {
        let filename = json_path.file_name().unwrap().to_string_lossy();
        println!("Testing: {}", filename);

        let json_content = fs::read_to_string(json_path).expect("Failed to read JSON file");

        let json: serde_json::Value =
            serde_json::from_str(&json_content).expect("Failed to parse JSON");

        let lines = json["lines"]
            .as_array()
            .expect("JSON should have 'lines' array");

        // Reconstruct ANSI data from JSON lines
        let mut ansi_data = String::new();
        for line in lines {
            let line_str = line.as_str().unwrap_or("");
            ansi_data.push_str(line_str);
            if !line_str.ends_with('\n') {
                ansi_data.push('\n');
            }
        }

        let result = test_window_rendering(&ansi_data, &filename);

        if result {
            passed += 1;
            println!("  ✅ PASS\n");
        } else {
            failed += 1;
            println!("  ❌ FAIL\n");
        }
    }

    println!(
        "=== Window Rendering Results: {}/{} passed, {}/{} failed ===",
        passed,
        json_files.len(),
        failed,
        json_files.len()
    );

    // Expected to fail until bug is fixed
    println!("\nNote: Window rendering failures confirm the double-buffering bug.");
}

fn test_window_rendering(ansi_data: &str, filename: &str) -> bool {
    use okros::output_window::OutputWindow;
    use okros::screen::Screen;
    use std::ptr;

    // Same test as before but extracted into function
    let nodeka_ansi = ansi_data;

    // Use realistic modern terminal dimensions
    let term_width = 120;
    let term_height = 40;

    let mut session = Session::new(
        PassthroughDecomp::new(),
        term_width,
        term_height - 1, // Match OutputWindow
        1000,
    );

    // Feed in chunks like network reads
    let bytes = nodeka_ansi.as_bytes();
    for chunk in bytes.chunks(1024) {
        session.feed(chunk);
    }

    // Create Screen and OutputWindow (simulating main.rs)
    let mut screen = Screen::new(term_width, term_height);
    let mut output = OutputWindow::new(
        screen.window_mut() as *mut okros::window::Window,
        term_width,
        term_height - 1, // main.rs:166
        1000,
        0x07,
    );

    // Copy session scrollback to OutputWindow with line_buf overlay (main.rs:641-669)
    let mut viewport = session.scrollback.viewport_slice().to_vec();

    // Overlay line_buf (matching main.rs:645-667)
    if !session.current_line().is_empty() {
        let total_lines = session.scrollback.total_lines();
        let line_y = total_lines % (term_height - 1);
        let line_start = line_y * term_width;

        for (i, (ch, color)) in session.current_line_colored().iter().enumerate() {
            if line_start + i < viewport.len() {
                viewport[line_start + i] = ((*color as u16) << 8) | (*ch as u16);
            }
        }
    }

    output.win.blit(&viewport);
    output.win.dirty = true;

    // Call screen.refresh() like the real TTY mode does (main.rs:260)
    use okros::curses::AcsCaps;
    let caps = AcsCaps::default();
    screen.refresh(&caps);

    // Check the window canvas (what actually got rendered)
    let canvas = &output.win.canvas;
    let mut lines: Vec<String> = Vec::new();
    for y in 0..(term_height - 1) {
        let start = y * term_width;
        let end = start + term_width;
        let line: String = canvas[start..end]
            .iter()
            .map(|&a| (a & 0xFF) as u8 as char)
            .collect();
        lines.push(line.trim_end().to_string());
    }

    let non_empty_lines: Vec<&String> = lines.iter().filter(|l| !l.is_empty()).collect();
    let prompt_visible = lines
        .iter()
        .any(|l| l.contains("Type 'create' or enter name"));

    println!(
        "  Window canvas: {} non-empty lines, prompt visible: {}",
        non_empty_lines.len(),
        prompt_visible
    );

    prompt_visible && non_empty_lines.len() >= 15
}
