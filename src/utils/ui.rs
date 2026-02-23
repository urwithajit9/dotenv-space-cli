// ============================================================================
// UI utilities
// ============================================================================

pub mod ui {
    use colored::*;
    use indicatif::{ProgressBar, ProgressStyle};

    /// Create a progress bar
    pub fn progress_bar(len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Print a boxed message
    pub fn print_box(title: &str, message: &str) {
        let width = 60;
        let border = "─".repeat(width - 4);

        println!("\n{}", format!("┌─{}─┐", border).cyan());
        println!(
            "{}",
            format!("│ {:<width$} │", title, width = width - 4).cyan()
        );

        if !message.is_empty() {
            for line in message.lines() {
                println!(
                    "{}",
                    format!("│ {:<width$} │", line, width = width - 4).cyan()
                );
            }
        }

        println!("{}\n", format!("└─{}─┘", border).cyan());
    }

    /// Print success message
    pub fn success(message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    /// Print error message
    pub fn error(message: &str) {
        println!("{} {}", "✗".red(), message);
    }

    /// Print warning message
    pub fn warning(message: &str) {
        println!("{} {}", "⚠️".yellow(), message);
    }

    /// Print info message
    pub fn info(message: &str) {
        println!("{} {}", "ℹ️".cyan(), message);
    }

    /// Create a separator line
    pub fn separator() {
        println!("{}", "─".repeat(60).dimmed());
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_progress_bar_creation() {
            let pb = progress_bar(100, "Testing");
            assert_eq!(pb.length().unwrap(), 100);
        }
    }
}
