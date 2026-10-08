use core_common::{CoreError, CoreResult};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Number(f64), Identifier(String), Plus, Minus, Star, Slash, Percent, Caret,
    LeftParen, RightParen, End,
}

pub(crate) fn tokenize(input: &str) -> CoreResult<Vec<Token>> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }

        if c.is_ascii_digit() || c == '.' {
            let start = i;
            let mut has_digit = false;
            while i < chars.len() && chars[i].is_ascii_digit() { has_digit = true; i += 1; }
            if i < chars.len() && chars[i] == '.' {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() { has_digit = true; i += 1; }
            }
            if !has_digit { return Err(CoreError::Parse("invalid number".into())); }

            if i < chars.len() && matches!(chars[i], 'e' | 'E') {
                i += 1;
                if i < chars.len() && matches!(chars[i], '+' | '-') { i += 1; }
                let exponent_start = i;
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                if exponent_start == i { return Err(CoreError::Parse("invalid exponent".into())); }
            }

            let text: String = chars[start..i].iter().collect();
            let value = text.parse::<f64>()
                .map_err(|_| CoreError::Parse(format!("invalid number: {text}")))?;
            if !value.is_finite() {
                return Err(CoreError::Parse(format!("number is not finite: {text}")));
            }
            tokens.push(Token::Number(value));
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
            tokens.push(Token::Identifier(chars[start..i].iter().collect::<String>().to_lowercase()));
            continue;
        }

        tokens.push(match c {
            '+' => Token::Plus, '-' => Token::Minus, '*' => Token::Star, '/' => Token::Slash,
            '%' => Token::Percent, '^' => Token::Caret, '(' => Token::LeftParen, ')' => Token::RightParen,
            _ => return Err(CoreError::Parse(format!("unexpected character: {c}"))),
        });
        i += 1;
    }

    tokens.push(Token::End);
    Ok(tokens)
}
