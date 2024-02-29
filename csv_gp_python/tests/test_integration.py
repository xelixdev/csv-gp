from pathlib import Path
from tempfile import NamedTemporaryFile

import csv_gp
import pytest

FIXTURES = Path(__file__).parent / "fixtures"


def test_kitchen_sink():
    result = csv_gp.check_file(str(FIXTURES / "kitchen_sink.csv"), ",", encoding="utf-8")

    assert result
    assert result.column_count == 2
    assert result.row_count == 9
    assert result.all_empty_rows == [1, 2]
    assert result.blank_rows == [3]
    assert result.quoted_delimiter == [4]
    assert result.quoted_newline == [5]
    assert result.quoted_quote == [6, 7]
    assert result.quoted_quote_correctly == [6]
    assert result.incorrect_cell_quote == [7]
    assert result.too_few_columns == [8]
    assert result.too_many_columns == [9]
    assert result.column_count_per_line == [2, 2, 2, 0, 2, 2, 2, 2, 1, 3]
    assert result.valid_rows == {0, 1, 2, 4, 5, 6}
    assert not result.header_messed_up


def test_invalid_delimiter():
    with pytest.raises(ValueError):
        csv_gp.check_file(str(FIXTURES / "kitchen_sink.csv"), ",+", encoding="utf-8")


def test_different_encoding():
    result = csv_gp.check_file(str(FIXTURES / "mac_roman.csv"), ",", encoding="macintosh")

    assert result
    assert result.column_count == 2
    assert result.row_count == 2
    assert result.invalid_character_count == 0


def test_unknown_encoding():
    with pytest.raises(csv_gp.UnknownEncoding):
        csv_gp.check_file(str(FIXTURES / "kitchen_sink.csv"), ",", encoding="foo")


def test_wrong_encoding():
    result = csv_gp.check_file(str(FIXTURES / "mac_roman.csv"), ",", encoding="utf-8")

    assert result
    assert result.column_count == 2
    assert result.row_count == 2
    assert result.invalid_character_count == 1


def test_file_non_existent():
    with pytest.raises(ValueError):
        csv_gp.check_file("shadow_realm.csv", ",", encoding="utf-8")


def test_empty_file():
    result = csv_gp.check_file(str(FIXTURES / "empty.csv"), ",", encoding="utf-8")

    assert result
    assert result.row_count == 0
    assert result.column_count == 0


def test_header_messed_up():
    result = csv_gp.check_file(str(FIXTURES / "header_messed_up.csv"), ",", encoding="utf-8")

    assert result
    assert result.header_messed_up


def test_output_file():
    with NamedTemporaryFile() as temp_file:
        result = csv_gp.check_file(
            str(FIXTURES / "kitchen_sink.csv"),
            ",",
            encoding="utf-8",
            valid_rows_output_path=temp_file.name,
        )

        assert result
        assert temp_file.read() == (FIXTURES / "kitchen_sink_valid.csv").read_bytes()


def test_get_rows():
    result = csv_gp.get_rows(str(FIXTURES / "kitchen_sink.csv"), ",", encoding="utf-8", row_numbers={0, 1, 3})

    assert len(result) == 3
    assert result == [(0, ["a", "b"]), (1, ["", ""]), (3, [])]


def test_unknown_encoding_ibm852():
    with pytest.raises(csv_gp.UnknownEncoding, match="unknown encoding ibm852"):
        csv_gp.check_file(str(FIXTURES / "kitchen_sink.csv"), ",", encoding="ibm852")


def test_quote_and_newline():
    result = csv_gp.check_file(str(FIXTURES / "quote_and_newline.csv"), ",", encoding="utf-8")

    assert result
    assert result.column_count == 3
    assert result.row_count == 6
    assert result.all_empty_rows == []
    assert result.blank_rows == []
    assert result.quoted_delimiter == [1, 2]
    assert result.quoted_newline == [1, 2, 3, 4]
    assert result.quoted_quote == [1, 2, 3, 4]
    assert result.quoted_quote_correctly == [1, 2, 3, 4]
    assert result.incorrect_cell_quote == []
    assert result.too_few_columns == []
    assert result.too_many_columns == []
    assert result.column_count_per_line == [3] * 6
    assert result.valid_rows == {0, 1, 2, 3, 4, 5}
    assert not result.header_messed_up


def test_quote_last_cell():
    result = csv_gp.check_file(str(FIXTURES / "quote_last_cell.csv"), ",", encoding="utf-8")

    assert result
    assert result.column_count == 3
    assert result.row_count == 3
    assert result.all_empty_rows == []
    assert result.blank_rows == []
    assert result.quoted_delimiter == [2]
    assert result.quoted_newline == []
    assert result.quoted_quote == []
    assert result.quoted_quote_correctly == []
    assert result.incorrect_cell_quote == []
    assert result.too_few_columns == []
    assert result.too_many_columns == []
    assert result.column_count_per_line == [3] * 3
    assert result.valid_rows == {0, 1, 2}
    assert not result.header_messed_up


def test_incorrect_quote():
    result = csv_gp.check_file(str(FIXTURES / "incorrect_quote.csv"), ",", encoding="utf-8")

    assert result
    assert result.column_count == 3
    assert result.row_count == 4
    assert result.all_empty_rows == []
    assert result.blank_rows == []
    assert result.quoted_delimiter == [3]
    assert result.quoted_newline == []
    assert result.quoted_quote == []
    assert result.quoted_quote_correctly == []
    assert result.incorrect_cell_quote == [1, 2, 3]
    assert result.too_few_columns == [2, 3]
    assert result.too_many_columns == []
    assert result.column_count_per_line == [3, 3, 2, 1]
    assert result.valid_rows == {0}
    assert not result.header_messed_up
