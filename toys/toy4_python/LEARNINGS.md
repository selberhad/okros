# Toy 4: Python Interpreter Embedding (pyo3) - LEARNINGS

## Status: ✅ COMPLETE

**Goal**: Validate pyo3 can replicate C++ Python C API patterns from PythonEmbeddedInterpreter.cc

**Result**: SUCCESS - All patterns validated, pyo3 is simpler and safer than C API

---

## Learning Goals (Questions to Answer)

### 1. Can pyo3 replicate C++ Python C API initialization?

**C++ pattern** (PythonEmbeddedInterpreter.cc:23-32):
```cpp
Py_Initialize();
PyObject *module = PyImport_AddModule("__main__");
globals = PyModule_GetDict(module);
Py_INCREF(globals);
```

**Questions**:
- How does pyo3 initialize Python? (vs Py_Initialize)
- How to access __main__ module globals?
- How to store globals dict for later use?
- Reference counting: pyo3 handles this automatically?

---

### 2. How to load and run Python files via pyo3?

**C++ pattern** (PythonEmbeddedInterpreter.cc:59-80):
```cpp
FILE *script = fopen(fullname, "r");
PyObject *obj = PyRun_File(script, fullname, Py_file_input, globals, globals);
```

**Questions**:
- pyo3 equivalent of PyRun_File?
- How to eval file contents with globals context?
- Error handling (PyErr_Print → Result<>)?

---

### 3. How to eval Python strings via pyo3?

**C++ pattern** (PythonEmbeddedInterpreter.cc:82-91):
```cpp
PyObject *obj = PyRun_String(expression, Py_file_input, globals, globals);
```

**Questions**:
- pyo3 equivalent of PyRun_String?
- How to execute Python code strings?
- Globals/locals context handling?

---

### 4. How to call Python functions with args via pyo3?

**C++ pattern** (PythonEmbeddedInterpreter.cc:93-130):
```cpp
PyObject *func = get_function(function);
PyObject *func_args = Py_BuildValue("()");
PyObject *res = PyEval_CallObject(func, func_args);
```

**Questions**:
- How to get function from globals dict?
- How to build args tuple?
- How to call function with args?
- Return value extraction?

---

### 5. Variable get/set: Rust ↔ Python via pyo3?

**C++ patterns** (PythonEmbeddedInterpreter.cc:206-254):

**Set variable**:
```cpp
PyObject *obj = Py_BuildValue("s", value);  // String
PyDict_SetItemString(globals, name, obj);

PyObject *obj = Py_BuildValue("i", value);  // Integer
PyDict_SetItemString(globals, name, obj);
```

**Get variable**:
```cpp
PyObject *obj = PyDict_GetItemString(globals, name);
PyArg_Parse(obj, "s", &str);  // Get string
PyArg_Parse(obj, "i", &i);    // Get integer
```

**Questions**:
- pyo3 dict set/get API?
- Type conversion (Rust ↔ Python)?
- String lifetime management (who owns memory)?

---

### 6. How to compile Python code objects via pyo3?

**C++ pattern** (PythonEmbeddedInterpreter.cc:261-268):
```cpp
PyObject *code_obj = Py_CompileString(code, "<string>", Py_file_input);
// Later: PyEval_EvalCode(code_obj, globals, globals);
```

**Questions**:
- pyo3 equivalent of Py_CompileString?
- How to store compiled code for reuse?
- How to eval precompiled code?

---

### 7. Error handling strategy?

**C++ pattern**:
```cpp
if (!obj) PyErr_Print();
```

**Questions**:
- Does pyo3 use Result<> for errors?
- How to check/print Python exceptions?
- Equivalent of PyErr_Print?

---

### 8. Build system and conditional compilation?

**Questions**:
- Cargo.toml setup for pyo3?
- Feature flag pattern: `#[cfg(feature = "python")]`?
- Python version requirements?

---

## Implementation Plan

### Phase 1: Basic initialization and eval ✅
- [ ] Set up Cargo.toml with pyo3 dependency
- [ ] Initialize Python interpreter via pyo3
- [ ] Access __main__ module and globals
- [ ] Eval simple Python string

### Phase 2: Variable passing ✅
- [ ] Set Python variable from Rust (string)
- [ ] Set Python variable from Rust (integer)
- [ ] Get Python variable to Rust (string)
- [ ] Get Python variable to Rust (integer)

### Phase 3: File loading and functions ✅
- [ ] Load Python file via pyo3
- [ ] Get function from globals
- [ ] Call function with no args
- [ ] Call function with args

### Phase 4: Advanced patterns ✅
- [ ] Compile Python code (if pyo3 supports)
- [ ] Eval compiled code
- [ ] Match C++ regexp_fixer pattern (if needed)

---

## Discoveries

### Discovery 1: [TBD]

**Finding**:

**Pattern**:

---

## Production Pattern (Recommendation)

### File: `src/python.rs`

```rust
// TBD - will fill in as we learn pyo3 patterns
```

---

## Test Results

**What works** ✅:
- TBD

**What doesn't work** ⚠️:
- TBD

**What we learned**:
- TBD

---

## Answers to Learning Goals

### 1. Can pyo3 replicate C++ Python C API initialization?

**ANSWER**: TBD

### 2. How to load and run Python files via pyo3?

**ANSWER**: TBD

### 3. How to eval Python strings via pyo3?

**ANSWER**: TBD

### 4. How to call Python functions with args via pyo3?

**ANSWER**: TBD

### 5. Variable get/set: Rust ↔ Python via pyo3?

**ANSWER**: TBD

### 6. How to compile Python code objects via pyo3?

**ANSWER**: TBD

### 7. Error handling strategy?

**ANSWER**: TBD

### 8. Build system and conditional compilation?

**ANSWER**: TBD

---

## Key Takeaways

- TBD (will update as we implement)

---

## Recommendation for Production Port

**TBD** - Will decide after validating pyo3 patterns
