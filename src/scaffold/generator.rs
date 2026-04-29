use std::{fs, path::Path};
use tera::{Tera, Context};
use anyhow::{Result, Context as _};

pub struct Generator {
    project_dir: String,
    template:    String,
    ctx:         Context,
}

impl Generator {
    pub fn new(project_dir: &str, template: &str) -> Result<Self> {
        let mut ctx = Context::new();
        ctx.insert("project_name", project_dir);
        ctx.insert("template", template);
        ctx.insert("fhevm_version", "0.2.0");
        ctx.insert("relayer_sdk_version", "0.2.0");
        ctx.insert("year", &chrono::Utc::now().format("%Y").to_string());

        Ok(Self {
            project_dir: project_dir.to_string(),
            template: template.to_string(),
            ctx,
        })
    }

    /// Render and write all files for the chosen template.
    pub fn render_all(&self) -> Result<()> {
        self.write_shared_sdk_files()?;

        match self.template.as_str() {
            "blank"   => self.render_template_files(BLANK_FILES)?,
            "erc7984" => self.render_template_files(ERC7984_FILES)?,
            "lending" => self.render_template_files(LENDING_FILES)?,
            "auction" => self.render_template_files(AUCTION_FILES)?,
            "voting"  => self.render_template_files(VOTING_FILES)?,
            other     => anyhow::bail!("Unknown template: {}", other),
        }

        Ok(())
    }

    /// Write foundry.toml, fhevm-forge.toml, .env.example, package.json,
    /// tsconfig.json, AGENT.md, README.md
    pub fn write_config_files(&self) -> Result<()> {
        self.write_file("foundry.toml",     &self.render_str(FOUNDRY_TOML)?)?;
        self.write_file("fhevm-forge.toml", FHEVM_FORGE_TOML)?;
        self.write_file(".env.example",     ENV_EXAMPLE)?;
        self.write_file("AGENT.md",         AGENT_MD)?;
        self.write_file("README.md",        &self.render_str(README_MD)?)?;
        self.write_package_json()?;
        self.write_tsconfig()?;
        Ok(())
    }

    fn write_shared_sdk_files(&self) -> Result<()> {
        let sdk_files: &[(&str, &str)] = &[
            ("lib/fhevm/instance.ts",       FHEVM_INSTANCE_TS),
            ("lib/fhevm/encrypt.ts",        FHEVM_ENCRYPT_TS),
            ("lib/fhevm/decrypt.ts",        FHEVM_DECRYPT_TS),
            ("lib/fhevm/gateway.ts",        FHEVM_GATEWAY_TS),
            ("lib/fhevm/errors.ts",         FHEVM_ERRORS_TS),
            ("lib/fhevm/config.ts",         FHEVM_CONFIG_TS),
            ("lib/fhevm/index.ts",          FHEVM_INDEX_TS),
            ("lib/hooks/useEncrypt.ts",     HOOK_ENCRYPT_TS),
            ("lib/hooks/useReencrypt.ts",   HOOK_REENCRYPT_TS),
            ("lib/hooks/useHealthCheck.ts", HOOK_HEALTH_CHECK_TS),
            ("agent/fhevm-agent.ts",        FHEVM_AGENT_TS),
        ];

        for (path, content) in sdk_files {
            self.write_file(path, content)?;
        }

        Ok(())
    }

    fn write_package_json(&self) -> Result<()> {
        let content = format!(
r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "typecheck": "tsc --noEmit"
  }},
  "dependencies": {{
    "@zama-fhe/relayer-sdk": "^0.2.0",
    "ethers": "^6.0.0"
  }},
  "devDependencies": {{
    "typescript": "^5.0.0",
    "tsx": "^4.0.0",
    "@types/node": "^20.0.0"
  }}
}}
"#,
            self.project_dir
        );
        self.write_file("package.json", &content)
    }

    fn write_tsconfig(&self) -> Result<()> {
        let content = r#"{
  "compilerOptions": {
    "target":           "ES2022",
    "module":           "ESNext",
    "moduleResolution": "bundler",
    "strict":           true,
    "esModuleInterop":  true,
    "skipLibCheck":     true,
    "outDir":           "dist",
    "paths": {
      "@/*": ["./*"]
    }
  },
  "include": ["lib/**/*", "agent/**/*", "*.ts"],
  "exclude": ["node_modules", "dist"]
}
"#;
        self.write_file("tsconfig.json", content)
    }

    fn render_template_files(&self, files: &[(&str, &str)]) -> Result<()> {
        for (path, content) in files {
            let rendered = self.render_str(content)?;
            self.write_file(path, &rendered)?;
        }
        Ok(())
    }

    fn render_str(&self, template_str: &str) -> Result<String> {
        Tera::one_off(template_str, &self.ctx, false)
            .map_err(|e| anyhow::anyhow!("Template rendering error: {}", e))
    }

    fn write_file(&self, relative_path: &str, content: &str) -> Result<()> {
        let full_path = Path::new(&self.project_dir).join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create dir: {}", parent.display()))?;
        }
        fs::write(&full_path, content)
            .with_context(|| format!("Could not write: {}", full_path.display()))?;
        Ok(())
    }
}

// ─── Embedded Template Contents ───────────────────────────────────────────────

// Shared config files
const FOUNDRY_TOML:     &str = include_str!("../../templates/shared/foundry.toml.tera");
const FHEVM_FORGE_TOML: &str = include_str!("../../templates/shared/fhevm-forge.toml");
const ENV_EXAMPLE:      &str = include_str!("../../templates/shared/.env.example");
const AGENT_MD:         &str = include_str!("../../templates/shared/AGENT.md");
const README_MD:        &str = include_str!("../../templates/shared/README.md.tera");

// Shared TypeScript SDK files (embedded into every scaffolded project)
const FHEVM_INSTANCE_TS:    &str = include_str!("../../templates/shared/lib/fhevm/instance.ts");
const FHEVM_ENCRYPT_TS:     &str = include_str!("../../templates/shared/lib/fhevm/encrypt.ts");
const FHEVM_DECRYPT_TS:     &str = include_str!("../../templates/shared/lib/fhevm/decrypt.ts");
const FHEVM_GATEWAY_TS:     &str = include_str!("../../templates/shared/lib/fhevm/gateway.ts");
const FHEVM_ERRORS_TS:      &str = include_str!("../../templates/shared/lib/fhevm/errors.ts");
const FHEVM_CONFIG_TS:      &str = include_str!("../../templates/shared/lib/fhevm/config.ts");
const FHEVM_INDEX_TS:       &str = include_str!("../../templates/shared/lib/fhevm/index.ts");
const HOOK_ENCRYPT_TS:      &str = include_str!("../../templates/shared/hooks/useEncrypt.ts");
const HOOK_REENCRYPT_TS:    &str = include_str!("../../templates/shared/hooks/useReencrypt.ts");
const HOOK_HEALTH_CHECK_TS: &str = include_str!("../../templates/shared/hooks/useHealthCheck.ts");
const FHEVM_AGENT_TS:       &str = include_str!("../../templates/shared/agent/fhevm-agent.ts");

// Template: blank
const BLANK_FILES: &[(&str, &str)] = &[
    ("src/Counter.sol",    include_str!("../../templates/blank/src/Counter.sol")),
    ("test/Counter.t.sol", include_str!("../../templates/blank/test/Counter.t.sol")),
];

// Template: erc7984
const ERC7984_FILES: &[(&str, &str)] = &[
    ("src/ConfidentialToken.sol",    include_str!("../../templates/erc7984/src/ConfidentialToken.sol")),
    ("test/ConfidentialToken.t.sol", include_str!("../../templates/erc7984/test/ConfidentialToken.t.sol")),
    ("script/Deploy.s.sol",          include_str!("../../templates/erc7984/script/Deploy.s.sol")),
];

// Template: lending
const LENDING_FILES: &[(&str, &str)] = &[
    ("src/ConfidentialVault.sol",             include_str!("../../templates/lending/src/ConfidentialVault.sol")),
    ("src/tokens/ConfidentialCollateral.sol", include_str!("../../templates/lending/src/tokens/ConfidentialCollateral.sol")),
    ("src/tokens/ConfidentialDebt.sol",       include_str!("../../templates/lending/src/tokens/ConfidentialDebt.sol")),
    ("src/PriceOracle.sol",                   include_str!("../../templates/lending/src/PriceOracle.sol")),
    ("test/ConfidentialVault.t.sol",          include_str!("../../templates/lending/test/ConfidentialVault.t.sol")),
    ("script/Deploy.s.sol",                   include_str!("../../templates/lending/script/Deploy.s.sol")),
];

// Template: auction
const AUCTION_FILES: &[(&str, &str)] = &[
    ("src/BlindAuction.sol",    include_str!("../../templates/auction/src/BlindAuction.sol")),
    ("test/BlindAuction.t.sol", include_str!("../../templates/auction/test/BlindAuction.t.sol")),
    ("script/Deploy.s.sol",     include_str!("../../templates/auction/script/Deploy.s.sol")),
];

// Template: voting
const VOTING_FILES: &[(&str, &str)] = &[
    ("src/ConfidentialVoting.sol",    include_str!("../../templates/voting/src/ConfidentialVoting.sol")),
    ("test/ConfidentialVoting.t.sol", include_str!("../../templates/voting/test/ConfidentialVoting.t.sol")),
    ("script/Deploy.s.sol",           include_str!("../../templates/voting/script/Deploy.s.sol")),
];
