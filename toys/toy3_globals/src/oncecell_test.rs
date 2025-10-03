// Test OnceCell + UnsafeCell pattern for comparison
// Slightly safer (prevents double-init) but more boilerplate

use once_cell::sync::OnceCell;
use std::cell::UnsafeCell;

// =============================================================================
// Mock types (same as raw_static_test.rs)
// =============================================================================

struct Screen {
    width: usize,
    height: usize,
    refresh_count: usize,
}

impl Screen {
    fn new(width: usize, height: usize) -> Self {
        println!("  Screen::new({}, {})", width, height);
        Screen { width, height, refresh_count: 0 }
    }

    fn refresh(&mut self) {
        self.refresh_count += 1;
        println!("  Screen::refresh() [count={}]", self.refresh_count);
    }

    fn resize(&mut self, w: usize, h: usize) {
        println!("  Screen::resize({}, {})", w, h);
        self.width = w;
        self.height = h;
    }
}

struct Config {
    beep_enabled: bool,
    tab_size: usize,
}

impl Config {
    fn new() -> Self {
        println!("  Config::new()");
        Config { beep_enabled: true, tab_size: 4 }
    }

    fn get_beep(&self) -> bool {
        self.beep_enabled
    }

    fn set_tab_size(&mut self, size: usize) {
        println!("  Config::set_tab_size({})", size);
        self.tab_size = size;
    }
}

struct Session {
    name: String,
}

impl Session {
    fn new(name: &str) -> Self {
        println!("  Session::new(\"{}\")", name);
        Session { name: name.to_string() }
    }

    fn send(&mut self, msg: &str) {
        println!("  Session::send(\"{}\") [name={}]", msg, self.name);
    }
}

// =============================================================================
// Global state - OnceCell + UnsafeCell pattern
// =============================================================================

static SCREEN: OnceCell<UnsafeCell<Screen>> = OnceCell::new();
static CONFIG: OnceCell<UnsafeCell<Config>> = OnceCell::new();
static CURRENT_SESSION: OnceCell<UnsafeCell<Option<Session>>> = OnceCell::new();

// Helper functions
pub fn screen() -> &'static mut Screen {
    let cell = SCREEN.get().expect("Screen not initialized! Call init_globals() first");
    unsafe { &mut *cell.get() }
}

pub fn config() -> &'static mut Config {
    let cell = CONFIG.get().expect("Config not initialized! Call init_globals() first");
    unsafe { &mut *cell.get() }
}

pub fn current_session() -> Option<&'static mut Session> {
    let cell = CURRENT_SESSION.get().expect("Globals not initialized!");
    unsafe { (*cell.get()).as_mut() }
}

pub fn set_current_session(session: Session) {
    let cell = CURRENT_SESSION.get().expect("Globals not initialized!");
    unsafe {
        *cell.get() = Some(session);
    }
}

pub fn clear_current_session() {
    let cell = CURRENT_SESSION.get().expect("Globals not initialized!");
    unsafe {
        *cell.get() = None;
    }
}

// Initialize globals once at startup
pub fn init_globals() {
    println!("Initializing globals...");

    SCREEN.set(UnsafeCell::new(Screen::new(80, 24)))
        .expect("Screen already initialized!");

    CONFIG.set(UnsafeCell::new(Config::new()))
        .expect("Config already initialized!");

    CURRENT_SESSION.set(UnsafeCell::new(None))
        .expect("Session already initialized!");

    println!("Globals initialized!\n");
}

// =============================================================================
// Test functions (same as raw_static_test.rs)
// =============================================================================

fn render_frame() {
    println!("render_frame():");
    screen().refresh();

    if config().get_beep() {
        println!("  *BEEP*");
    }
}

fn handle_resize(w: usize, h: usize) {
    println!("handle_resize({}, {}):", w, h);
    screen().resize(w, h);
}

fn process_input(input: &str) {
    println!("process_input(\"{}\"):", input);

    if let Some(session) = current_session() {
        session.send(input);
    } else {
        println!("  No active session!");
    }
}

fn connect_to_mud(name: &str) {
    println!("connect_to_mud(\"{}\"):", name);
    set_current_session(Session::new(name));
}

fn disconnect_from_mud() {
    println!("disconnect_from_mud():");
    clear_current_session();
}

fn modify_config() {
    println!("modify_config():");
    config().set_tab_size(8);
}

// =============================================================================
// Main test
// =============================================================================

fn main() {
    println!("=== OnceCell + UnsafeCell Pattern Test ===\n");

    // Test 1: Initialization
    println!("Test 1: Initialization");
    init_globals();

    // Test 2: Basic access
    println!("Test 2: Basic global access");
    render_frame();
    println!();

    // Test 3: Mutation
    println!("Test 3: Global mutation");
    handle_resize(100, 30);
    modify_config();
    println!();

    // Test 4: Nullable global (no session yet)
    println!("Test 4: Nullable global (no session)");
    process_input("hello");
    println!();

    // Test 5: Create nullable global
    println!("Test 5: Create session");
    connect_to_mud("example.com");
    println!();

    // Test 6: Use nullable global
    println!("Test 6: Use session");
    process_input("look");
    process_input("north");
    println!();

    // Test 7: Multiple screen refreshes
    println!("Test 7: Multiple operations");
    render_frame();
    render_frame();
    println!();

    // Test 8: Clear nullable global
    println!("Test 8: Disconnect");
    disconnect_from_mud();
    process_input("hello again");
    println!();

    // Test 9: Reconnect
    println!("Test 9: Reconnect");
    connect_to_mud("another-mud.com");
    process_input("west");
    println!();

    println!("=== Results ===");
    println!("✓ Initialization works (with double-init protection)");
    println!("✓ Global access works (clean syntax)");
    println!("✓ Mutation works");
    println!("✓ Nullable globals work (Option pattern)");
    println!("✓ Multiple functions can access globals");
    println!("\nConclusion: OnceCell pattern works but adds boilerplate");
    println!("Comparison: Safer init, but helper functions are more complex");
}
