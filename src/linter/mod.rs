use std::{fs, path::Path};
use anyhow::Result;
use walkdir::WalkDir;

pub mod rules;
pub mod reporter;

use rules::{LintError, LintRule};

pub struct Linter {
    rules: Vec<Box<dyn LintRule>>,
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
        }
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
