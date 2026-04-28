// Forge script subprocess executor shared across commands.

use anyhow::Result;

pub struct ForgeRunner {
    pub rpc_url: String,
    pub verbose: bool,
    envs:        Vec<(String, String)>,
    extra_args:  Vec<String>,
}

impl ForgeRunner {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url:     rpc_url.to_string(),
            verbose:     false,
            envs:        Vec::new(),
            extra_args:  Vec::new(),
        }
    }

    pub fn verbose(mut self, v: bool) -> Self {
        self.verbose = v;
        self
    }

    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.envs.push((key.into(), val.into()));
        self
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    pub async fn run_script(&self, script_path: &str, broadcast: bool) -> Result<String> {
        let mut cmd = tokio::process::Command::new("forge");
        cmd.args(["script", script_path, "--rpc-url", &self.rpc_url]);

        if broadcast {
            cmd.arg("--broadcast");
        }

        for arg in &self.extra_args {
            cmd.arg(arg);
        }

        if self.verbose {
            cmd.arg("-vvv");
        }

        for (k, v) in &self.envs {
            cmd.env(k, v);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("forge script failed:\n{}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}
