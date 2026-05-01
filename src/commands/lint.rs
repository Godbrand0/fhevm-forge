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
        println!("\n{}", "Applying auto-fixes...".green().bold());
        let mut fix_count = 0;

        let re_003 = regex::Regex::new(r"\b(?:view|pure)\b").unwrap();
        let re_001 = regex::Regex::new(r"euint\d+\s+(\w+)\s*=\s*TFHE\.").unwrap();

        for (file_path, file_errors) in &by_file {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                let mut modified = false;

                for e in file_errors {
                    let idx = e.line.saturating_sub(1);
                    if idx >= lines.len() { continue; }

                    if e.rule_id == "FHEVM-003" {
                        let old_line = lines[idx].clone();
                        lines[idx] = re_003.replace_all(&old_line, "").into_owned();
                        if lines[idx] != old_line {
                            modified = true;
                            fix_count += 1;
                        }
                    } else if e.rule_id == "FHEVM-001" {
                        if let Some(cap) = re_001.captures(&lines[idx]) {
                            let var_name = cap[1].to_string();
                            lines[idx].push_str(&format!(" TFHE.allowThis({});", var_name));
                            modified = true;
                            fix_count += 1;
                        }
                    }
                }

                if modified {
                    let new_content = lines.join("\n") + "\n";
                    let _ = std::fs::write(file_path, new_content);
                }
            }
        }

        println!("Fixed {} issue(s).", fix_count);
        if fix_count > 0 {
            println!("Please run the linter again to verify.");
            return Ok(());
        }
    } else {
        println!(
            "\nRun {} to attempt auto-fix of safe issues.",
            "fhevm-forge lint --fix".cyan()
        );
    }

    let error_count = errors.iter().filter(|e| e.severity == Severity::Error).count();
    if error_count > 0 && (!fix || error_count > 0) { // Keep exit 1 if errors remain
        std::process::exit(1);
    }

    Ok(())
}
