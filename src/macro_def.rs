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

    #[test]
    fn test_macro_multiple_keys() {
        let m1 = Macro::new(1, "north");
        let m2 = Macro::new(2, "south");
        let m3 = Macro::new(3, "look");

        assert_eq!(m1.key, 1);
        assert_eq!(m2.key, 2);
        assert_eq!(m3.key, 3);
    }

    #[test]
    fn test_macro_empty_text() {
        let m = Macro::new(5, "");
        assert_eq!(m.key, 5);
        assert_eq!(m.text, "");
    }

    #[test]
    fn test_macro_multiline_text() {
        let m = Macro::new(10, "north\nsouth\nlook");
        assert_eq!(m.text, "north\nsouth\nlook");
    }
}
