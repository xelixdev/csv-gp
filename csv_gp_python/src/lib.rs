use std::collections::HashSet;

use ::csv_gp::{csv_details::CSVDetails, error::CSVError};
use error::until_err;
use pyo3::{create_exception, exceptions::PyValueError, prelude::*};

mod error;

// Results struct wrapper
#[pyclass(name = "CSVDetails", module = "csv_gp")]
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
    fn blank_rows(&self) -> Vec<usize> {
        self.0.blank_rows.clone()
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

create_exception!(csv_gp, PyUnknownEncoding, pyo3::exceptions::PyException);

struct PyCSVError(CSVError);

impl From<PyCSVError> for PyErr {
    fn from(e: PyCSVError) -> Self {
        match e.0 {
            CSVError::UnknownEncoding(x) => PyUnknownEncoding::new_err(x.to_string()),
            x => PyValueError::new_err(x.to_string()),
        }
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
    delimiter: char,
    encoding: &str,
    valid_rows_output_path: Option<&str>,
) -> Result<PyCSVDetails, PyCSVError> {
    let result = ::csv_gp::checker::check_file(path, delimiter, encoding, valid_rows_output_path)?;
    Ok(PyCSVDetails::new(result))
}

#[pyfunction]
fn get_rows(
    path: String,
    delimiter: char,
    encoding: &str,
    row_numbers: HashSet<usize>,
) -> Result<Vec<(usize, Vec<String>)>, PyCSVError> {
    let lines = ::csv_gp::parser::parse_file(path, delimiter, encoding)?;

    let mut err = Ok(());

    let filtered = lines
        .scan(&mut err, until_err)
        .enumerate()
        .filter(|(i, _)| row_numbers.contains(i))
        .map(|(i, x)| (i, x.into_iter().map(|y| y.to_string()).collect()))
        .collect();
    err.map_err(Into::<CSVError>::into)?;

    Ok(filtered)
}

#[pymodule]
fn csv_gp(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_file, m)?)?;
    m.add_function(wrap_pyfunction!(get_rows, m)?)?;
    m.add_class::<PyCSVDetails>()?;
    m.add("UnknownEncoding", py.get_type::<PyUnknownEncoding>())?;
    Ok(())
}
