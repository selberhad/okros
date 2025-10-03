use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

/// Python interpreter wrapper matching C++ PythonEmbeddedInterpreter patterns
pub struct PythonEmbeddedInterpreter {
    globals: Py<PyDict>,
}

impl PythonEmbeddedInterpreter {
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

            Ok(PythonEmbeddedInterpreter { globals })
        })
    }

    /// Eval Python expression string
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:82-91):
    /// ```cpp
    /// PyObject *obj = PyRun_String(expression, Py_file_input, globals, globals);
    /// ```
    pub fn eval(&self, expression: &str) -> PyResult<()> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);

            // Run Python code with globals as both globals and locals
            py.run_bound(expression, Some(&globals), Some(&globals))?;

            Ok(())
        })
    }

    /// Load and run Python file
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:59-80):
    /// ```cpp
    /// FILE *script = fopen(fullname, "r");
    /// PyObject *obj = PyRun_File(script, fullname, Py_file_input, globals, globals);
    /// ```
    pub fn load_file(&self, path: &str) -> PyResult<()> {
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

    /// Set string variable in Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:218-228):
    /// ```cpp
    /// PyObject *obj = Py_BuildValue("s", value);
    /// PyDict_SetItemString(globals, name, obj);
    /// ```
    pub fn set_string(&self, name: &str, value: &str) -> PyResult<()> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            globals.set_item(name, value)?;
            Ok(())
        })
    }

    /// Set integer variable in Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:206-216):
    /// ```cpp
    /// PyObject *obj = Py_BuildValue("i", value);
    /// PyDict_SetItemString(globals, name, obj);
    /// ```
    pub fn set_int(&self, name: &str, value: i32) -> PyResult<()> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            globals.set_item(name, value)?;
            Ok(())
        })
    }

    /// Get string variable from Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:243-254):
    /// ```cpp
    /// PyObject *obj = PyDict_GetItemString(globals, name);
    /// PyArg_Parse(obj, "s", &str);
    /// ```
    pub fn get_string(&self, name: &str) -> PyResult<String> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            let value = globals.get_item(name)?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                    format!("Variable '{}' not found", name)
                ))?;

            value.extract::<String>()
        })
    }

    /// Get integer variable from Python globals
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:230-241):
    /// ```cpp
    /// PyObject *obj = PyDict_GetItemString(globals, name);
    /// PyArg_Parse(obj, "i", &i);
    /// ```
    pub fn get_int(&self, name: &str) -> PyResult<i32> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);
            let value = globals.get_item(name)?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                    format!("Variable '{}' not found", name)
                ))?;

            value.extract::<i32>()
        })
    }

    /// Call Python function with no args
    ///
    /// C++ equivalent (PythonEmbeddedInterpreter.cc:115-123):
    /// ```cpp
    /// PyObject *func_args = Py_BuildValue("()");
    /// PyObject *res = PyEval_CallObject(func, func_args);
    /// ```
    pub fn call_function(&self, function_name: &str) -> PyResult<()> {
        Python::with_gil(|py| {
            let globals = self.globals.bind(py);

            // Get function from globals
            let func = globals.get_item(function_name)?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                    format!("Function '{}' not found", function_name)
                ))?;

            // Call function with no args
            func.call0()?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let _interp = PythonEmbeddedInterpreter::new().unwrap();
        // If we got here, initialization worked
    }

    #[test]
    fn test_eval() {
        let interp = PythonEmbeddedInterpreter::new().unwrap();
        interp.eval("x = 42").unwrap();

        let value = interp.get_int("x").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_set_get_string() {
        let interp = PythonEmbeddedInterpreter::new().unwrap();
        interp.set_string("test_var", "Hello from Rust!").unwrap();

        let value = interp.get_string("test_var").unwrap();
        assert_eq!(value, "Hello from Rust!");
    }

    #[test]
    fn test_set_get_int() {
        let interp = PythonEmbeddedInterpreter::new().unwrap();
        interp.set_int("num", 123).unwrap();

        let value = interp.get_int("num").unwrap();
        assert_eq!(value, 123);
    }

    #[test]
    fn test_call_function() {
        let interp = PythonEmbeddedInterpreter::new().unwrap();

        // Define a simple function
        interp.eval("def test_func():\n  global result\n  result = 'called'").unwrap();

        // Call it
        interp.call_function("test_func").unwrap();

        // Check result
        let value = interp.get_string("result").unwrap();
        assert_eq!(value, "called");
    }

    #[test]
    fn test_load_file() {
        let interp = PythonEmbeddedInterpreter::new().unwrap();

        // Create temp file
        std::fs::write("/tmp/test_script.py", "test_value = 999").unwrap();

        // Load file
        interp.load_file("/tmp/test_script.py").unwrap();

        // Check variable was set
        let value = interp.get_int("test_value").unwrap();
        assert_eq!(value, 999);

        // Cleanup
        std::fs::remove_file("/tmp/test_script.py").unwrap();
    }

    #[test]
    fn test_python_computation() {
        let interp = PythonEmbeddedInterpreter::new().unwrap();

        // Run actual Python computation - this would fail if Python isn't executing
        interp.eval("result = sum([i*i for i in range(10)])").unwrap();

        // sum of squares 0^2 + 1^2 + ... + 9^2 = 285
        let value = interp.get_int("result").unwrap();
        assert_eq!(value, 285);
    }

    #[test]
    fn test_python_strings() {
        let interp = PythonEmbeddedInterpreter::new().unwrap();

        // Python string manipulation
        interp.eval("msg = 'hello ' + 'world'").unwrap();
        interp.eval("msg = msg.upper()").unwrap();

        let value = interp.get_string("msg").unwrap();
        assert_eq!(value, "HELLO WORLD");
    }
}
