// Forge script subprocess executor.
// The deploy command (commands/deploy.rs) calls forge directly via tokio::process::Command.
// This module provides shared utilities for building and running forge subprocesses.

use anyhow::Result;

pub struct ForgeRunner {
    pub rpc_url: String,
    pub verbose: bool,
}

impl ForgeRunner {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            verbose: false,
        }
    }

    pub fn verbose(mut self, v: bool) -> Self {
        self.verbose = v;
        self
    }

    pub async fn run_script(&self, script_path: &str, broadcast: bool) -> Result<String> {
        let mut cmd = tokio::process::Command::new("forge");
        cmd.args(["script", script_path, "--rpc-url", &self.rpc_url]);

        if broadcast {
            cmd.arg("--broadcast");
        }

        if self.verbose {
            cmd.arg("-vvv");
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("forge script failed:\n{}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}
