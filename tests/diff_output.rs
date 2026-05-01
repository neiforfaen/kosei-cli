// Integration tests for the print_diff function
// These tests verify that print_diff handles various scenarios correctly.
// They test that the function doesn't panic and correctly processes different types of content.

#[test]
fn test_diff_no_changes() {
    let original = "line 1\nline 2\nline 3\n";
    let updated = "line 1\nline 2\nline 3\n";

    // Should not panic with identical content
    kosei_cli::print_diff("test.txt", original, updated);
}

#[test]
fn test_diff_displays_removed_lines() {
    let original = "line 1\nline 2\nline 3\n";
    let updated = "line 1\nline 3\n";

    // Should not panic when content is removed
    kosei_cli::print_diff("test.txt", original, updated);
}

#[test]
fn test_diff_displays_added_lines() {
    let original = "line 1\nline 3\n";
    let updated = "line 1\nline 2\nline 3\n";

    // Should not panic when content is added
    kosei_cli::print_diff("test.txt", original, updated);
}

#[test]
fn test_diff_handles_multiline() {
    let original = "line 1\nline 2\nline 3\nline 4\n";
    let updated = "line 1\nline 2 modified\nline 3\nline 4\n";

    // Should not panic with modified multiline content
    kosei_cli::print_diff("test.txt", original, updated);
}

#[test]
fn test_diff_empty_to_content() {
    let original = "";
    let updated = "new line 1\nnew line 2\n";

    // Should not panic when adding content to empty string
    kosei_cli::print_diff("test.txt", original, updated);
}

#[test]
fn test_diff_content_to_empty() {
    let original = "line 1\nline 2\n";
    let updated = "";

    // Should not panic when removing all content
    kosei_cli::print_diff("test.txt", original, updated);
}
