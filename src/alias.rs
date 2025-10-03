// Alias - Command text expansion with parameter substitution
//
// Ported from mcl-cpp-reference/Alias.cc

const MAX_ALIAS_BUFFER: usize = 1024;

#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub text: String,
}

impl Alias {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: text.into(),
        }
    }

    /// Expand alias text with argument substitution
    /// - %N: single token N (e.g., %1, %2)
    /// - %-N: tokens 1 through N (e.g., %-2 = tokens 1 and 2)
    /// - %+N: tokens N through end (e.g., %+2 = token 2 to last)
    /// - %%: literal %
    pub fn expand(&self, arg: &str) -> String {
        let mut result = String::with_capacity(MAX_ALIAS_BUFFER);
        let mut chars = self.text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(&next) = chars.peek() {
                    chars.next(); // consume peeked char

                    // Check for %-N (range from 1 to N)
                    if next == '-' {
                        if let Some(&digit_ch) = chars.peek() {
                            if digit_ch.is_ascii_digit() {
                                chars.next();
                                let n = digit_ch.to_digit(10).unwrap() as usize;
                                if let Some(tokens) = self.print_tokens(arg, 1, n) {
                                    result.push_str(&tokens);
                                }
                                continue;
                            }
                        }
                        // Not a valid pattern, output as-is
                        result.push('%');
                        result.push('-');
                        continue;
                    }

                    // Check for %+N (range from N to end)
                    if next == '+' {
                        if let Some(&digit_ch) = chars.peek() {
                            if digit_ch.is_ascii_digit() {
                                chars.next();
                                let n = digit_ch.to_digit(10).unwrap() as usize;
                                if let Some(tokens) = self.print_tokens(arg, n, usize::MAX) {
                                    result.push_str(&tokens);
                                }
                                continue;
                            }
                        }
                        // Not a valid pattern, output as-is
                        result.push('%');
                        result.push('+');
                        continue;
                    }

                    // Check for %N (single token)
                    if next.is_ascii_digit() {
                        let n = next.to_digit(10).unwrap() as usize;
                        if let Some(tokens) = self.print_tokens(arg, n, n) {
                            result.push_str(&tokens);
                        }
                        continue;
                    }

                    // Check for %% (literal %)
                    if next == '%' {
                        result.push('%');
                        continue;
                    }

                    // Unknown pattern - leave it alone (C++ behavior: output %%%c)
                    result.push('%');
                    result.push(next);
                } else {
                    // % at end of string
                    result.push('%');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Find where the count-th token starts (1-indexed)
    fn find_token<'a>(&self, arg: &'a str, count: usize) -> Option<&'a str> {
        let mut current = 1;
        let mut rest = arg;

        loop {
            // Skip leading whitespace
            rest = rest.trim_start();

            if rest.is_empty() {
                return None;
            }

            if count == current {
                return Some(rest);
            }

            // Skip to next whitespace
            if let Some(pos) = rest.find(char::is_whitespace) {
                rest = &rest[pos..];
                current += 1;
            } else {
                // Last token
                return None;
            }
        }
    }

    /// Extract tokens from begin to end (inclusive, 1-indexed)
    /// Special: if begin == 0, return entire arg
    fn print_tokens(&self, arg: &str, begin: usize, end: usize) -> Option<String> {
        if begin == 0 {
            return Some(arg.to_string());
        }

        let begin_str = self.find_token(arg, begin)?;

        // Find where 'end+1' token starts (or end of string)
        let end_str = if end == usize::MAX {
            // %+N case - go to end
            ""
        } else {
            self.find_token(arg, end + 1).unwrap_or("")
        };

        // Calculate the slice
        if end_str.is_empty() {
            // Take from begin to end of arg
            Some(begin_str.to_string())
        } else {
            // Calculate offset from original arg
            let begin_offset = begin_str.as_ptr() as usize - arg.as_ptr() as usize;
            let end_offset = end_str.as_ptr() as usize - arg.as_ptr() as usize;
            let slice = &arg[begin_offset..end_offset].trim_end();
            Some(slice.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_basic() {
        let alias = Alias::new("test", "hello");
        assert_eq!(alias.expand("world"), "hello");
    }

    #[test]
    fn test_alias_single_token() {
        let alias = Alias::new("say", "tell bob %1");
        assert_eq!(alias.expand("hello world"), "tell bob hello");

        let alias = Alias::new("say", "tell %1 %2");
        assert_eq!(alias.expand("bob hello"), "tell bob hello");
    }

    #[test]
    fn test_alias_range_from_start() {
        let alias = Alias::new("shout", "yell %-2");
        assert_eq!(alias.expand("hello world foo"), "yell hello world");
    }

    #[test]
    fn test_alias_range_to_end() {
        let alias = Alias::new("cmd", "do %+2");
        assert_eq!(alias.expand("one two three four"), "do two three four");
    }

    #[test]
    fn test_alias_literal_percent() {
        let alias = Alias::new("test", "give 100%% effort");
        assert_eq!(alias.expand(""), "give 100% effort");
    }

    #[test]
    fn test_alias_unknown_pattern() {
        let alias = Alias::new("test", "value: %x");
        assert_eq!(alias.expand(""), "value: %x");
    }

    #[test]
    fn test_alias_all_args() {
        let alias = Alias::new("echo", "%0");
        assert_eq!(alias.expand("hello world"), "hello world");
    }

    #[test]
    fn test_alias_missing_token() {
        let alias = Alias::new("test", "got %3");
        assert_eq!(alias.expand("one two"), "got "); // Token 3 doesn't exist
    }
}
