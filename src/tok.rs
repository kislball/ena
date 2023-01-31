pub const ONCE_OPEN: char = '(';
pub const ONCE_CLOSE: char = ')';
pub const UNIQUE_OPEN: char = '{';
pub const UNIQUE_CLOSE: char = '}';
pub const STRING_QUOTES: char = '"';
pub const ESCAPE_CHAR: char = '\'';
pub const ATOM_CHAR: char = ':';
pub const COMMENT_SYMBOL: char = '#';
pub const STRING_ESCAPE_CHAR: char = '\\';

fn is_id_beginning(ch: char) -> bool {
    !ch.is_numeric()
        && !ch.is_whitespace()
        && ch != STRING_QUOTES
        && ch != ESCAPE_CHAR
        && ch != ATOM_CHAR
}

#[derive(Debug)]
pub enum TokenizerErrorInner {
    UnknownToken(char),
    UnclosedString,
    UnexpectedEOF,
    TooManyDotsInNumber,
    CannotEscapeNonRegularId,
    UnexpectedEscapeChar,
    UnexpectedAtomChar,
    InvalidEscape,
}

#[derive(Debug)]
pub struct TokenizerError(pub usize, pub TokenizerErrorInner);

#[derive(Debug)]
pub enum TokenInner {
    Identifier(String),
    EscapedIdentifier(String),
    Atom(String),
    Comment(String),
    String(String),
    Number(f64),
    Keyword(KeywordType),
    OnceOpen,
    OnceEscapedOpen,
    OnceClose,
    UniqueEscapedOpen,
    UniqueOpen,
    UniqueClose,
}

pub fn is_closer(open: &TokenInner, close: &TokenInner) -> Option<bool> {
    match *open {
        TokenInner::OnceOpen => match *close {
            TokenInner::OnceClose => Some(true),
            _ => Some(false),
        },
        TokenInner::UniqueOpen => match *close {
            TokenInner::UniqueClose => Some(true),
            _ => Some(false),
        },
        TokenInner::OnceEscapedOpen => match *close {
            TokenInner::OnceClose => Some(true),
            _ => Some(false),
        },
        TokenInner::UniqueEscapedOpen => match *close {
            TokenInner::UniqueClose => Some(true),
            _ => Some(false),
        },
        _ => None,
    }
}

#[derive(Debug)]
pub struct Token(pub usize, pub TokenInner);

#[derive(Debug, Copy, Clone)]
pub enum KeywordType {
    If,
    While,
    Return,
    True,
    False,
    Null,
    None,
}

impl From<&str> for KeywordType {
    fn from(value: &str) -> Self {
        if value == "if" {
            KeywordType::If
        } else if value == "while" {
            KeywordType::While
        } else if value == "return" {
            KeywordType::Return
        } else if value == "true" {
            KeywordType::True
        } else if value == "false" {
            KeywordType::False
        } else if value == "null" {
            KeywordType::Null
        } else {
            KeywordType::None
        }
    }
}

enum IdentifierType {
    Escaped,
    Regular,
    Atom,
}

pub struct Tokenizer {
    pub tokens: Vec<Token>,
    pub str: String,
    at: usize,
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer {
            tokens: vec![],
            str: String::from(""),
            at: 0,
        }
    }

    pub fn clean(&mut self) {
        self.at = 0;
        self.str = String::from("");
        self.tokens = vec![];
    }

    pub fn parse(&mut self, str: String) -> Result<&mut Vec<Token>, TokenizerError> {
        self.clean();
        self.str = str;
        self.str.push(' '); // needs a whitespace for ids and numbers to work
        let en: Vec<char> = self.str.chars().enumerate().map(|x| x.1).collect();

        loop {
            let c = match en.get(self.at) {
                Some(ch) => *ch,
                None => {
                    break;
                }
            };
            if c == ATOM_CHAR {
                let next = match en.get(self.at + 1) {
                    Some(ch) => {
                        self.at += 1;
                        *ch
                    }
                    None => {
                        return Err(TokenizerError(
                            self.at,
                            TokenizerErrorInner::UnexpectedEscapeChar,
                        ));
                    }
                };

                if is_id_beginning(next) {
                    if let Some(err) = self.parse_id(&en, IdentifierType::Atom) {
                        return Err(err);
                    }
                } else {
                    return Err(TokenizerError(
                        self.at,
                        TokenizerErrorInner::UnexpectedAtomChar,
                    ));
                }
            } else if c == COMMENT_SYMBOL {
                let mut comment_data = String::new();
                self.at += 1;
                loop {
                    let c = match en.get(self.at) {
                        Some(ch) => *ch,
                        None => {
                            break;
                        }
                    };
                    if c == '\n' {
                        self.tokens
                            .push(Token(self.at, TokenInner::Comment(comment_data)));
                        break;
                    } else {
                        comment_data.push(c);
                        self.at += 1;
                    }
                }
            } else if c == ONCE_OPEN {
                self.tokens.push(Token(self.at, TokenInner::OnceOpen));
                self.at += 1;
            } else if c == ONCE_CLOSE {
                self.tokens.push(Token(self.at, TokenInner::OnceClose));
                self.at += 1;
            } else if c == UNIQUE_OPEN {
                self.tokens.push(Token(self.at, TokenInner::UniqueOpen));
                self.at += 1;
            } else if c == UNIQUE_CLOSE {
                self.tokens.push(Token(self.at, TokenInner::UniqueClose));
                self.at += 1;
            } else if is_id_beginning(c) {
                if let Some(err) = self.parse_id(&en, IdentifierType::Regular) {
                    return Err(err);
                }
            } else if c == ESCAPE_CHAR {
                let next = match en.get(self.at + 1) {
                    Some(ch) => {
                        self.at += 1;
                        *ch
                    }
                    None => {
                        return Err(TokenizerError(
                            self.at,
                            TokenizerErrorInner::UnexpectedEscapeChar,
                        ));
                    }
                };

                if next == ONCE_OPEN {
                    self.tokens
                        .push(Token(self.at, TokenInner::OnceEscapedOpen));
                    self.at += 1;
                } else if next == UNIQUE_OPEN {
                    self.tokens
                        .push(Token(self.at, TokenInner::UniqueEscapedOpen));
                    self.at += 1;
                } else if is_id_beginning(next) {
                    if let Some(err) = self.parse_id(&en, IdentifierType::Escaped) {
                        return Err(err);
                    }
                } else {
                    return Err(TokenizerError(
                        self.at,
                        TokenizerErrorInner::UnexpectedEscapeChar,
                    ));
                }
            } else if c.is_whitespace() {
                self.at += 1;
            } else if c.is_numeric() {
                self.parse_number(&en);
            } else if c == STRING_QUOTES {
                if let Some(err) = self.parse_str(&en) {
                    return Err(err);
                }
            } else {
                return Err(TokenizerError(
                    self.at,
                    TokenizerErrorInner::UnknownToken(c),
                ));
            }
        }

        Ok(&mut self.tokens)
    }

    fn parse_number(&mut self, en: &[char]) -> Option<TokenizerError> {
        let c = match en.get(self.at) {
            Some(ch) => *ch,
            None => {
                return Some(TokenizerError(self.at, TokenizerErrorInner::UnexpectedEOF));
            }
        };
        let mut str = String::new();
        str.push(c);
        let begin = self.at;
        self.at += 1;
        let mut had_dot = false;

        loop {
            let c = match en.get(self.at) {
                Some(ch) => *ch,
                None => {
                    return Some(TokenizerError(self.at, TokenizerErrorInner::UnexpectedEOF));
                }
            };

            if c == '_' {
                self.at += 1;
            } else if c == '.' {
                if had_dot {
                    return Some(TokenizerError(
                        self.at,
                        TokenizerErrorInner::TooManyDotsInNumber,
                    ));
                } else {
                    had_dot = true;
                    str.push('.');
                }
                self.at += 1;
            } else if c.is_numeric() {
                str.push(c);
                self.at += 1;
            } else if c.is_whitespace() {
                break;
            } else {
                return Some(TokenizerError(
                    self.at,
                    TokenizerErrorInner::UnknownToken(c),
                ));
            }
        }

        let token = Token(begin, TokenInner::Number(str.parse::<f64>().unwrap()));

        self.tokens.push(token);
        self.at += 1;
        None
    }

    fn parse_id(&mut self, en: &[char], id_type: IdentifierType) -> Option<TokenizerError> {
        let c = match en.get(self.at) {
            Some(ch) => *ch,
            None => {
                return Some(TokenizerError(self.at, TokenizerErrorInner::UnexpectedEOF));
            }
        };
        let mut str = String::new();
        str.push(c);
        let begin = self.at;
        self.at += 1;

        loop {
            let c = match en.get(self.at) {
                Some(ch) => *ch,
                None => {
                    return Some(TokenizerError(self.at, TokenizerErrorInner::UnexpectedEOF));
                }
            };

            if !c.is_whitespace() {
                str.push(c);
                self.at += 1;
            } else {
                self.at += 1;
                break;
            }
        }

        let into_kw = KeywordType::from(str.as_str());
        match into_kw {
            KeywordType::None => match id_type {
                IdentifierType::Regular => {
                    self.tokens.push(Token(begin, TokenInner::Identifier(str)))
                }
                IdentifierType::Escaped => self
                    .tokens
                    .push(Token(begin, TokenInner::EscapedIdentifier(str))),
                IdentifierType::Atom => self.tokens.push(Token(begin, TokenInner::Atom(str))),
            },
            _ => {
                if !matches!(id_type, IdentifierType::Regular) {
                    return Some(TokenizerError(
                        self.at,
                        TokenizerErrorInner::CannotEscapeNonRegularId,
                    ));
                }
                self.tokens.push(Token(begin, TokenInner::Keyword(into_kw)))
            }
        };

        None
    }

    fn parse_str(&mut self, en: &[char]) -> Option<TokenizerError> {
        let begin = self.at;
        let mut str = String::new();
        self.at += 1;

        loop {
            let c = match en.get(self.at) {
                Some(ch) => *ch,
                None => {
                    return Some(TokenizerError(self.at, TokenizerErrorInner::UnclosedString));
                }
            };

            if c == STRING_QUOTES {
                self.at += 1;
                break;
            } else {
                if c == STRING_ESCAPE_CHAR {
                    self.at += 1;
                    let next = match en.get(self.at) {
                        Some(ch) => *ch,
                        None => {
                            return Some(TokenizerError(
                                self.at,
                                TokenizerErrorInner::InvalidEscape,
                            ));
                        }
                    };

                    if next == '\\' {
                        str.push('\\');
                    } else if next == 'n' {
                        str.push('\n');
                    } else if next == 'r' {
                        str.push('\r');
                    } else if next == 't' {
                        str.push('\t');
                    } else if next == STRING_QUOTES {
                        str.push(STRING_QUOTES);
                    } else {
                        return Some(TokenizerError(self.at, TokenizerErrorInner::InvalidEscape));
                    }
                } else {
                    str.push(c);
                }

                self.at += 1
            }
        }

        self.tokens.push(Token(begin, TokenInner::String(str)));
        None
    }
}

pub fn show_tokens(tokens: &Vec<Token>) {
    for c in tokens {
        println!("{c:?}");
    }
}
