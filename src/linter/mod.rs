use std::{fs, path::Path};
use anyhow::Result;
use walkdir::WalkDir;

pub mod rules;
pub mod reporter;

use rules::{LintError, LintRule};

pub struct Linter {
    rules:   Vec<Box<dyn LintRule>>,
    ignored: Vec<String>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(rules::MissingAllowThis),
                Box::new(rules::MissingAllowAddr),
                Box::new(rules::ViewFheFunction),
                Box::new(rules::EuintOverflow),
                Box::new(rules::InputProofReuse),
                Box::new(rules::GetRelayerCall),
                Box::new(rules::MissingOnlyGateway),
                Box::new(rules::HandleIndexOob),
                Box::new(rules::ResolverArgCount),
            ],
            ignored: Vec::new(),
        }
    }

    pub fn ignore(mut self, rule_ids: Vec<String>) -> Self {
        self.ignored = rule_ids;
        self
    }

    pub fn rule_ids(&self) -> Vec<&str> {
        self.rules.iter().map(|r| r.id()).collect()
    }

    pub fn analyze_path(&self, base_path: &str) -> Result<Vec<LintError>> {
        let mut all_errors: Vec<LintError> = Vec::new();

        if !Path::new(base_path).exists() {
            return Ok(all_errors);
        }

        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let ext = e.path().extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("");
                ext == "sol" || ext == "ts" || ext == "tsx"
            })
        {
            let source = fs::read_to_string(entry.path())?;
            for rule in &self.rules {
                if self.ignored.iter().any(|id| id == rule.id()) {
                    continue;
                }
                let errors = rule.check(entry.path(), &source);
                all_errors.extend(errors);
            }
        }

        all_errors.sort_by(|a, b| {
            a.file.cmp(&b.file).then(a.line.cmp(&b.line))
        });

        Ok(all_errors)
    }
}
