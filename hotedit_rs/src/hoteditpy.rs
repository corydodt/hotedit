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
fn invoke(
    initial: String,
    validate_unchanged: Option<bool>,
    delete_temp: Option<bool>,
) -> PyResult<String> {
    let hehe = crate::HotEdit {
        validate_unchanged: validate_unchanged.unwrap_or(false),
        delete_temp: delete_temp.unwrap_or(true),
        find_editor: crate::determine_editor,
    };
    match hehe.invoke(&initial) {
        Ok(s) => Ok(s),
        Err(e) => {
            let ee = exceptions::PyRuntimeError::new_err(format!("{}", e));
            Err(ee)
            // FIXME -- are we leaking this exception?
        }
    }
}

/// The hotedit module
///
/// (Contents mirror hotedit.editor from the pure py implementation)
#[pymodule]
#[pyo3(name = "hotedit")]
fn hotedit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(determine_editor, m)?)?;
    m.add_function(wrap_pyfunction!(invoke, m)?)?;
    Ok(())
}
