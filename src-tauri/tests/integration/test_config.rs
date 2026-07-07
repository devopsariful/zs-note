//! Integration tests for configuration management.
//!
//! Covers default construction, YAML round‑trip serialisation and
//! the validation logic on [`Config`](zs_note_lib::config::Config).

use std::path::Path;

use zs_note_lib::config::Config;

// ---------------------------------------------------------------------------
// Default
// ---------------------------------------------------------------------------

#[test]
fn test_config_default() {
    let cfg = Config::default();

    // Version
    assert_eq!(cfg.version, "1");

    // Vault
    assert_eq!(cfg.vault.name, "My Vault");
    assert_eq!(cfg.vault.vault_type, "local");

    // Editor settings
    assert_eq!(cfg.editor.font_size, 16);
    assert!(cfg.editor.auto_save);
    assert!(!cfg.editor.vim_mode);
    assert!(cfg.editor.spell_check);

    // AI
    assert_eq!(cfg.ai.provider, "ollama");
    assert_eq!(cfg.ai.model, "llama3.2");
    assert_eq!(cfg.ai.max_tokens, 4096);
    assert!((cfg.ai.temperature - 0.7).abs() < f32::EPSILON);

    // Sandbox
    assert!(cfg.sandbox.enabled);
    assert_eq!(cfg.sandbox.default_memory_limit, 64 * 1024 * 1024);
    assert_eq!(cfg.sandbox.default_timeout, 30_000);

    // Git
    assert!(cfg.git.auto_commit);
    assert_eq!(cfg.git.commit_style, "conventional");
    assert!(cfg.git.remote.is_none());

    // Theme
    assert_eq!(cfg.theme.name, "zarish-light");
    assert_eq!(cfg.theme.font_size, 16);

    // Features
    assert!(cfg.features.contains(&"sandbox".to_string()));
    assert!(cfg.features.contains(&"ai".to_string()));
    assert!(cfg.features.contains(&"git".to_string()));

    // Collections
    assert!(cfg.knowledge.is_empty());
    assert!(cfg.publish.is_empty());
    assert!(cfg.plugins.enabled.is_empty());
    assert!(!cfg.voice.enabled);
}

// ---------------------------------------------------------------------------
// Validate
// ---------------------------------------------------------------------------

#[test]
fn test_config_validate_valid() {
    let cfg = Config::default();
    assert!(cfg.validate().is_ok(), "default config should be valid");
}

#[test]
fn test_config_validate_invalid_version() {
    let mut cfg = Config::default();
    cfg.version = "0".to_string();
    let err = cfg.validate().expect_err("version < 1 should fail");
    assert!(err.to_string().contains("version"));
}

#[test]
fn test_config_validate_empty_vault_name() {
    let mut cfg = Config::default();
    cfg.vault.name = "".to_string();
    let err = cfg.validate().expect_err("empty vault name should fail");
    assert!(err.to_string().contains("name"));
}

#[test]
fn test_config_validate_font_size_bounds() {
    let mut cfg = Config::default();

    cfg.editor.font_size = 4;
    let err = cfg.validate().expect_err("font size 4 should fail");
    assert!(err.to_string().contains("Font size"));

    cfg.editor.font_size = 100;
    let err = cfg.validate().expect_err("font size 100 should fail");
    assert!(err.to_string().contains("Font size"));

    cfg.editor.font_size = 14; // valid
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_config_validate_line_height() {
    let mut cfg = Config::default();

    cfg.editor.line_height = 0.5;
    let err = cfg.validate().expect_err("line height 0.5 should fail");
    assert!(err.to_string().contains("Line height"));

    cfg.editor.line_height = 3.5;
    let err = cfg.validate().expect_err("line height 3.5 should fail");
    assert!(err.to_string().contains("Line height"));

    cfg.editor.line_height = 1.5; // valid
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_config_validate_temperature() {
    let mut cfg = Config::default();

    cfg.ai.temperature = -0.1;
    let err = cfg
        .validate()
        .expect_err("negative temperature should fail");
    assert!(err.to_string().contains("Temperature"));

    cfg.ai.temperature = 2.5;
    let err = cfg.validate().expect_err("temperature > 2.0 should fail");
    assert!(err.to_string().contains("Temperature"));

    cfg.ai.temperature = 1.0;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_config_validate_max_tokens() {
    let mut cfg = Config::default();

    cfg.ai.max_tokens = 0;
    let err = cfg.validate().expect_err("zero max tokens should fail");
    assert!(err.to_string().contains("Max tokens"));

    cfg.ai.max_tokens = 200_000;
    let err = cfg.validate().expect_err("max tokens > 128000 should fail");
    assert!(err.to_string().contains("Max tokens"));

    cfg.ai.max_tokens = 8192;
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_config_validate_sandbox_limits() {
    let mut cfg = Config::default();

    cfg.sandbox.default_memory_limit = 1024 * 1024 * 1024; // 1GB
    let err = cfg.validate().expect_err("memory > 512MB should fail");
    assert!(err.to_string().contains("memory"));

    cfg.sandbox.default_memory_limit = 64 * 1024 * 1024; // valid
    assert!(cfg.validate().is_ok());

    cfg.sandbox.default_timeout = 600_000; // 600s
    let err = cfg.validate().expect_err("timeout > 300s should fail");
    assert!(err.to_string().contains("timeout"));

    cfg.sandbox.default_timeout = 30_000; // valid
    assert!(cfg.validate().is_ok());
}

#[test]
fn test_config_validate_tool_config() {
    let mut cfg = Config::default();
    cfg.sandbox.tools.push(zs_note_lib::types::ToolConfig {
        name: "".to_string(),
        wasm_path: std::path::PathBuf::from("tool.wasm"),
        permissions: vec![],
        memory_limit: 1024,
        timeout: 30_000,
    });
    let err = cfg.validate().expect_err("empty tool name should fail");
    assert!(err.to_string().contains("Tool name"));
}

#[test]
fn test_config_validate_knowledge_base() {
    let mut cfg = Config::default();
    cfg.knowledge.push(zs_note_lib::types::KnowledgeBase {
        name: "".to_string(),
        path: std::path::PathBuf::from("/tmp/kb"),
        formats: vec![],
        index_on_start: false,
    });
    let err = cfg.validate().expect_err("empty KB name should fail");
    assert!(err.to_string().contains("name"));
}

// ---------------------------------------------------------------------------
// Round‑trip
// ---------------------------------------------------------------------------

#[test]
fn test_config_roundtrip() {
    let original = Config::default();

    // Serialize to YAML string
    let yaml = serde_yaml::to_string(&original).expect("serialization should succeed");
    assert!(!yaml.is_empty(), "YAML output should not be empty");

    // Deserialize back
    let deserialized: Config = serde_yaml::from_str(&yaml).expect("deserialization should succeed");

    // Compare key fields
    assert_eq!(deserialized.version, original.version);
    assert_eq!(deserialized.vault.name, original.vault.name);
    assert_eq!(deserialized.editor.font_size, original.editor.font_size);
    assert_eq!(deserialized.editor.line_height, original.editor.line_height);
    assert_eq!(deserialized.ai.provider, original.ai.provider);
    assert_eq!(deserialized.ai.model, original.ai.model);
    assert_eq!(deserialized.ai.max_tokens, original.ai.max_tokens);
    assert_eq!(deserialized.theme.name, original.theme.name);
    assert_eq!(deserialized.sandbox.enabled, original.sandbox.enabled);
    assert_eq!(
        deserialized.sandbox.default_memory_limit,
        original.sandbox.default_memory_limit
    );
    assert_eq!(deserialized.git.commit_style, original.git.commit_style);
    assert_eq!(deserialized.keys.save, original.keys.save);
    assert_eq!(deserialized.features, original.features);
}

#[test]
fn test_config_roundtrip_modified() {
    let mut original = Config::default();
    original.vault.name = "My Research Vault".to_string();
    original.editor.font_size = 18;
    original.editor.vim_mode = true;
    original.ai.provider = "openai".to_string();
    original.ai.model = "gpt-4".to_string();
    original.git.commit_style = "simple".to_string();

    let yaml = serde_yaml::to_string(&original).unwrap();
    let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(deserialized.vault.name, "My Research Vault");
    assert_eq!(deserialized.editor.font_size, 18);
    assert!(deserialized.editor.vim_mode);
    assert_eq!(deserialized.ai.provider, "openai");
    assert_eq!(deserialized.ai.model, "gpt-4");
    assert_eq!(deserialized.git.commit_style, "simple");

    // Validate the round‑tripped config
    assert!(deserialized.validate().is_ok());
}

#[test]
fn test_config_save_and_load() {
    let dir = tempfile::tempdir().expect("temp dir");
    let original = Config::default();

    // Save to .znrc file
    original.save(dir.path()).expect("save should succeed");

    // Load it back
    let loaded = Config::load(dir.path()).expect("load should succeed");
    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.vault.name, original.vault.name);
    assert_eq!(loaded.editor.font_size, original.editor.font_size);
}

#[test]
fn test_config_load_missing_file_returns_default() {
    let dir = tempfile::tempdir().expect("temp dir");
    // No .znrc present
    let loaded = Config::load(dir.path()).expect("load with no .znrc should return default");
    assert_eq!(loaded.version, "1");
}
