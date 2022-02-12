/**
 * Remove the first and last characters from a string.
 */
pub fn trim_ends(string: String) -> String {
    let mut chars = string.chars();
    chars.next();
    chars.next_back();
    String::from(chars.as_str())
}