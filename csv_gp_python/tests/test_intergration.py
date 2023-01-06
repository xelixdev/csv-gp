from pathlib import Path

import pytest

import csv_gp

FIXTURES = Path(__file__).parent / "fixtures"


def test_kitchen_sink():
    result = csv_gp.check_file(str(FIXTURES / "kitchen_sink.csv"), ",", encoding="utf-8")

    assert result
    assert result.column_count == 2
    assert result.all_empty_rows == [1, 2, 3]
    assert result.quoted_delimiter == [4]
    assert result.quoted_newline == [5]
    assert result.quoted_quote == [6, 7]
    assert result.quoted_quote_correctly == [6]
    assert result.incorrect_cell_quote == [7]
    assert result.row_count == 7
    assert result.too_few_columns == [8]
    assert result.too_many_columns == [9]
    assert result.column_count_per_line == [2, 2, 2, 1, 2, 2, 2, 2, 1, 3]


def test_different_encoding():
    result = csv_gp.check_file(str(FIXTURES / "mac_roman.csv"), ",", encoding="macintosh")

    assert result
    assert result.column_count == 2
    assert result.row_count == 2
    assert result.invalid_character_count == 0


def test_unknown_encoding():
    with pytest.raises(ValueError):
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
