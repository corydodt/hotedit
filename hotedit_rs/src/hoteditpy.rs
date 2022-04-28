use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Inspect git core.editor setting, $EDITOR and $VISUAL for a command that opens an editor
///
/// If no editor is found, open vi
#[pyfunction]
fn determine_editor() -> PyResult<String> {
    Ok(crate::determine_editor().unwrap())
}

/// present a python function as a rust closure for the HotEdit{}
/// interface
fn wrap_find_editor(obj: PyObject) -> Box<crate::EditorFindFn> {
    Box::new(move || -> crate::ResultString {
        Python::with_gil(|py| {
            // if this isn't callable you get a Python exception when it's
            // called, which is fine
            let pyr = obj.call0(py);
            match pyr {
                Ok(obj) => Ok(obj.to_string()),
                Err(e) => Err(Box::from(e.to_string())),
            }
        })
    })
}

#[pyfunction]
fn invoke(
    initial: String,
    validate_unchanged: Option<bool>,
    delete_temp: Option<bool>,
    find_editor: Option<PyObject>,
) -> PyResult<String> {
    let find_fn = match find_editor {
        Some(f) => wrap_find_editor(f),
        None => Box::new(crate::determine_editor),
    };
    let hehe = crate::HotEdit {
        validate_unchanged: validate_unchanged.unwrap_or(false),
        delete_temp: delete_temp.unwrap_or(true),
        find_editor: &find_fn,
    };
    match hehe.invoke(&initial) {
        Ok(s) => Ok(s),
        Err(e) => Err(PyRuntimeError::new_err(format!("{}", e))),
    }
}

/// The hotedit module
///
/// (Module contents mirror hotedit.editor from the pure py implementation)
#[pymodule]
#[pyo3(name = "hotedit")]
fn hotedit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(determine_editor, m)?)?;
    m.add_function(wrap_pyfunction!(invoke, m)?)?;
    Ok(())
}
