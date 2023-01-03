fn split_rows(text: &str, delimiter: &str) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut rows = Vec::new();

    let mut opened_quotes = false;
    let mut current_selection = String::new();

    let mut i = 0usize;
    while i < text.len() {
        let current_char = chars[i];

        // Reached a newline not in quotes
        if current_char == '\n' && !opened_quotes {
            rows.push(current_selection.clone());
            current_selection = String::new();
        } else if current_char == '"' {
            current_selection.push(current_char);
            if let Some(next_char) = chars.get(i + 1) {
                if next_char == &'"' {
                    current_selection.push(*next_char);
                    i += 1;
                } else if current_selection == "\""
                    || (current_selection.ends_with(&format!("{}{}", delimiter, "\""))
                        && !opened_quotes)
                {
                    opened_quotes = true;
                } else if next_char.to_string() == delimiter || next_char == &'\n' {
                    opened_quotes = false;
                }
            }
        } else {
            current_selection.push(current_char);
        }

        i += 1;
    }

    // There was no trailing newline, append the rest of the data
    if !current_selection.is_empty() {
        rows.push(current_selection);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "test,row\nnext,row\n";

        assert_eq!(split_rows(input, ","), vec!["test,row", "next,row"])
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "test,row\nnext,row";

        assert_eq!(split_rows(input, ","), vec!["test,row", "next,row"])
    }

    #[test]
    fn test_quoted_newline() {
        let input = "test,\"row\n\"\nnext,row";

        assert_eq!(split_rows(input, ","), vec!["test,\"row\n\"", "next,row"])
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"\nnext,row";

        assert_eq!(
            split_rows(input, ","),
            vec!["test,\"\"\"row\"\"\"", "next,row"]
        )
    }
}
