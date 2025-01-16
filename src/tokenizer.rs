use regex::{Captures, Regex};

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Identifier,
    Integer,
    Operator,
    Punctuation,
    End,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub location: (usize, usize),
}

// Returns (capture_str, number_of_newlines, last_newline_pos)
pub fn parse_capture(cap: Captures<'_>) -> (String, usize, usize) {
    let text = cap.get(0).unwrap().as_str().to_string();
    let newlines = Regex::new(r"\n").unwrap();
    let (len, pos) = match newlines.captures(&text) {
        Some(new_cap) => (new_cap.len(), new_cap.get(new_cap.len() - 1).unwrap().end()),
        None => (0, 0),
    };
    return (text, len, pos);
}

pub fn tokenize(source_code: String) -> Vec<Token> {
    let whitespace = Regex::new(r"^\s+").unwrap();
    let multi_line_comment = Regex::new(r"(?s)^\/\*.*?\*\/").unwrap();
    let comment = Regex::new(r"^(?:\/\/|#)[^\n]*").unwrap();
    let identifier = Regex::new(r"^[a-zA-Z_][a-zA-Z\d_]*").unwrap();
    let integer = Regex::new(r"^\d+").unwrap(); // Allow leading zeroes
    let operator = Regex::new(r"^(?:\+|\-|\*|\/|==?|!=|<=?|>=?)").unwrap();
    let punctuation = Regex::new(r"^(?:\(|\)|\{|\}|,|;)").unwrap();

    let mut last_newline = 0;
    let mut tokens: Vec<Token> = Vec::new();
    let mut line_num = 1;
    let mut i = 0;
    while i < source_code.len() {
        let substr = &source_code[i..];

        // Ignored matches
        // Whitespace
        let capture = whitespace.captures(substr);
        if let Some(cap) = capture {
            let (text, newlines, newline_pos) = parse_capture(cap);
            i += text.len();
            if newlines > 0 {
                line_num += newlines;
                last_newline = (i - text.len()) + newline_pos;
            }
            continue;
        }
        // Multi-line-comments
        let capture = multi_line_comment.captures(substr);
        if let Some(cap) = capture {
            let (text, newlines, newline_pos) = parse_capture(cap);
            i += text.len();
            if newlines > 0 {
                line_num += newlines;
                last_newline = (i - text.len()) + newline_pos;
            }
            continue;
        }

        // One line comments
        let capture = comment.captures(substr);
        if let Some(cap) = capture {
            let text = cap.get(0).unwrap().as_str().to_string();
            i += &text.len();
            continue;
        }
        // End of ignored matches

        let col_num = i - last_newline + 1;

        // TODO: Deduplicate code

        // Identifier
        let capture = identifier.captures(substr);
        if let Some(cap) = capture {
            let text = cap.get(0).unwrap().as_str().to_string();
            i += &text.len();
            tokens.push(Token {
                token_type: TokenType::Identifier,
                value: text,
                location: (line_num, col_num),
            });
            continue;
        }

        // Integer
        let capture = integer.captures(substr);
        if let Some(cap) = capture {
            let text = cap.get(0).unwrap().as_str().to_string();
            i += &text.len();
            tokens.push(Token {
                token_type: TokenType::Integer,
                value: text,
                location: (line_num, col_num),
            });
            continue;
        }

        // Operator
        let capture = operator.captures(substr);
        if let Some(cap) = capture {
            let text = cap.get(0).unwrap().as_str().to_string();
            i += &text.len();
            tokens.push(Token {
                token_type: TokenType::Operator,
                value: text,
                location: (line_num, col_num),
            });
            continue;
        }

        // Punctuation
        let capture = punctuation.captures(substr);
        if let Some(cap) = capture {
            let text = cap.get(0).unwrap().as_str().to_string();
            i += &text.len();
            tokens.push(Token {
                token_type: TokenType::Punctuation,
                value: text,
                location: (line_num, col_num),
            });
            continue;
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test_with_locations(code_str: &str, expected: Vec<(TokenType, &str, (usize, usize))>) {
        let code = code_str.to_string();
        let tokens = tokenize(code);
        assert_eq!(tokens.len(), expected.len());
        for i in 0..tokens.len() {
            assert_eq!(tokens[i].token_type, expected[i].0);
            assert_eq!(tokens[i].value, expected[i].1.to_string());
            assert_eq!(tokens[i].location, expected[i].2);
        }
    }

    fn run_test(code_str: &str, expected: Vec<(TokenType, &str)>) {
        let code = code_str.to_string();
        let tokens = tokenize(code);
        dbg!(&tokens);
        assert_eq!(tokens.len(), expected.len());
        for i in 0..tokens.len() {
            assert_eq!(tokens[i].token_type, expected[i].0);
            assert_eq!(tokens[i].value, expected[i].1.to_string());
        }
    }

    #[test]
    fn test_tokenizer_basics() {
        run_test(
            "if  3\nwhile",
            vec![
                (TokenType::Identifier, "if"),
                (TokenType::Integer, "3"),
                (TokenType::Identifier, "while"),
            ],
        );
    }

    #[test]
    fn test_tokenizer_location() {
        run_test_with_locations(
            "1 3 \n 2",
            vec![
                (TokenType::Integer, "1", (1, 1)),
                (TokenType::Integer, "3", (1, 3)),
                (TokenType::Integer, "2", (2, 2)),
            ],
        );
    }

    #[test]
    fn test_other_tokens() {
        run_test(
            "+ -1;\n(",
            vec![
                (TokenType::Operator, "+"),
                (TokenType::Operator, "-"),
                (TokenType::Integer, "1"),
                (TokenType::Punctuation, ";"),
                (TokenType::Punctuation, "("),
            ],
        );
    }

    #[test]
    fn test_comments() {
        run_test(
            "1//2+2 \n1#2+2",
            vec![(TokenType::Integer, "1"), (TokenType::Integer, "1")],
        );
    }

    #[test]
    fn test_multi_line_comments() {
        run_test(
            "/*\nMany lines\nof comment\ntext.\n*/\nprint_int(123)\n/* Another\ncomment. */\n",
            vec![
                (TokenType::Identifier, "print_int"),
                (TokenType::Punctuation, "("),
                (TokenType::Integer, "123"),
                (TokenType::Punctuation, ")"),
            ],
        );
    }
}
