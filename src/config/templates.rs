use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub name: String,
    pub description: String,
    pub config: crate::SymorConfig,
    pub patterns: Vec<String>,
}
pub struct TemplateManager {
    templates: HashMap<String, ConfigTemplate>,
    custom_templates_path: PathBuf,
}
impl TemplateManager {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            custom_templates_path: PathBuf::from(".symor/templates"),
        }
    }
    pub fn load_builtin_templates(&mut self) -> Result<()> {
        let dev_template = ConfigTemplate {
            name: "development".to_string(),
            description: "Development environment with frequent backups".to_string(),
            config: crate::SymorConfig {
                home_dir: PathBuf::from(".symor"),
                versioning: crate::VersioningConfig {
                    enabled: true,
                    max_versions: 50,
                    compression: 3,
                },
                linking: crate::LinkingConfig {
                    link_type: "copy".to_string(),
                    preserve_permissions: true,
                },
            },
            patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
        };
        let prod_template = ConfigTemplate {
            name: "production".to_string(),
            description: "Production environment with optimal compression".to_string(),
            config: crate::SymorConfig {
                home_dir: PathBuf::from(".symor"),
                versioning: crate::VersioningConfig {
                    enabled: true,
                    max_versions: 20,
                    compression: 9,
                },
                linking: crate::LinkingConfig {
                    link_type: "hard".to_string(),
                    preserve_permissions: true,
                },
            },
            patterns: vec!["*.txt".to_string(), "*.md".to_string()],
        };
        let backup_template = ConfigTemplate {
            name: "backup".to_string(),
            description: "Backup-focused configuration".to_string(),
            config: crate::SymorConfig {
                home_dir: PathBuf::from(".symor"),
                versioning: crate::VersioningConfig {
                    enabled: true,
                    max_versions: 100,
                    compression: 6,
                },
                linking: crate::LinkingConfig {
                    link_type: "copy".to_string(),
                    preserve_permissions: true,
                },
            },
            patterns: vec!["*".to_string()],
        };
        self.templates.insert(dev_template.name.clone(), dev_template);
        self.templates.insert(prod_template.name.clone(), prod_template);
        self.templates.insert(backup_template.name.clone(), backup_template);
        Ok(())
    }
    pub fn get_template(&self, name: &str) -> Option<&ConfigTemplate> {
        self.templates.get(name)
    }
    pub fn list_templates(&self) -> Vec<&ConfigTemplate> {
        self.templates.values().collect()
    }
    pub fn create_from_template(
        &self,
        template_name: &str,
        overrides: &ConfigOverrides,
    ) -> Result<crate::SymorConfig> {
        let template = self
            .get_template(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;
        let mut config = template.config.clone();
        if let Some(max_versions) = overrides.max_versions {
            config.versioning.max_versions = max_versions;
        }
        if let Some(compression) = overrides.compression {
            config.versioning.compression = compression;
        }
        if let Some(link_type) = &overrides.link_type {
            config.linking.link_type = link_type.clone();
        }
        Ok(config)
    }
    pub fn save_custom_template(
        &self,
        name: String,
        config: crate::SymorConfig,
    ) -> Result<()> {
        use std::fs;
        let template = ConfigTemplate {
            name: name.clone(),
            description: format!("Custom template: {}", name),
            config,
            patterns: vec!["*".to_string()],
        };
        let custom_path = self.custom_templates_path.join(format!("{}.json", name));
        fs::create_dir_all(&self.custom_templates_path)?;
        let json_data = serde_json::to_string_pretty(&template)?;
        fs::write(custom_path, json_data)?;
        Ok(())
    }
    pub fn load_custom_templates(&mut self) -> Result<()> {
        use std::fs;
        if !self.custom_templates_path.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(&self.custom_templates_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json_data = fs::read_to_string(&path)?;
                let template: ConfigTemplate = serde_json::from_str(&json_data)?;
                self.templates.insert(template.name.clone(), template);
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
    pub max_versions: Option<usize>,
    pub compression: Option<u8>,
    pub link_type: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub config_path: PathBuf,
    pub auto_switch: bool,
    pub variables: HashMap<String, String>,
}