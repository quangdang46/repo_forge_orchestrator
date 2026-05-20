//! Configuration schema for rfo.

use serde::{Deserialize, Serialize};

/// Top-level application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default = "default_projects_dir")]
    pub projects_dir: String,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_parallel")]
    pub parallel: u32,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    #[serde(default = "default_github_host")]
    pub host: String,
    #[serde(default = "default_auth")]
    pub auth: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_strategy")]
    pub update_strategy: String,
    #[serde(default)]
    pub autostash: bool,
    #[serde(default = "default_false")]
    pub terminal_prompt: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetyConfig {
    #[serde(default = "default_secret_scan")]
    pub secret_scan: String,
    #[serde(default = "default_true")]
    pub require_plan_for_ai_apply: bool,
    #[serde(default = "default_max_risk")]
    pub max_auto_apply_risk: String,
}

fn default_projects_dir() -> String {
    "~/projects".into()
}
fn default_layout() -> String {
    "flat".into()
}
fn default_parallel() -> u32 {
    8
}
fn default_timeout() -> u32 {
    30
}
fn default_github_host() -> String {
    "github.com".into()
}
fn default_auth() -> String {
    "auto".into()
}
fn default_strategy() -> String {
    "ff-only".into()
}
fn default_secret_scan() -> String {
    "block".into()
}
fn default_max_risk() -> String {
    "low".into()
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
