// Toy 11: Plugins Stack

pub trait Interpreter {
    fn run(&mut self, function: &str, arg: &str, out: &mut String) -> bool;
    fn run_quietly(&mut self, function: &str, arg: &str, out: &mut String, _suppress_error: bool) -> bool {
        self.run(function, arg, out)
    }
    fn load_file(&mut self, _filename: &str, _suppress: bool) -> bool { false }
    fn eval(&mut self, _expr: &str, _out: &mut String) {}
    fn set_int(&mut self, _var: &str, _val: i64) {}
    fn set_str(&mut self, _var: &str, _val: &str) {}
    fn get_int(&mut self, _name: &str) -> i64 { 0 }
    fn get_str(&mut self, _name: &str) -> String { String::new() }
}

pub struct StackedInterpreter<I: Interpreter> {
    list: Vec<I>,
    disabled: Vec<String>,
}

impl<I: Interpreter> StackedInterpreter<I> {
    pub fn new() -> Self { Self{ list: Vec::new(), disabled: Vec::new() } }
    pub fn add(&mut self, i: I) { self.list.push(i); }
    pub fn disable(&mut self, fname: &str) { if !self.disabled.iter().any(|s| s==fname) { self.disabled.push(fname.to_string()); } }
    pub fn enable(&mut self, fname: &str) { self.disabled.retain(|s| s != fname); }
    pub fn is_enabled(&self, fname: &str) -> bool { !self.disabled.iter().any(|s| s==fname) }

    pub fn run(&mut self, function: &str, arg: &str, out: &mut String) -> bool {
        if !self.is_enabled(function) { return false; }
        let mut cur = arg.to_string();
        let mut any = false;
        for I in &mut self.list {
            let mut tmp = String::new();
            if I.run(function, &cur, &mut tmp) { cur = tmp; any = true; }
        }
        if any { *out = cur; }
        any
    }

    pub fn run_quietly(&mut self, function: &str, arg: &str, out: &mut String, suppress_error: bool) -> bool {
        if !self.is_enabled(function) { return false; }
        let mut cur = arg.to_string();
        let mut any = false;
        for I in &mut self.list {
            let mut tmp = String::new();
            if I.run_quietly(function, &cur, &mut tmp, suppress_error) { cur = tmp; any = true; }
        }
        if any { *out = cur; }
        any
    }

    pub fn set_int(&mut self, var: &str, val: i64) { for I in &mut self.list { I.set_int(var, val); } }
    pub fn set_str(&mut self, var: &str, val: &str) { for I in &mut self.list { I.set_str(var, val); } }
    pub fn get_int(&mut self, name: &str) -> i64 { self.list.first_mut().map(|i| i.get_int(name)).unwrap_or(0) }
    pub fn get_str(&mut self, name: &str) -> String { self.list.first_mut().map(|i| i.get_str(name)).unwrap_or_default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Mock { name: &'static str }
    impl Interpreter for Mock {
        fn run(&mut self, function: &str, arg: &str, out: &mut String) -> bool {
            if function == "sys/test" { *out = format!("{}[{}]", arg, self.name); true } else { false }
        }
        fn set_str(&mut self, _var: &str, _val: &str) {}
        fn get_str(&mut self, _name: &str) -> String { self.name.to_string() }
    }

    #[test]
    fn chains_in_order() {
        let mut st = StackedInterpreter::new();
        st.add(Mock{ name: "A"});
        st.add(Mock{ name: "B"});
        let mut out = String::new();
        let ok = st.run("sys/test", "in", &mut out);
        assert!(ok);
        assert_eq!(out, "in[A][B]");
    }

    #[test]
    fn disable_skips() {
        let mut st = StackedInterpreter::new();
        st.add(Mock{ name: "A"});
        st.add(Mock{ name: "B"});
        st.disable("sys/test");
        let mut out = String::new();
        let ok = st.run("sys/test", "in", &mut out);
        assert!(!ok);
        assert_eq!(out, "");
    }
}

