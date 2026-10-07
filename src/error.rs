// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(file: Option<&str>, source: &str, offset: usize) -> Location {
        let before = &source[..offset.min(source.len())];
        let line = before.matches('\n').count() + 1;
        let column = match before.rfind('\n') {
            Some(i) => before[i + 1..].chars().count() + 1,
            None => before.chars().count() + 1,
        };
        Location {
            file: file.map(str::to_string),
            line,
            column,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.file {
            Some(file) => write!(f, "{}:{}:{}", file, self.line, self.column),
            None => write!(f, "{}:{}", self.line, self.column),
        }
    }
}

fn write_location(f: &mut fmt::Formatter<'_>, location: &Option<Location>) -> fmt::Result {
    match location {
        Some(location) => write!(f, "{}: ", location),
        None => Ok(()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub offset: Option<usize>,
    pub location: Option<Location>,
}

impl ParseError {
    pub fn locate(mut self, file: Option<&str>, source: &str) -> ParseError {
        if let Some(offset) = self.offset {
            self.location = Some(Location::new(file, source, offset));
        }
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_location(f, &self.location)?;
        match (&self.location, self.offset) {
            (None, Some(offset)) => write!(f, "parse error at offset {}: {}", offset, self.message),
            _ => write!(f, "parse error: {}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckError {
    pub theorem: String,
    pub step_index: usize,
    pub offset: usize,
    pub message: String,
    pub location: Option<Location>,
}

impl CheckError {
    pub fn locate(mut self, file: Option<&str>, source: &str) -> CheckError {
        self.location = Some(Location::new(file, source, self.offset));
        self
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_location(f, &self.location)?;
        write!(
            f,
            "proof error in theorem `{}`, step {}: {}",
            self.theorem,
            self.step_index + 1,
            self.message
        )
    }
}

impl std::error::Error for CheckError {}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadError {
    pub message: String,
    pub location: Option<Location>,
}

impl LoadError {
    pub fn new(message: impl Into<String>) -> LoadError {
        LoadError {
            message: message.into(),
            location: None,
        }
    }

    pub fn at(mut self, file: Option<&str>, source: &str, offset: usize) -> LoadError {
        self.location = Some(Location::new(file, source, offset));
        self
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_location(f, &self.location)?;
        write!(f, "error: {}", self.message)
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleError {
    pub rule: String,
    pub reason: String,
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rule `{}` failed: {}", self.rule, self.reason)
    }
}

impl std::error::Error for RuleError {}

#[derive(Debug)]
pub enum AlifError {
    Parse(ParseError),
    Check(CheckError),
    Load(LoadError),
}

impl fmt::Display for AlifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlifError::Parse(e) => write!(f, "{}", e),
            AlifError::Check(e) => write!(f, "{}", e),
            AlifError::Load(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AlifError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AlifError::Parse(e) => Some(e),
            AlifError::Check(e) => Some(e),
            AlifError::Load(e) => Some(e),
        }
    }
}

impl From<ParseError> for AlifError {
    fn from(e: ParseError) -> Self {
        AlifError::Parse(e)
    }
}

impl From<CheckError> for AlifError {
    fn from(e: CheckError) -> Self {
        AlifError::Check(e)
    }
}

impl From<LoadError> for AlifError {
    fn from(e: LoadError) -> Self {
        AlifError::Load(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_first_line() {
        let loc = Location::new(None, "abc\ndef", 1);
        assert_eq!((loc.line, loc.column), (1, 2));
    }

    #[test]
    fn location_later_line() {
        let loc = Location::new(Some("f.alif"), "abc\ndef", 5);
        assert_eq!((loc.line, loc.column), (2, 2));
        assert_eq!(loc.to_string(), "f.alif:2:2");
    }

    #[test]
    fn location_counts_chars_not_bytes() {
        let loc = Location::new(None, "ёж x", "ёж ".len());
        assert_eq!(loc.column, 4);
    }

    #[test]
    fn location_offset_past_end_is_clamped() {
        let loc = Location::new(None, "ab", 99);
        assert_eq!((loc.line, loc.column), (1, 3));
    }

    #[test]
    fn check_error_display() {
        let e = CheckError {
            theorem: "t".to_string(),
            step_index: 2,
            offset: 0,
            message: "hypothesis not found".to_string(),
            location: None,
        };
        let text = e.to_string();
        assert!(text.contains("theorem `t`"));
        assert!(text.contains("step 3"));
        assert!(text.contains("hypothesis not found"));
    }

    #[test]
    fn check_error_display_with_location() {
        let e = CheckError {
            theorem: "t".to_string(),
            step_index: 0,
            offset: 4,
            message: "m".to_string(),
            location: None,
        }
        .locate(Some("a.alif"), "ab\ncdef");
        assert!(e.to_string().starts_with("a.alif:2:2: "));
    }

    #[test]
    fn parse_error_display_with_offset() {
        let e = ParseError {
            message: "unexpected token".to_string(),
            offset: Some(10),
            location: None,
        };
        assert!(e.to_string().contains("offset 10"));
    }

    #[test]
    fn parse_error_display_with_location() {
        let e = ParseError {
            message: "unexpected token".to_string(),
            offset: Some(3),
            location: None,
        }
        .locate(None, "ab\ncd");
        assert_eq!(e.to_string(), "2:1: parse error: unexpected token");
    }

    #[test]
    fn rule_error_display() {
        let e = RuleError {
            rule: "AndIntro".to_string(),
            reason: "wrong arity".to_string(),
        };
        assert!(e.to_string().contains("AndIntro"));
    }

    #[test]
    fn load_error_display() {
        let e = LoadError::new("duplicate").at(Some("a.alif"), "x\ny", 2);
        assert_eq!(e.to_string(), "a.alif:2:1: error: duplicate");
    }
}
