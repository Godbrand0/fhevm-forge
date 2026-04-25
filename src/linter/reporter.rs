// Linter error display helpers.
// The commands/lint.rs module handles primary output.
// This module provides structured formatting utilities.

use colored::Colorize;
use crate::linter::rules::{LintError, Severity};

pub fn format_error(e: &LintError) -> String {
    let severity = match e.severity {
        Severity::Error   => "ERROR".red().bold().to_string(),
        Severity::Warning => "WARN ".yellow().bold().to_string(),
    };
    format!(
        "[{}] {}  {}:{}  {}",
        e.rule_id.dimmed(),
        severity,
        e.file,
        e.line,
        e.message,
    )
}

pub fn print_summary(errors: &[LintError]) {
    let error_count   = errors.iter().filter(|e| e.severity == Severity::Error).count();
    let warning_count = errors.iter().filter(|e| e.severity == Severity::Warning).count();

    println!(
        "\n{}: {} error{}, {} warning{}",
        "Summary".bold(),
        error_count.to_string().red(),
        if error_count == 1 { "" } else { "s" },
        warning_count.to_string().yellow(),
        if warning_count == 1 { "" } else { "s" },
    );
}
