use anyhow::Result;
use colored::Colorize;
use crate::linter::{Linter, rules::Severity};

pub async fn run(path: &str, fix: bool) -> Result<()> {
    println!("\n{} {}\n", "Analyzing".cyan().bold(), path.yellow());

    let linter = Linter::new();
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
            let severity_str = match e.severity {
                Severity::Error   => "ERROR".red().bold(),
                Severity::Warning => "WARN ".yellow().bold(),
            };
            println!(
                "  [{}] {}  Line {:>4}  — {}",
                e.rule_id.dimmed(),
                severity_str,
                e.line,
                e.message,
            );
            if let Some(snippet) = &e.snippet {
                println!("              {}", snippet.dimmed());
            }
        }
        println!();
    }

    let error_count   = errors.iter().filter(|e| e.severity == Severity::Error).count();
    let warning_count = errors.iter().filter(|e| e.severity == Severity::Warning).count();

    println!(
        "{}: {} error{}, {} warning{}",
        "Summary".bold(),
        error_count.to_string().red(),
        if error_count == 1 { "" } else { "s" },
        warning_count.to_string().yellow(),
        if warning_count == 1 { "" } else { "s" },
    );

    if fix {
        println!("\n{}", "Auto-fix mode is not yet implemented for all rules.".yellow());
        println!("Safe auto-fixes (FHEVM-001, FHEVM-003) will be added in v0.2.0.");
    } else {
        println!(
            "\nRun {} to attempt auto-fix of safe issues.",
            "fhevm-forge lint --fix".cyan()
        );
    }

    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
