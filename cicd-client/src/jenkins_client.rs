//! Jenkins REST API 客户端
//!
//! 提供触发构建、查询构建状态等功能。
//! 使用方式：
//! ```ignore
//! let client = JenkinsClient::new("https://jenkins.example.com", "user", "api_token");
//! let params = serde_json::json!({
//!     "MODULE": "declaration-server",
//!     "ENVIRONMENT": "test",
//!     "VERSION": "develop",
//!     // ...
//! });
//! client.build_with_params("your-job-name", &params).await?;
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Jenkins 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenkinsConfig {
    pub base_url: String,
    pub username: String,
    pub api_token: String,
}

/// Jenkins 客户端（简化版，实际应用中建议使用 reqwest 等 HTTP 库）
pub struct JenkinsClient {
    config: Arc<JenkinsConfig>,
}

impl JenkinsClient {
    pub fn new(config: JenkinsConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// 构建带参数的 Job
    pub async fn build_with_params(
        &self,
        job_name: &str,
        params: &serde_json::Value,
    ) -> Result<()> {
        let url = format!(
            "{}/job/{}/buildWithParameters",
            self.config.base_url.trim_end_matches('/'),
            job_name
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.api_token))
            .form(&extract_form_params(params))
            .send()
            .await
            .with_context(|| format!("Failed to send request to Jenkins at {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Jenkins API error {}: {}", status, body);
        }

        Ok(())
    }

    /// 查询构建队列
    pub async fn get_queue(&self) -> Result<serde_json::Value> {
        let url = format!("{}/queue/api/json", self.config.base_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.api_token))
            .send()
            .await?;
        Ok(response.json().await?)
    }

    /// 查询 Job 最新构建信息
    pub async fn get_last_build(&self, job_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "{}/job/{}/lastBuild/api/json",
            self.config.base_url.trim_end_matches('/'),
            job_name
        );
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.api_token))
            .send()
            .await?;
        Ok(response.json().await?)
    }

    /// 获取构建控制台日志
    pub async fn get_build_console(
        &self,
        job_name: &str,
        build_number: u64,
    ) -> Result<String> {
        let url = format!(
            "{}/job/{}/{}/consoleText",
            self.config.base_url.trim_end_matches('/'),
            job_name,
            build_number
        );
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.api_token))
            .send()
            .await?;
        Ok(response.text().await?)
    }
}

/// 将 JSON 对象扁平化为 `key=value` 形式的表参数
fn extract_form_params(value: &serde_json::Value) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            let str_val = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => String::new(),
            };
            params.push((k.clone(), str_val));
        }
    }
    params
}
