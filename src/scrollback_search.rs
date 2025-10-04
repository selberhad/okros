// ScrollbackSearch - Modal dialog for searching scrollback
//
// Ported from: mcl-cpp-reference/h/ScrollbackSearch.h + OutputWindow.cc:324-337
//
// C++ pattern: ScrollbackSearch extends InputBox
// Rust pattern: Function that creates InputBox with search callback

use crate::history::HistoryId;
use crate::input_box::InputBox;
use crate::output_window::OutputWindow;
use crate::window::Window;

/// Create scrollback search dialog (C++ ScrollbackSearch class)
///
/// C++ ScrollbackSearch.h:4 - Constructor takes parent window and forward flag
/// C++ OutputWindow.cc:324-337 - execute() implementation
pub fn create_scrollback_search(
    parent: *mut Window,
    output: *mut OutputWindow,
    forward: bool,
) -> InputBox {
    let prompt = if forward {
        "Regexp search forward in scrollback"
    } else {
        "Regexp search backwards in scrollback"
    };

    InputBox::new(
        parent,
        prompt,
        HistoryId::SearchScrollback, // C++ uses hi_search_scrollback
        Box::new(move |text: &str| {
            // C++ ScrollbackSearch::execute (OutputWindow.cc:324-337)
            if !text.is_empty() {
                unsafe {
                    if !output.is_null() {
                        // Call OutputWindow::search() (C++ line 333)
                        if let Some(message) = (*output).search(text, forward) {
                            // TODO: Display message in status bar
                            // For now, just log it
                            eprintln!("Search: {}", message);
                        }
                    }
                }
            }
            // Dialog closes automatically after execute (C++ line 336: die())
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn creates_search_dialog() {
        let root = Window::new(ptr::null_mut(), 80, 24);
        let mut ow = OutputWindow::new(root.as_ref() as *const _ as *mut _, 80, 20, 500, 0x07);

        let dialog = create_scrollback_search(
            root.as_ref() as *const _ as *mut _,
            &mut ow as *mut OutputWindow,
            false,
        );

        // Dialog should be created (can't easily test callback without integration)
        drop(dialog);
        drop(ow);
        drop(root);
    }
}
