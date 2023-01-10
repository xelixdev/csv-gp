use std::collections::HashSet;

use ::csv_gp::{csv_details::CSVDetails, error::CSVError};
use pyo3::{exceptions::PyValueError, prelude::*};

// Results struct wrapper
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
    fn invalid_character_count(&self) -> usize {
        self.0.invalid_character_count
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

    #[getter]
    fn valid_rows(&self) -> HashSet<usize> {
        self.0.valid_rows.clone()
    }

    #[getter]
    fn header_messed_up(&self) -> bool {
        self.0.header_messed_up()
    }
}

// Error wrapper
struct PyCSVError(CSVError);

impl From<PyCSVError> for PyErr {
    fn from(e: PyCSVError) -> Self {
        PyValueError::new_err(e.0.to_string())
    }
}

impl From<CSVError> for PyCSVError {
    fn from(e: CSVError) -> Self {
        Self(e)
    }
}

#[pyfunction]
fn check_file(
    path: String,
    delimiter: &str,
    encoding: &str,
    valid_rows_output_path: Option<&str>,
) -> Result<PyCSVDetails, PyCSVError> {
    let result = ::csv_gp::checker::check_file(path, delimiter, encoding, valid_rows_output_path)?;
    Ok(PyCSVDetails::new(result))
}

#[pymodule]
fn csv_gp(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_file, m)?)?;
    m.add_class::<PyCSVDetails>()?;
    Ok(())
}
