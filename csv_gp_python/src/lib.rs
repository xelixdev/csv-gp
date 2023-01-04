use ::csv_gp::checker::CSVDetails;
use pyo3::prelude::*;

#[pyclass(name = "CSVDetails")]
struct PyCSVDetails(CSVDetails);

impl PyCSVDetails {
    fn new(csv_details: CSVDetails) -> Self {
        Self(csv_details)
    }
}

#[pymethods]
impl PyCSVDetails {
    #[getter]
    fn row_count(&self) -> usize {
        self.0.row_count
    }

    #[getter]
    fn column_count(&self) -> usize {
        self.0.column_count
    }

    #[getter]
    fn column_count_per_line(&self) -> Vec<usize> {
        self.0.column_count_per_line.clone()
    }

    #[getter]
    fn too_few_columns(&self) -> Vec<usize> {
        self.0.too_few_columns.clone()
    }

    #[getter]
    fn too_many_columns(&self) -> Vec<usize> {
        self.0.too_many_columns.clone()
    }

    #[getter]
    fn quoted_delimiter(&self) -> Vec<usize> {
        self.0.quoted_delimiter.clone()
    }

    #[getter]
    fn quoted_newline(&self) -> Vec<usize> {
        self.0.quoted_newline.clone()
    }

    #[getter]
    fn quoted_quote(&self) -> Vec<usize> {
        self.0.quoted_quote.clone()
    }

    #[getter]
    fn quoted_quote_correctly(&self) -> Vec<usize> {
        self.0.quoted_quote_correctly.clone()
    }

    #[getter]
    fn incorrect_cell_quote(&self) -> Vec<usize> {
        self.0.incorrect_cell_quote.clone()
    }

    #[getter]
    fn all_empty_rows(&self) -> Vec<usize> {
        self.0.all_empty_rows.clone()
    }
}

#[pyfunction]
fn check_file(path: String, delimiter: &str) -> PyResult<PyCSVDetails> {
    let result = ::csv_gp::checker::check_file(path, delimiter)?;
    Ok(PyCSVDetails::new(result))
}

/// A Python module implemented in Rust.
#[pymodule]
fn csv_gp(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_file, m)?)?;
    m.add_class::<PyCSVDetails>()?;
    Ok(())
}
