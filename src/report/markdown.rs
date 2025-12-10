use crate::prelude::*;
use crate::report::TestResults;

/// Generate Markdown report
pub fn generate_markdown_report(results: &TestResults) -> Fallible<String> {
    // Placeholder implementation
    let mut output = String::new();
    output.push_str("# Crater Experiment Report\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Total crates: {}\n", results.summary.total));
    output.push_str(&format!("- Regressions: {}\n", results.summary.regressed));
    output.push_str(&format!("- Fixes: {}\n", results.summary.fixed));
    output.push_str(&format!("- Broken: {}\n", results.summary.broken));
    output.push_str(&format!(
        "- Same build failures: {}\n",
        results.summary.same_build_fail
    ));
    output.push_str(&format!(
        "- Same test failures: {}\n",
        results.summary.same_test_fail
    ));
    output.push_str(&format!(
        "- Same test passes: {}\n",
        results.summary.same_test_pass
    ));
    output.push_str(&format!("- Skipped: {}\n", results.summary.skipped));

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::analyzer::Summary;
    use std::collections::HashMap;

    #[test]
    fn test_generate_markdown_report() {
        let results = TestResults {
            categories: HashMap::new(),
            summary: Summary {
                total: 100,
                regressed: 5,
                fixed: 3,
                broken: 2,
                same_build_fail: 10,
                same_test_fail: 5,
                same_test_pass: 70,
                same_test_skipped: 3,
                skipped: 2,
                unknown: 0,
                errors: 0,
            },
        };

        let markdown = generate_markdown_report(&results).unwrap();
        assert!(markdown.contains("# Crater Experiment Report"));
        assert!(markdown.contains("Total crates: 100"));
        assert!(markdown.contains("Regressions: 5"));
        assert!(markdown.contains("Fixes: 3"));
    }
}
