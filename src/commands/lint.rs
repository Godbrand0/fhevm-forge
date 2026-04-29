use anyhow::Result;
use colored::Colorize;
use crate::linter::{Linter, rules::Severity, reporter};

pub async fn run(path: &str, fix: bool, ignore: Vec<String>, list_rules: bool) -> Result<()> {
    let linter = Linter::new().ignore(ignore);

    if list_rules {
        println!("{}", "Available lint rules:".cyan().bold());
        for id in linter.rule_ids() {
            println!("  {}", id);
        }
        return Ok(());
    }

    println!("\n{} {}\n", "Analyzing".cyan().bold(), path.yellow());

    let errors = linter.analyze_path(path)?;

    if errors.is_empty() {
        println!("{}", "✅ No FHEVM issues found.".green().bold());
        return Ok(());
    }

    let mut by_file: std::collections::BTreeMap<String, Vec<&crate::linter::rules::LintError>> =
        std::collections::BTreeMap::new();

    for e in &errors {
        by_file.entry(e.file.clone()).or_default().push(e);
    }

    for (file, file_errors) in &by_file {
        println!("  {}", file.cyan());
        println!("  {}", "─".repeat(60).dimmed());

        for e in file_errors {
            println!("  {}", reporter::format_error(e));
            if let Some(snippet) = &e.snippet {
                println!("              {}", snippet.dimmed());
            }
        }
        println!();
    }

    reporter::print_summary(&errors);

    if fix {
        println!("\n{}", "Auto-fix mode is not yet implemented for all rules.".yellow());
        println!("Safe auto-fixes (FHEVM-001, FHEVM-003) will be added in v0.2.0.");
    } else {
        println!(
            "\nRun {} to attempt auto-fix of safe issues.",
            "fhevm-forge lint --fix".cyan()
        );
    }

    let error_count = errors.iter().filter(|e| e.severity == Severity::Error).count();
    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
