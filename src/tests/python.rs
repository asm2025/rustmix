use std::error::Error;

use pyo3::types::{PyBytes, PyDict};
use rustmix::python::{self, whisper};

pub fn test_python() -> Result<(), Box<dyn Error>> {
    let version = python::version()?;
    let user = python::user()?;
    println!("Python version: {:?}", version);
    println!("Username: {}", user);

    python::run("src/tests/test.py")?;

    let str = "Hello Rust from Python code!";
    let code = format!(
        r#"
import base64
s = '{}'
ret = base64.b64encode(s.encode('utf-8'))
"#,
        str
    );
    python::exec(&code, |l| {
        let ret = l.get_item("ret").unwrap().unwrap();
        let b64: &PyBytes = ret.downcast().unwrap();
        let b64 = b64.to_string();
        println!("String: '{}', Base64: '{}'", &str, &b64);
    })?;
    let code = r#"
def change_my_var():
    if 'myVar' in globals():
        globals()['myVar'] = 'New Value changed from Python code!'
        print(globals()['myVar'])
    else:
        print('myVar is not in globals')

if __name__ == "__main__":
    change_my_var()
"#;
    python::exec_with(
        &code,
        |g| g.set_item("myVar", "Original Value").unwrap(),
        |l| l.set_item("__name__", "__main__").unwrap(),
        |g, _l| {
            let my_var: String = g.get_item("myVar").unwrap().unwrap().extract().unwrap();
            println!("back to rust -> myVar: {}", my_var);
        },
    )?;
    Ok(())
}
