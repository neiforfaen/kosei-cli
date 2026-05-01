use colored::Colorize;
use similar::{ChangeTag, TextDiff};

pub fn print_diff(filename: &str, original: &str, updated: &str) {
    if original == updated {
        return;
    }

    let diff = TextDiff::from_lines(original, updated);
    let changes: Vec<_> = diff.iter_all_changes().collect();

    let has_changes = changes
        .iter()
        .any(|change| matches!(change.tag(), ChangeTag::Delete | ChangeTag::Insert));

    if !has_changes {
        return;
    }

    println!("{}:", filename);
    for change in changes {
        match change.tag() {
            ChangeTag::Delete => {
                println!("  {}", format!("- {}", change.value().trim_end()).red());
            }
            ChangeTag::Insert => {
                println!("  {}", format!("+ {}", change.value().trim_end()).green());
            }
            ChangeTag::Equal => {}
        }
    }
    println!();
}
