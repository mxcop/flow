/**
 * Remove the first and last characters from a string.
 */
pub fn trim_ends(string: String) -> String {
    let mut chars = string.chars();
    chars.next();
    chars.next_back();
    String::from(chars.as_str())
}

/**
 * Uppercase the first letter of a string.
 */
pub fn fuppercase(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}