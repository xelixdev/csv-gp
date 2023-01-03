use pyo3::prelude::*;

#[pyfunction]
fn check_file(path: String, delimiter: &str) -> PyResult<String> {
    let result = csv_gp::checker::check_file(path, delimiter).unwrap();
    Ok(format!("There are {} rows in the file", result.row_count))
}

/// A Python module implemented in Rust.
#[pymodule]
fn csv_gp_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_file, m)?)?;
    Ok(())
}
