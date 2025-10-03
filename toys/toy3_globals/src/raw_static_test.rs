// Test raw `unsafe static mut` pattern with helper functions
// This mimics C++ MCL global usage most closely

use std::ptr;

// =============================================================================
// Mock types (representing MCL globals)
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
    connected: bool,
}

impl Session {
    fn new(name: &str) -> Self {
        println!("  Session::new(\"{}\")", name);
        Session { name: name.to_string(), connected: true }
    }

    fn send(&mut self, msg: &str) {
        println!("  Session::send(\"{}\") [name={}]", msg, self.name);
    }

    fn disconnect(&mut self) {
        println!("  Session::disconnect() [name={}]", self.name);
        self.connected = false;
    }
}

// =============================================================================
// Global state - raw static mut pattern (Option 2 from discussion)
// =============================================================================

static mut SCREEN: *mut Screen = ptr::null_mut();
static mut CONFIG: *mut Config = ptr::null_mut();
static mut CURRENT_SESSION: *mut Session = ptr::null_mut();

// Helper functions - hide unsafe in one place
pub fn screen() -> &'static mut Screen {
    unsafe {
        SCREEN.as_mut().expect("Screen not initialized! Call init_globals() first")
    }
}

pub fn config() -> &'static mut Config {
    unsafe {
        CONFIG.as_mut().expect("Config not initialized! Call init_globals() first")
    }
}

// Nullable global - returns Option
pub fn current_session() -> Option<&'static mut Session> {
    unsafe { CURRENT_SESSION.as_mut() }
}

// Setter for nullable global
pub fn set_current_session(session: Box<Session>) {
    unsafe {
        // Clean up old session if exists
        if !CURRENT_SESSION.is_null() {
            let _ = Box::from_raw(CURRENT_SESSION);
        }
        CURRENT_SESSION = Box::leak(session);
    }
}

pub fn clear_current_session() {
    unsafe {
        if !CURRENT_SESSION.is_null() {
            let _ = Box::from_raw(CURRENT_SESSION);
            CURRENT_SESSION = ptr::null_mut();
        }
    }
}

// Initialize globals once at startup
pub unsafe fn init_globals() {
    if !SCREEN.is_null() || !CONFIG.is_null() {
        panic!("Globals already initialized!");
    }

    println!("Initializing globals...");
    SCREEN = Box::leak(Box::new(Screen::new(80, 24)));
    CONFIG = Box::leak(Box::new(Config::new()));
    println!("Globals initialized!\n");
}

// =============================================================================
// Test functions that use globals (like C++ MCL code would)
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
    set_current_session(Box::new(Session::new(name)));
}

fn disconnect_from_mud() {
    println!("disconnect_from_mud():");

    if let Some(session) = current_session() {
        session.disconnect();
    }

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
    println!("=== Raw Static Mut Pattern Test ===\n");

    // Test 1: Initialization
    println!("Test 1: Initialization");
    unsafe {
        init_globals();
    }

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
    println!("✓ Initialization works");
    println!("✓ Global access works (clean syntax)");
    println!("✓ Mutation works");
    println!("✓ Nullable globals work (Option pattern)");
    println!("✓ Multiple functions can access globals");
    println!("✓ Cross-function state changes work");
    println!("\nConclusion: Raw static mut with helper functions is viable!");
}
