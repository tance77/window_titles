use std::{error::Error, fmt, process::Command};

use crate::{ConnectionTrait, Result};

const PREFIX: &str = r#"tell application "System Events""#;
const SUFFIX: &str = r#"to get the title of every window of every process"#;
const PERMISSION_ERROR: &str = "osascript is not allowed assistive access";

pub struct Connection;
impl ConnectionTrait for Connection {
	fn new() -> Result<Self> { Ok(Self) }
	fn window_titles(&self) -> Result<Vec<String>> {
		let arguments = &["-ss", "-e", &format!("{} {}", PREFIX, SUFFIX)];
		let command = Command::new("osascript").args(arguments).output();

		let command = match command {
			Ok(command_output) => command_output,
			Err(_) => return Err(WindowTitleError::ExecuteFailed.into()),
		};

		let error = String::from_utf8_lossy(&command.stderr);
		match error.contains(PERMISSION_ERROR) {
			true => Err(WindowTitleError::NoAccessibilityPermission.into()),
			false => Ok(split(&String::from_utf8_lossy(&command.stdout))),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum WindowTitleError {
	NoAccessibilityPermission,
	ExecuteFailed
}
impl fmt::Display for WindowTitleError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result<> {
		match self {
			WindowTitleError::NoAccessibilityPermission => write!(fmt, "Permission to use the accessibility API has not been granted"),
			WindowTitleError::ExecuteFailed => write!(fmt, "Failed to execute the command")
		}
	}
}
impl Error for WindowTitleError {}

fn split(string: &str) -> Vec<String> {
	let mut titles = Vec::new();
	let mut chars_iter = string.char_indices().peekable();
	while let Some((start, _)) = chars_iter.peek().cloned() {
		if string[start..].starts_with('"') {
			let mut title_chars = Vec::new();
			let mut found_end_quote = false;
			// Skip the initial quote
			chars_iter.next();
			while let Some((_, c)) = chars_iter.next() {
				// Check for an unescaped quote
				if c == '"' && title_chars.last() != Some(&'\\') {
					found_end_quote = true;
					break;
				}
				title_chars.push(c);
			}
			if found_end_quote {
				// Convert characters to String, handling escaped characters
				let title: String = title_chars.into_iter().collect::<String>().replace("\\\"", "\"");
				titles.push(title);
			}
		} else {
			// Move to the next character if the current one isn't a quote
			chars_iter.next();
		}
	}
	titles
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_split() {
		let string = r#"{{}, {"0"}, {"1", "2"}}"#;
		assert_eq!(split(string), &["0", "1", "2"]);
	}

	#[test]
	fn test_split_handles_no_end_quote() {
		let input = r#"{"\" - Brave", "1", "2"}"#;
		assert_eq!(split(input), vec![r#"" - Brave"#, "1", "2"]);
	}

	#[test]
	fn emoji_test(){
		let input = r#"{"👋"}, {"😾"}, {"🤮", "🎃"}"#;
		assert_eq!(split(input), vec![r#"👋"#, r#"😾"#, r#"🤮"#, r#"🎃"#]);
	}
}
