use std::{fs, path::Path};
use tera::{Tera, Context};
use anyhow::{Result, Context as _};

pub struct Generator {
    project_dir: String,
    template:    String,
    frontend:    bool,
    ctx:         Context,
}

impl Generator {
    pub fn new(project_dir: &str, template: &str, frontend: bool) -> Result<Self> {
        let mut ctx = Context::new();
        ctx.insert("project_name",        project_dir);
        ctx.insert("template",            template);
        ctx.insert("fhevm_version",       "0.2.0");
        ctx.insert("relayer_sdk_version", "0.2.0");
        ctx.insert("year",                &chrono::Utc::now().format("%Y").to_string());

        Ok(Self {
            project_dir: project_dir.to_string(),
            template:    template.to_string(),
            frontend,
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

        if self.frontend {
            self.write_frontend_files()?;
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
        self.write_file("package.json",     &self.render_str(PACKAGE_JSON)?)?;
        self.write_file("tsconfig.json",    TSCONFIG_JSON)?;
        Ok(())
    }

    // ── Shared SDK files (lib/fhevm/ + hooks/) ──────────────────────────────

    fn write_shared_sdk_files(&self) -> Result<()> {
        // lib/fhevm/
        self.write_file("lib/fhevm/instance.ts", FHEVM_INSTANCE)?;
        self.write_file("lib/fhevm/config.ts",   FHEVM_CONFIG)?;
        self.write_file("lib/fhevm/errors.ts",   FHEVM_ERRORS)?;
        self.write_file("lib/fhevm/encrypt.ts",  FHEVM_ENCRYPT)?;
        self.write_file("lib/fhevm/decrypt.ts",  FHEVM_DECRYPT)?;
        self.write_file("lib/fhevm/index.ts",    FHEVM_INDEX)?;
        // hooks/ (React-agnostic, ethers-signer variants for agents/scripts)
        self.write_file("hooks/useEncrypt.ts",   HOOK_ENCRYPT)?;
        self.write_file("hooks/useReencrypt.ts", HOOK_REENCRYPT)?;
        Ok(())
    }

    // ── Frontend scaffold (frontend/ subdirectory) ──────────────────────────

    fn write_frontend_files(&self) -> Result<()> {
        // Config + framework files
        self.write_file("frontend/package.json",   &self.render_str(FRONTEND_PACKAGE_JSON)?)?;
        self.write_file("frontend/next.config.ts", FRONTEND_NEXT_CONFIG)?;
        self.write_file("frontend/tsconfig.json",  FRONTEND_TSCONFIG)?;
        self.write_file("frontend/.env.local.example", FRONTEND_ENV_EXAMPLE)?;

        // App Router layout + providers
        self.write_file("frontend/app/layout.tsx",   &self.render_str(FRONTEND_LAYOUT)?)?;
        self.write_file("frontend/app/providers.tsx", FRONTEND_PROVIDERS)?;
        self.write_file("frontend/app/globals.css",   FRONTEND_GLOBALS_CSS)?;

        // Contract ABI + page — currently Counter only; other templates get blank page
        let (contract, page) = match self.template.as_str() {
            "blank" => (FRONTEND_CONTRACT_BLANK, FRONTEND_PAGE_BLANK),
            _       => (FRONTEND_CONTRACT_BLANK, FRONTEND_PAGE_BLANK),
        };
        self.write_file("frontend/app/contract.ts", contract)?;
        self.write_file("frontend/app/page.tsx",    page)?;

        // Wagmi-compatible hooks (override the ethers-signer variants in hooks/)
        self.write_file("frontend/hooks/useEncrypt.ts",   FRONTEND_HOOK_ENCRYPT)?;
        self.write_file("frontend/hooks/useReencrypt.ts", FRONTEND_HOOK_REENCRYPT)?;

        // Shared lib/fhevm/ — same SDK files, self-contained inside frontend/
        self.write_file("frontend/lib/fhevm/instance.ts", FHEVM_INSTANCE)?;
        self.write_file("frontend/lib/fhevm/config.ts",   FHEVM_CONFIG)?;
        self.write_file("frontend/lib/fhevm/errors.ts",   FHEVM_ERRORS)?;
        self.write_file("frontend/lib/fhevm/encrypt.ts",  FHEVM_ENCRYPT)?;
        self.write_file("frontend/lib/fhevm/decrypt.ts",  FHEVM_DECRYPT)?;
        self.write_file("frontend/lib/fhevm/index.ts",    FHEVM_INDEX)?;

        Ok(())
    }

    // ── Helpers ─────────────────────────────────────────────────────────────

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
const PACKAGE_JSON:     &str = include_str!("../../templates/shared/package.json.tera");
const TSCONFIG_JSON:    &str = include_str!("../../templates/shared/tsconfig.json");

// Shared lib/fhevm/ SDK
const FHEVM_INSTANCE: &str = include_str!("../../templates/shared/lib/fhevm/instance.ts");
const FHEVM_CONFIG:   &str = include_str!("../../templates/shared/lib/fhevm/config.ts");
const FHEVM_ERRORS:   &str = include_str!("../../templates/shared/lib/fhevm/errors.ts");
const FHEVM_ENCRYPT:  &str = include_str!("../../templates/shared/lib/fhevm/encrypt.ts");
const FHEVM_DECRYPT:  &str = include_str!("../../templates/shared/lib/fhevm/decrypt.ts");
const FHEVM_INDEX:    &str = include_str!("../../templates/shared/lib/fhevm/index.ts");

// Shared hooks/ (ethers-signer variants for agents/scripts)
const HOOK_ENCRYPT:   &str = include_str!("../../templates/shared/hooks/useEncrypt.ts");
const HOOK_REENCRYPT: &str = include_str!("../../templates/shared/hooks/useReencrypt.ts");

// Frontend scaffold — shared
const FRONTEND_PACKAGE_JSON:  &str = include_str!("../../templates/frontend/package.json.tera");
const FRONTEND_NEXT_CONFIG:   &str = include_str!("../../templates/frontend/next.config.ts");
const FRONTEND_TSCONFIG:      &str = include_str!("../../templates/frontend/tsconfig.json");
const FRONTEND_ENV_EXAMPLE:   &str = include_str!("../../templates/frontend/.env.local.example");
const FRONTEND_LAYOUT:        &str = include_str!("../../templates/frontend/app/layout.tsx.tera");
const FRONTEND_PROVIDERS:     &str = include_str!("../../templates/frontend/app/providers.tsx");
const FRONTEND_GLOBALS_CSS:   &str = include_str!("../../templates/frontend/app/globals.css");
const FRONTEND_HOOK_ENCRYPT:   &str = include_str!("../../templates/frontend/hooks/useEncrypt.ts");
const FRONTEND_HOOK_REENCRYPT: &str = include_str!("../../templates/frontend/hooks/useReencrypt.ts");

// Frontend scaffold — Counter (blank template)
const FRONTEND_CONTRACT_BLANK: &str = include_str!("../../templates/frontend/app/contract.ts");
const FRONTEND_PAGE_BLANK:     &str = include_str!("../../templates/frontend/app/page.tsx");

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
