// pub mod whisper;

// use pyo3::prelude::*;
// use pyo3::types::IntoPyDict;

// #[derive(Debug)]
// pub struct VersionInfo {
//     pub major: u8,
//     pub minor: u8,
//     pub patch: u8,
//     pub suffix: Option<String>,
// }

// pub fn version() -> PyResult<VersionInfo> {
//     Python::with_gil(|py| {
//         let version = py.version_info();
//         Ok(VersionInfo {
//             major: version.major,
//             minor: version.minor,
//             patch: version.patch,
//             suffix: version.suffix.map(|s| s.to_string()),
//         })
//     })
// }

// pub fn user() -> PyResult<String> {
//     Python::with_gil(|py| {
//         let locals = [("os", py.import("os")?)].into_py_dict(py);
//         let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
//         let user: String = py.eval(code, None, Some(&locals))?.extract()?;
//         Ok(user)
//     })
// }
