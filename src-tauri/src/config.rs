use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::types::{
    EditorSettings, ImageHost, KnowledgeBase, Provider, ProviderConfig, PublishTarget, SyncConfig,
    ToolConfig, VaultConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".into(),
            model: "llama3.2".into(),
            api_key: None,
            base_url: Some("http://localhost:11434".into()),
            max_tokens: 4096,
            temperature: 0.7,
            providers: vec![ProviderConfig {
                id: "ollama".into(),
                name: "Ollama".into(),
                provider_type: Provider::Ollama,
                api_key: None,
                base_url: Some("http://localhost:11434".into()),
                models: vec!["llama3.2".into()],
                default_model: "llama3.2".into(),
                enabled: true,
                temperature: Some(0.7),
                max_tokens: Some(4096),
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub font_family: String,
    pub font_size: u32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "zarish-light".into(),
            background: "#ffffff".into(),
            foreground: "#1a1a2e".into(),
            accent: "#7c3aed".into(),
            font_family: "Inter, sans-serif".into(),
            font_size: 16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub default_memory_limit: usize,
    pub default_timeout: u64,
    pub allowed_domains: Vec<String>,
    pub max_module_size: usize,
    pub tools: Vec<ToolConfig>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_memory_limit: 64 * 1024 * 1024,
            default_timeout: 30_000,
            allowed_domains: vec!["*.example.com".into()],
            max_module_size: 10 * 1024 * 1024,
            tools: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_commit: bool,
    pub commit_style: String,
    pub remote: Option<RemoteConfig>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_commit: true,
            commit_style: "conventional".into(),
            remote: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub url: String,
    pub branch: String,
    pub ssh_key: Option<String>,
    pub auto_sync: bool,
    pub sync_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub save: String,
    pub search: String,
    pub command_palette: String,
    pub toggle_sidebar: String,
    pub toggle_preview: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            save: "ctrl+s".into(),
            search: "ctrl+p".into(),
            command_palette: "ctrl+shift+p".into(),
            toggle_sidebar: "ctrl+b".into(),
            toggle_preview: "ctrl+shift+v".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    pub enabled: Vec<String>,
    pub settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub language: String,
    pub model: String,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            language: "en-US".into(),
            model: "base".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub vault: VaultConfig,
    pub editor: EditorSettings,
    pub ai: AIConfig,
    pub theme: ThemeConfig,
    pub sandbox: SandboxConfig,
    pub git: GitConfig,
    pub sync: SyncConfig,
    pub keys: KeyBindings,
    pub plugins: PluginsConfig,
    pub voice: VoiceConfig,
    pub knowledge: Vec<KnowledgeBase>,
    pub publish: Vec<PublishTarget>,
    #[serde(rename = "imageHost")]
    pub image_host: Option<ImageHost>,
    pub features: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1".into(),
            vault: VaultConfig {
                name: "My Vault".into(),
                path: PathBuf::from("."),
                vault_type: "local".into(),
                created_at: chrono::Utc::now(),
                version: "1".into(),
            },
            editor: EditorSettings {
                theme: "zarish-light".into(),
                font_size: 16,
                font_family: "Inter, sans-serif".into(),
                line_height: 1.7,
                prose_width: 720,
                vim_mode: false,
                spell_check: true,
                auto_save: true,
            },
            ai: AIConfig::default(),
            theme: ThemeConfig::default(),
            sandbox: SandboxConfig::default(),
            git: GitConfig::default(),
            sync: SyncConfig {
                auto_commit: true,
                commit_style: "conventional".into(),
                remote: None,
            },
            keys: KeyBindings::default(),
            plugins: PluginsConfig::default(),
            voice: VoiceConfig::default(),
            knowledge: Vec::new(),
            publish: Vec::new(),
            image_host: None,
            features: vec!["sandbox".into(), "ai".into(), "git".into(), "search".into()],
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        let ver: u32 = self.version.parse().context("Invalid version number")?;
        if ver < 1 {
            bail!("Config version must be >= 1");
        }
        if self.vault.name.is_empty() {
            bail!("Vault name is required");
        }
        if self.editor.font_size < 8 || self.editor.font_size > 72 {
            bail!("Font size must be between 8 and 72");
        }
        if self.editor.line_height < 1.0 || self.editor.line_height > 3.0 {
            bail!("Line height must be between 1.0 and 3.0");
        }
        if self.ai.temperature < 0.0 || self.ai.temperature > 2.0 {
            bail!("Temperature must be between 0.0 and 2.0");
        }
        if self.ai.max_tokens == 0 || self.ai.max_tokens > 128_000 {
            bail!("Max tokens must be between 1 and 128000");
        }
        if self.sandbox.default_memory_limit > 512 * 1024 * 1024 {
            bail!("Sandbox memory limit cannot exceed 512MB");
        }
        if self.sandbox.default_timeout > 300_000 {
            bail!("Sandbox timeout cannot exceed 300 seconds");
        }
        if self.sandbox.max_module_size > 50 * 1024 * 1024 {
            bail!("Max WASM module size cannot exceed 50MB");
        }
        for tool in &self.sandbox.tools {
            if tool.name.is_empty() {
                bail!("Tool name is required");
            }
            if tool.timeout > 300_000 {
                bail!("Tool timeout cannot exceed 300 seconds");
            }
        }
        for kb in &self.knowledge {
            if kb.name.is_empty() {
                bail!("Knowledge base name is required");
            }
        }
        Ok(())
    }

    fn ensure_provider_records(&mut self) {
        if !self.ai.providers.is_empty() {
            return;
        }

        let provider_type = self
            .ai
            .provider
            .parse::<Provider>()
            .unwrap_or(Provider::Custom);

        self.ai.providers.push(ProviderConfig {
            id: self.ai.provider.clone(),
            name: self.ai.provider.clone(),
            provider_type,
            api_key: self.ai.api_key.clone(),
            base_url: self.ai.base_url.clone(),
            models: vec![self.ai.model.clone()],
            default_model: self.ai.model.clone(),
            enabled: true,
            temperature: Some(self.ai.temperature),
            max_tokens: Some(self.ai.max_tokens),
        });
    }

    pub fn load(path: &Path) -> Result<Self> {
        let config_path = path.join(".znrc");
        if !config_path.exists() {
            tracing::info!("No .znrc found at {:?}, using defaults", config_path);
            return Ok(Config::default());
        }
        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read .znrc from {:?}", config_path))?;
        let mut config: Config = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse .znrc at {:?}", config_path))?;
        config.ensure_provider_records();
        config.validate()?;
        tracing::info!("Loaded config from {:?}", config_path);
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let config_path = path.join(".znrc");
        let yaml = serde_yaml::to_string(self).context("Failed to serialize config to YAML")?;
        std::fs::write(&config_path, &yaml)
            .with_context(|| format!("Failed to write .znrc to {:?}", config_path))?;
        tracing::info!("Saved config to {:?}", config_path);
        Ok(())
    }

    pub fn start_hot_reload(
        path: PathBuf,
        config: Arc<RwLock<Config>>,
    ) -> Result<RecommendedWatcher> {
        let _config_path = path.join(".znrc");
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        let path = event.paths.first().cloned();
                        if let Some(p) = path {
                            if p.file_name().map(|n| n == ".znrc").unwrap_or(false) {
                                tracing::info!("Detected .znrc change, reloading config");
                                let parent = p.parent().map(|p| p.to_path_buf());
                                if let Some(parent) = parent {
                                    match Config::load(&parent) {
                                        Ok(new_config) => {
                                            let mut cfg = config.blocking_write();
                                            *cfg = new_config;
                                            tracing::info!("Config hot-reloaded");
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to reload config: {:#}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            NotifyConfig::default(),
        )?;
        watcher.watch(&path, RecursiveMode::NonRecursive)?;
        Ok(watcher)
    }
}
