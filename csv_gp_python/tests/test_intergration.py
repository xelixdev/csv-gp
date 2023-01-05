from pathlib import Path

import csv_gp
import pytest

FIXTURES = Path(__file__).parent / "fixtures"


def test_kitchen_sink():
    result = csv_gp.check_file(str(FIXTURES / "kitchen_sink.csv"), ",", encoding="utf-8")

    assert result
    assert result.column_count == 2
    assert result.all_empty_rows == [1, 2]
    assert result.quoted_delimiter == [3]
    assert result.quoted_newline == [4]
    assert result.quoted_quote == [5, 6]
    assert result.quoted_quote_correctly == [5]
    assert result.incorrect_cell_quote == [6]
    assert result.row_count == 9
    assert result.too_few_columns == [7]
    assert result.too_many_columns == [8]
    assert result.column_count_per_line == [2] * 7 + [1, 3]


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
