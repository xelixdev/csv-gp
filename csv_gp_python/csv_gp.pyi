from typing import Optional

class UnknownEncoding(Exception):
    pass

class CSVDetails:
    @property
    def row_count(self) -> int:
        """
        Number of non-empty rows (including the header) in the file
        """
        ...

    @property
    def column_count(self) -> int:
        """
        Number of columns according to the header
        """
        ...

    @property
    def invalid_character_count(self) -> int:
        """
        Number of REPLACEMENT CHARACTERs (U+FFFD) in the file
        """
        ...

    @property
    def column_count_per_line(self) -> list[int]:
        """
        Number of columns per line, the index corresponding to the line number
        """
        ...

    @property
    def too_few_columns(self) -> list[int]:
        """
        List of line numbers that contain fewer columns than the header
        """
        ...

    @property
    def too_many_columns(self) -> list[int]:
        """
        List of line numbers that contain more columns than the header
        """
        ...

    @property
    def quoted_delimiter(self) -> list[int]:
        """
        List of line numbers that contain a correctly quoted delimiter
        """
        ...

    @property
    def quoted_newline(self) -> list[int]:
        """
        List of line numbers that contain a correctly quoted newline
        """
        ...

    @property
    def quoted_quote(self) -> list[int]:
        """
        List of line numbers that contain quoted-quotes ("")
        """
        ...

    @property
    def quoted_quote_correctly(self) -> list[int]:
        """
        List of line numbers that contain correctly quoted-quotes (only contained within quoted cells)
        """
        ...

    @property
    def incorrect_cell_quote(self) -> list[int]:
        """
        List of line numbers that have incorrectly quoted cells

        Incorrect meaning:
            - Missing an opening or closing quote
            - Containing unquoted quotes
        """
        ...

    @property
    def all_empty_rows(self) -> list[int]:
        """
        List of line numbers where all cells in the row are empty (either zero characters or just `""`)
        """
        ...

    @property
    def blank_rows(self) -> list[int]:
        """
        List of line numbers that are completely blank
        """
        ...

    @property
    def valid_rows(self) -> set[int]:
        """
        Set of all row numbers that are valid in the file
        """
        ...

    @property
    def header_messed_up(self) -> bool:
        """
        The header is considered messed up when none of the rows have the same number of columns as the header
        """
        ...

def check_file(path: str, delimiter: str, encoding: str, valid_rows_output_path: Optional[str] = None) -> CSVDetails:
    """
    Check the file located at `path`, interpreting the file with `delimiter` and `encoding`

    If `valid_rows_output_path` is passed, a file containing the valid rows will be written to the specified path
    """
    ...

def get_rows(path: str, delimiter: str, encoding: str, row_numbers: set[int]) -> list[tuple[int, list[str]]]:
    """
    Returns all the rows in the file in the `row_numbers` set
    """
    ...
