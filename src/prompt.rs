//! Interactive prompts modeled after `@cliffy/prompt` (Input, Secret,
//! Select and Toggle).

/// Asks for a line of text. Returns an empty string when nothing is entered.
pub fn input(_message: &str) -> Result<String, String> {
    todo!()
}

/// Asks for a hidden value (passwords).
pub fn secret(_message: &str) -> Result<String, String> {
    todo!()
}

/// Lets the user pick one of `options` (name shown, value returned).
pub fn select(_message: &str, _options: &[(String, String)]) -> Result<String, String> {
    todo!()
}

/// Yes/no question.
pub fn toggle(_message: &str) -> Result<bool, String> {
    todo!()
}
