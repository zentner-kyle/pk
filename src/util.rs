
pub fn some_char<F>(opt_char: Option<char>, f: F) -> bool where F: Fn(char) -> bool {
    if let Some(c) = opt_char {
        f(c)
    } else {
        false
    }
}
