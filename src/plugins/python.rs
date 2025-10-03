//! Python interpreter plugin (feature-gated)
//!
//! Ported from: plugins/PythonEmbeddedInterpreter.cc
//! Uses pyo3 for Python C API abstraction (simpler than raw C API)

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use crate::plugins::stack::Interpreter;

/// Python interpreter wrapper matching C++ PythonEmbeddedInterpreter patterns
pub struct PythonInterpreter {
    globals: Py<PyDict>,
}

impl PythonInterpreter {
    /// Initialize Python interpreter and set up globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:23-32):
    /// ```cpp
    /// Py_Initialize();
    /// module = PyImport_AddModule("__main__");
    /// globals = PyModule_GetDict(module);
    /// Py_INCREF(globals);
    /// ```
    pub fn new() -> PyResult<Self> {
        Python::with_gil(|py| {
            // Get __main__ module
            let main_module = PyModule::import_bound(py, "__main__")?;

            // Get globals dict from __main__
            let globals = main_module.dict();

            // Store globals (pyo3 handles refcounting automatically)
            let globals = globals.clone().unbind();

            Ok(PythonInterpreter { globals })
        })
    }

    /// Internal eval helper
    fn eval_internal(&mut self, expression: &str) -> PyResult<()> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            py.run_bound(expression, Some(&globals), Some(&globals))?;
            Ok(())
        })
    }

    /// Internal load_file helper
    fn load_file_internal(&mut self, path: &str) -> PyResult<()> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);

            // Read file contents
            let code = std::fs::read_to_string(path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
                    format!("Failed to read file: {}", e)
                ))?;

            // Run file contents with globals
            py.run_bound(&code, Some(&globals), Some(&globals))?;

            Ok(())
        })
    }

    /// Call Python function with string arg, return result as string
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:93-130):
    /// ```cpp
    /// PyObject *func = get_function(function);
    /// PyObject *func_args = Py_BuildValue("(s)", arg);
    /// PyObject *res = PyEval_CallObject(func, func_args);
    /// ```
    fn call_function_internal(&mut self, function_name: &str, arg: &str) -> PyResult<String> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);

            // Get function from globals
            let func = globals.get_item(function_name)?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                    format!("Function '{}' not found", function_name)
                ))?;

            // Call function with arg
            let result = func.call1((arg,))?;

            // Extract string result
            result.extract::<String>()
        })
    }
}

impl Interpreter for PythonInterpreter {
    /// Run Python function with arg, return result in out
    fn run(&mut self, function: &str, arg: &str, out: &mut String) -> bool {
        match self.call_function_internal(function, arg) {
            Ok(result) => {
                *out = result;
                true
            }
            Err(_) => false,
        }
    }

    /// Run quietly - suppress Python errors if requested
    fn run_quietly(&mut self, function: &str, arg: &str, out: &mut String, suppress_error: bool) -> bool {
        match self.call_function_internal(function, arg) {
            Ok(result) => {
                *out = result;
                true
            }
            Err(e) => {
                if !suppress_error {
                    Python::with_gil(|py| {
                        e.print(py);
                    });
                }
                false
            }
        }
    }

    /// Load Python file
    fn load_file(&mut self, filename: &str, suppress: bool) -> bool {
        match self.load_file_internal(filename) {
            Ok(_) => true,
            Err(e) => {
                if !suppress {
                    Python::with_gil(|py| {
                        e.print(py);
                    });
                }
                false
            }
        }
    }

    /// Eval Python expression
    fn eval(&mut self, expr: &str, out: &mut String) {
        if let Err(e) = self.eval_internal(expr) {
            Python::with_gil(|py| {
                e.print(py);
            });
            *out = String::new();
        }
    }

    /// Set integer variable in Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:206-216):
    /// ```cpp
    /// PyObject *obj = Py_BuildValue("i", value);
    /// PyDict_SetItemString(globals, name, obj);
    /// ```
    fn set_int(&mut self, var: &str, val: i64) {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            let _ = globals.set_item(var, val);
        });
    }

    /// Set string variable in Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:218-228):
    /// ```cpp
    /// PyObject *obj = Py_BuildValue("s", value);
    /// PyDict_SetItemString(globals, name, obj);
    /// ```
    fn set_str(&mut self, var: &str, val: &str) {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            let _ = globals.set_item(var, val);
        });
    }

    /// Get integer variable from Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:230-241):
    /// ```cpp
    /// PyObject *obj = PyDict_GetItemString(globals, name);
    /// PyArg_Parse(obj, "i", &i);
    /// ```
    fn get_int(&mut self, name: &str) -> i64 {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            globals.get_item(name)
                .ok()
                .and_then(|v| v)
                .and_then(|v| v.extract::<i64>().ok())
                .unwrap_or(0)
        })
    }

    /// Get string variable from Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:243-254):
    /// ```cpp
    /// PyObject *obj = PyDict_GetItemString(globals, name);
    /// PyArg_Parse(obj, "s", &str);
    /// ```
    fn get_str(&mut self, name: &str) -> String {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            globals.get_item(name)
                .ok()
                .and_then(|v| v)
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let _interp = PythonInterpreter::new().unwrap();
        // If we got here, initialization worked
    }

    #[test]
    fn test_set_get_string() {
        let mut interp = PythonInterpreter::new().unwrap();
        interp.set_str("test_var", "Hello from Rust!");

        let value = interp.get_str("test_var");
        assert_eq!(value, "Hello from Rust!");
    }

    #[test]
    fn test_set_get_int() {
        let mut interp = PythonInterpreter::new().unwrap();
        interp.set_int("num", 123);

        let value = interp.get_int("num");
        assert_eq!(value, 123);
    }

    #[test]
    fn test_eval() {
        let mut interp = PythonInterpreter::new().unwrap();
        let mut out = String::new();
        interp.eval("x = 42", &mut out);

        let value = interp.get_int("x");
        assert_eq!(value, 42);
    }

    #[test]
    fn test_run_function() {
        let mut interp = PythonInterpreter::new().unwrap();

        // Define a function that takes a string and returns it uppercased
        let mut out = String::new();
        interp.eval("def test_func(s):\n  return s.upper()", &mut out);

        // Call it via run
        let mut result = String::new();
        let ok = interp.run("test_func", "hello", &mut result);
        assert!(ok);
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_load_file() {
        let mut interp = PythonInterpreter::new().unwrap();

        // Create temp file
        std::fs::write("/tmp/test_mcl_python.py", "test_value = 999").unwrap();

        // Load file
        let ok = interp.load_file("/tmp/test_mcl_python.py", false);
        assert!(ok);

        // Check variable was set
        let value = interp.get_int("test_value");
        assert_eq!(value, 999);

        // Cleanup
        std::fs::remove_file("/tmp/test_mcl_python.py").unwrap();
    }

    #[test]
    fn test_python_computation() {
        let mut interp = PythonInterpreter::new().unwrap();

        // Run actual Python computation
        let mut out = String::new();
        interp.eval("result = sum([i*i for i in range(10)])", &mut out);

        // sum of squares 0^2 + 1^2 + ... + 9^2 = 285
        let value = interp.get_int("result");
        assert_eq!(value, 285);
    }

    #[test]
    fn test_run_quietly_suppresses_errors() {
        let mut interp = PythonInterpreter::new().unwrap();

        let mut out = String::new();
        // Call non-existent function with suppress=true
        let ok = interp.run_quietly("nonexistent", "arg", &mut out, true);
        assert!(!ok);
    }
}
