use pyo3::exceptions;
use pyo3::prelude::*;

/// Inspect git core.editor setting, $EDITOR and $VISUAL for a command that opens an editor
///
/// If no editor is found, open vi
#[pyfunction]
fn determine_editor() -> PyResult<String> {
    Ok(crate::determine_editor().unwrap())
}

#[pyfunction]
fn invoke(initial: String) -> PyResult<String> {
    let hehe = crate::HotEdit::new();
    match hehe.invoke(&initial) {
        Ok(s) => Ok(s),
        Err(e) => {
            let ee = exceptions::PyRuntimeError::new_err(format!("{}", e));
            Err(ee)
            // FIXME -- are we leaking this exception?
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "hotedit")]
fn hotedit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(determine_editor, m)?)?;
    m.add_function(wrap_pyfunction!(invoke, m)?)?;
    Ok(())
}
