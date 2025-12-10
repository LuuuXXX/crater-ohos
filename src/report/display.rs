// Allow dead code for Phase 3 functions not yet fully connected
#![allow(dead_code)]

use crate::results::TestResult;

/// Color codes for terminal output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Reset,
}

impl Color {
    pub fn to_ansi(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Trait for getting the display color of a result
pub trait ResultColor {
    fn color(&self) -> Color;
}

impl ResultColor for TestResult {
    fn color(&self) -> Color {
        match self {
            TestResult::TestPass => Color::Green,
            TestResult::TestSkipped => Color::Yellow,
            TestResult::Skipped => Color::Yellow,
            TestResult::BuildFail(_) => Color::Red,
            TestResult::TestFail(_) => Color::Red,
            TestResult::PrepareFail(_) => Color::Magenta,
            TestResult::BrokenCrate(_) => Color::Magenta,
            TestResult::Error => Color::Red,
        }
    }
}

/// Trait for getting a short display name
pub trait ResultName {
    fn name(&self) -> &'static str;
}

impl ResultName for TestResult {
    fn name(&self) -> &'static str {
        match self {
            TestResult::TestPass => "test-pass",
            TestResult::TestSkipped => "test-skipped",
            TestResult::Skipped => "skipped",
            TestResult::BuildFail(_) => "build-fail",
            TestResult::TestFail(_) => "test-fail",
            TestResult::PrepareFail(_) => "prepare-fail",
            TestResult::BrokenCrate(_) => "broken",
            TestResult::Error => "error",
        }
    }
}

/// Format a result with color
pub fn format_result(result: &TestResult, use_color: bool) -> String {
    if use_color {
        format!(
            "{}{}{}",
            result.color().to_ansi(),
            result.name(),
            Color::Reset.to_ansi()
        )
    } else {
        result.name().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::FailureReason;

    #[test]
    fn test_result_colors() {
        assert_eq!(TestResult::TestPass.color(), Color::Green);
        assert_eq!(TestResult::TestSkipped.color(), Color::Yellow);
        assert_eq!(
            TestResult::BuildFail(FailureReason::Unknown).color(),
            Color::Red
        );
        assert_eq!(
            TestResult::TestFail(FailureReason::Timeout).color(),
            Color::Red
        );
    }

    #[test]
    fn test_result_names() {
        assert_eq!(TestResult::TestPass.name(), "test-pass");
        assert_eq!(TestResult::TestSkipped.name(), "test-skipped");
        assert_eq!(
            TestResult::BuildFail(FailureReason::Unknown).name(),
            "build-fail"
        );
    }

    #[test]
    fn test_format_result_no_color() {
        let result = TestResult::TestPass;
        assert_eq!(format_result(&result, false), "test-pass");
    }

    #[test]
    fn test_format_result_with_color() {
        let result = TestResult::TestPass;
        let formatted = format_result(&result, true);
        assert!(formatted.contains("test-pass"));
        assert!(formatted.contains("\x1b["));
    }

    #[test]
    fn test_color_ansi() {
        assert_eq!(Color::Red.to_ansi(), "\x1b[31m");
        assert_eq!(Color::Green.to_ansi(), "\x1b[32m");
        assert_eq!(Color::Reset.to_ansi(), "\x1b[0m");
    }
}
