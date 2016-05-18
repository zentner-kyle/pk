use std;

use unicode_xid::UnicodeXID;

use util::{some_char};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error<'a> {
    pub rest: &'a str
}

impl<'a> Error<'a> {
    fn new(rest: &str) -> Error {
        Error {
            rest: rest,
        }
    }
}

fn start_and_continue<F, G>(src: &str, f: F, g: G) -> Result<&str, Error>
    where F: Fn(char) -> bool,
          G: Fn(char) -> bool {
    let mut rest;
    let mut cs = src.chars();
    if some_char(cs.next(), f) {
        rest = cs.as_str();
    } else {
        return Err(Error::new(src));
    }
    while some_char(cs.next(), &g) {
        rest = cs.as_str();
    }
    return Ok(rest);
}

pub fn identifier(src: &str) -> Result<&str, Error> {
    start_and_continue(src, UnicodeXID::is_xid_start, UnicodeXID::is_xid_continue)
}

pub fn decimal_integer(src: &str) -> Result<&str, Error> {
    start_and_continue(src, |c| c.is_digit(10) && c != '0', |c| c.is_digit(10))
}

pub fn whitespace(src: &str) -> Result<&str, Error> {
    start_and_continue(src, char::is_whitespace, char::is_whitespace)
}

pub fn integer(src: &str) -> Result<&str, Error> {
    let mut rest;
    let mut cs = src.chars();

    if !some_char(cs.next(), |c| c == '+' || c == '-') {
        cs = src.chars();
    }

    let base;

    let first = cs.next();
    let start_second = cs.as_str();
    let second = cs.next();
    let start_third = cs.as_str();
    let third = cs.next();
    let start_fourth = cs.as_str();
    let fourth = cs.next();

    if first == Some('0') && second == Some('x') && some_char(third, |c| c.is_digit(16)) {
        base = 16;
        rest = start_third;
    } else if first == Some('0') && second == Some('o') && some_char(third, |c| c.is_digit(8)) {
        base = 8;
        rest = start_third;
    } else if first == Some('0') && second == Some('b') && some_char(third, |c| c.is_digit(2)) {
        base = 2;
        rest = start_third;
    } else if first == Some('0') && some_char(second, |c| c.is_digit(10)) {
        if !some_char(second, |c| c.is_digit(8)) {
            return Err(Error::new(src));
        } else {
            base = 8;
            rest = start_second;
        }
    } else if let (Some(f), true) = (first.and_then(|c| c.to_digit(10)), second == Some('x')) {
        if some_char(third, |c| c.is_digit(f)) {
            base = f;
            rest = start_second;
        } else {
            base = 10;
            rest = src;
        }
    } else if let (Some(f), Some(s), true) =
                  (first.and_then(|c| c.to_digit(10)),
                   second.and_then(|c| c.to_digit(10)),
                   third == Some('x')) {
        let base_val = f * 10 + s;
        if base_val <= 36 && some_char(fourth, |c| c.is_digit(base_val)) {
            base = base_val;
            rest = start_fourth;
        } else {
            base = 10;
            rest = src;
        }
    } else if some_char(first, |c| c.is_digit(10)) && !some_char(second, |c| c.is_digit(10)) {
        return Ok(start_second);
    } else {
        base = 10;
        rest = src;
    }
    cs = rest.chars();
    loop {
        if let Some(c) = cs.next() {
            if c.is_digit(base) {
                rest = cs.as_str();
                continue;
            } else if c.is_digit(10) {
                // For example, a 9 in a base 8 number.
                return Err(Error::new(rest));
            }
        }
        return Ok(rest);
    }
}

pub fn float(src: &str) -> Result<&str, Error> {
    let mut cs = src.chars();
    let mut rest = src;

    if some_char(cs.next(), |c| c == '+' || c == '-') {
        rest = cs.as_str();
    } else {
        cs = src.chars();
    }

    let mut saw_decimal_point = false;
    let mut saw_exponent = false;
    let mut saw_numeral = false;
    loop {
        if let Some(c) = cs.next() {
            if c.is_digit(10) {
                rest = cs.as_str();
                saw_numeral = true;
                continue;
            } else if c == '.' {
                if saw_decimal_point || saw_exponent {
                    break;
                } else {
                    saw_decimal_point = true;
                    rest = cs.as_str();
                    continue;
                }
            } else if c == 'e' || c == 'E' {
                if saw_numeral {
                    if !some_char(cs.next(), |c| c.is_digit(10)) || saw_exponent {
                        break;
                    } else {
                        saw_exponent = true;
                        rest = cs.as_str();
                        continue;
                    }
                } else {
                    return Err(Error::new(src));
                }
            }
        }
        break;
    }
    if saw_numeral && (saw_decimal_point || saw_exponent) {
        return Ok(rest);
    } else {
        return Err(Error::new(src));
    }
}

pub fn char_escape(src: &str) -> Result<&str, Error> {
    let mut rest;
    let mut cs = src.chars();
    rest = cs.as_str();
    if let Some(c) = cs.next() {
        match c {
            '0' | 'n' | 't' | '\\' | '"' | '\'' => {
                return Ok(cs.as_str());
            },
            'x' => {
                rest = cs.as_str();
                if some_char(cs.next(), |c| c.is_digit(16)) {
                    rest = cs.as_str();
                    if some_char(cs.next(), |c| c.is_digit(16)) {
                        return Ok(cs.as_str());
                    }
                }
            },
            'u' => {
                rest = cs.as_str();
                if some_char(cs.next(), |c| c == '{') {
                    for i in 0..8 {
                        rest = cs.as_str();
                        match cs.next() {
                            // This matches ECMAScript.
                            Some('}') if i != 0 => {
                                return Ok(cs.as_str());
                            },
                            Some(c) if c.is_digit(16) => {
                                if i < 6 {
                                    continue;
                                } else {
                                    break;
                                }
                            },
                            _ => {}
                        }
                        break;
                    }
                }
            },
            _ => {}
        }
    }
    return Err(Error::new(rest));
}

pub fn string(src: &str) -> Result<&str, Error> {
    let mut rest;
    let mut cs = src.chars();
    let first_char;
    match cs.next() {
        Some(c@'"') | Some(c@'\'') => {
            first_char = c;
            rest = cs.as_str();
        },
        _ => {
            return Err(Error::new(src));
        }
    }
    loop {
        rest = cs.as_str();
        if let Some(c) = cs.next() {
            match c {
                _ if c == first_char => {
                    return Ok(cs.as_str());
                },
                '\\' => {
                    cs = try!(char_escape(cs.as_str())).chars();
                    rest = cs.as_str();
                },
                _ => {},
            }
        } else {
            break;
        }
    }
    return Err(Error::new(rest));
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_lexes_identifiers() {
        assert!(::lex::identifier("test").is_ok());
        assert_eq!(::lex::identifier("test"), Ok(""));
        assert_eq!(::lex::identifier("test this"), Ok(" this"));
    }

    #[test]
    fn it_lexes_decimal_integers() {
        assert!(::lex::decimal_integer("1").is_ok());
        assert_eq!(::lex::decimal_integer("10"), Ok(""));
    }

    #[test]
    fn it_lexes_integers() {
        assert!(::lex::integer("1").is_ok());
        assert!(::lex::integer("-0").is_ok());
        assert!(::lex::integer("+0").is_ok());
        assert_eq!(::lex::integer("10"), Ok(""));
        assert_eq!(::lex::integer("0x"), Ok("x"));
        assert_eq!(::lex::integer("0xa"), Ok(""));
        assert_eq!(::lex::integer("0xaA"), Ok(""));
        assert_eq!(::lex::integer("0o"), Ok("o"));
        assert_eq!(::lex::integer("0b"), Ok("b"));
        assert_eq!(::lex::integer("16xff"), Ok(""));
        assert_eq!(::lex::integer("40x0"), Ok("x0"));
    }

    #[test]
    fn it_lexes_floats() {
        assert!(::lex::float("1").is_err());
        assert!(::lex::float("+1").is_err());
        assert!(::lex::float("-1").is_err());
        assert_eq!(::lex::float("1.0"), Ok(""));
        assert_eq!(::lex::float("1.e10"), Ok(""));
        assert_eq!(::lex::float("1.e"), Ok("e"));
        assert!(::lex::float(".e1").is_err());
        assert!(::lex::float(".e").is_err());
    }

    #[test]
    fn it_lexes_whitespace() {
        assert!(::lex::whitespace(" ").is_ok());
        assert_eq!(::lex::whitespace(" \t\n1"), Ok("1"));
    }

    #[test]
    fn it_lexes_strings() {
        assert!(::lex::string("\"test\"").is_ok());
        assert_eq!(::lex::string("\"test\" rest"), Ok(" rest"));
        assert!(::lex::string("\"\\u{}\"").is_err());
        assert!(::lex::string("\"\\u{0}\"").is_ok());
        assert!(::lex::string("\"\\u{000000}\"").is_ok());
        assert!(::lex::string("\"\\u{0000000}\"").is_err());
        assert!(::lex::string("\"\\u{00000000}\"").is_err());
    }
}
