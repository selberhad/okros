// Macro - Keyboard shortcut bindings
//
// Ported from mcl-cpp-reference/h/Alias.h (Macro struct)

#[derive(Debug, Clone)]
pub struct Macro {
    pub key: i32,
    pub text: String,
}

impl Macro {
    pub fn new(key: i32, text: impl Into<String>) -> Self {
        Self {
            key,
            text: text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_creation() {
        let m = Macro::new(1, "north");
        assert_eq!(m.key, 1);
        assert_eq!(m.text, "north");
    }
}
