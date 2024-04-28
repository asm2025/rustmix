use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyDict},
};
use std::fs;

#[derive(Debug)]
pub struct VersionInfo {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub suffix: Option<String>,
}

pub fn version() -> PyResult<VersionInfo> {
    Python::with_gil(|py| {
        let version = py.version_info();
        Ok(VersionInfo {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            suffix: version.suffix.map(|s| s.to_string()),
        })
    })
}

pub fn user() -> PyResult<String> {
    Python::with_gil(|py| {
        let locals = [("os", py.import_bound("os")?)].into_py_dict_bound(py);
        let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
        let user: String = py.eval_bound(code, None, Some(&locals))?.extract()?;
        Ok(user)
    })
}

pub fn run(file_name: &str) -> PyResult<()> {
    Python::with_gil(|py| {
        let code = fs::read_to_string(file_name)?;
        let locals = [("__name__", "__main__")].into_py_dict_bound(py);
        py.run_bound(&code, None, Some(&locals))?;
        Ok(())
    })
}

pub fn exec<C: FnOnce(&Bound<'_, PyDict>)>(code: &str, callback: C) -> PyResult<()> {
    Python::with_gil(|py| {
        let locals = PyDict::new_bound(py);
        py.run_bound(&code, None, Some(&locals))?;
        callback(&locals);
        Ok(())
    })
}

pub fn exec_with<
    G: FnOnce(&Bound<'_, PyDict>),
    L: FnOnce(&Bound<'_, PyDict>),
    C: FnOnce(&Bound<'_, PyDict>, &Bound<'_, PyDict>),
>(
    code: &str,
    set_globals: G,
    set_locals: L,
    callback: C,
) -> PyResult<()> {
    Python::with_gil(|py| {
        let global = PyDict::new_bound(py);
        let locals = PyDict::new_bound(py);
        set_globals(&global);
        set_locals(&locals);
        py.run_bound(&code, Some(&global), Some(&locals))?;
        callback(&global, &locals);
        Ok(())
    })
}
