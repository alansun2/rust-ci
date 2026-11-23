use serde::{Deserialize, Serialize};

/// 支持的模块列表（从 Jenkinsfile 提取）
pub const MODULES: &[&str] = &[
    "declaration-server",
    "addr-server",
    "setting-server",
    "message-server",
    "warehouse-server",
    "operation-data",
    "hscode-server",
    "category-server",
    "waybill",
    "sysafariview-server",
    "job-allocation",
    "file-sy-server",
    "kt-util",
];

/// 环境类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Test,
    Dev,
    Pro,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Test => "test",
            Environment::Dev => "dev",
            Environment::Pro => "pro",
        }
    }

    pub fn all() -> Vec<Environment> {
        vec![Environment::Test, Environment::Dev, Environment::Pro]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Environment::Test => "测试 (test)",
            Environment::Dev => "开发 (dev)",
            Environment::Pro => "生产 (pro)",
        }
    }
}

/// 构建方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildMethod {
    Local,
    GitHubActions,
}

impl BuildMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            BuildMethod::Local => "local",
            BuildMethod::GitHubActions => "github-actions",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            BuildMethod::Local => "本地构建 (local)",
            BuildMethod::GitHubActions => "GitHub Actions",
        }
    }

    /// 根据环境返回可用的构建方式顺序
    pub fn available_for(env: Environment) -> Vec<BuildMethod> {
        // pro 环境优先 github-actions
        if env == Environment::Pro {
            vec![BuildMethod::GitHubActions, BuildMethod::Local]
        } else {
            vec![BuildMethod::Local, BuildMethod::GitHubActions]
        }
    }
}

/// 触发类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    Manual,
    Tag,
}

impl TriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriggerType::Manual => "manual",
            TriggerType::Tag => "tag",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TriggerType::Manual => "手动触发 (manual)",
            TriggerType::Tag => "Tag 触发 (tag)",
        }
    }

    pub fn all() -> Vec<TriggerType> {
        vec![TriggerType::Manual, TriggerType::Tag]
    }
}

/// 获取模块对应的端口
pub fn get_module_port(module: &str) -> u16 {
    match module {
        "declaration-server" => 8501,
        "addr-server" => 8001,
        "setting-server" => 9501,
        "message-server" => 14001,
        "warehouse-server" => 8901,
        "operation-data" => 13001,
        "hscode-server" => 8601,
        "category-server" => 8401,
        "waybill" => 8701,
        "sysafariview-server" => 9301,
        "job-allocation" => 12001,
        "file-sy-server" => 9001,
        "kt-util" => 9709,
        _ => 8080,
    }
}

/// 获取模块对应的服务名（Eureka 注册名）
pub fn get_module_service_name(module: &str) -> String {
    let s: &str = match module {
        "declaration-server" => "declaration-service",
        "addr-server" => "address-service",
        "setting-server" => "setting-server",
        "message-server" => "message-service",
        "warehouse-server" => "warehouse-service",
        "operation-data" => "operation-data",
        "hscode-server" => "hscode-service",
        "category-server" => "category-service",
        "waybill" => "waybill-service",
        "sysafariview-server" => "sysafariview-service",
        "job-allocation" => "job-allocation",
        "file-sy-server" => "file-service",
        "kt-util" => "kt-util",
        other => other,
    };
    s.to_string()
}

/// 根据环境获取 Eureka URL
pub fn get_eureka_url(env: Environment) -> &'static str {
    match env {
        Environment::Pro => "http://eureka.pro.sysafari.com/eureka",
        Environment::Dev => "http://thinkserver-asl:8801/eureka",
        Environment::Test => "http://eureka.dev.sysafari.cn/eureka",
    }
}

/// 根据环境获取可用的服务器主机列表
pub fn get_server_hosts(env: Environment) -> Vec<&'static str> {
    match env {
        Environment::Pro => vec![
            "mq.pro.asl.com",
            "vault.pro.asl.com",
            "env.pro.asl.com",
            "basic.pro.asl.com",
            "ser.pro.asl.com",
        ],
        Environment::Dev => vec!["thinkserver-asl"],
        Environment::Test => vec!["env.dev.asl.com", "ser.dev.asl.com"],
    }
}

/// 获取滚动更新时可用的服务器 B 主机列表
pub fn get_server_b_hosts(use_rolling: bool) -> Vec<&'static str> {
    if !use_rolling {
        return vec!["N/A"];
    }
    vec![
        "ser.pro.asl.com",
        "env.pro.asl.com",
        "basic.pro.asl.com",
        "vault.pro.asl.com",
        "env.dev.asl.com",
        "ser.dev.asl.com",
    ]
}

/// 滚动更新默认值：pro 默认 true，其他默认 false
pub fn use_rolling_update_default(env: Environment) -> bool {
    env == Environment::Pro
}

/// 获取 Docker 镜像标签
pub fn get_image(env: Environment, module: &str, version: &str) -> String {
    let registry = match env {
        Environment::Pro => "registry.eu-central-1.aliyuncs.com",
        _ => "registry.cn-shanghai.aliyuncs.com",
    };
    format!("{}/asl/sysafari-server:{}-{}", registry, module, version)
}

/// 计算下线后等待时间（秒）
pub fn resolve_wait_time_after_down(env: Environment, module: &str, custom: Option<u32>) -> u32 {
    if let Some(val) = custom {
        if val > 0 {
            return val;
        }
    }

    match env {
        Environment::Test | Environment::Dev => 30,
        Environment::Pro => {
            if module == "declaration-server" {
                120
            } else {
                60
            }
        }
    }
}

/// CI/CD 配置参数（主状态）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CicdConfig {
    pub module: String,
    pub environment: Environment,
    pub version: String,
    pub build_method: BuildMethod,
    pub trigger_type: TriggerType,
    pub always_pull_image: bool,
    pub server_host: String,
    pub use_rolling_update: bool,
    pub server_b_host: String,
    pub wait_time_after_down: Option<u32>,
}

impl Default for CicdConfig {
    fn default() -> Self {
        let environment = Environment::Test;
        let rolling_default = use_rolling_update_default(environment);

        Self {
            module: MODULES[0].to_string(),
            environment,
            version: "develop".to_string(),
            build_method: BuildMethod::Local,
            trigger_type: TriggerType::Manual,
            always_pull_image: true,
            server_host: get_server_hosts(environment)[0].to_string(),
            use_rolling_update: rolling_default,
            server_b_host: get_server_b_hosts(false)[0].to_string(),
            wait_time_after_down: None,
        }
    }
}

impl CicdConfig {
    /// 生成可用于 Jenkins API 调用的参数字典
    pub fn to_jenkins_params(&self) -> serde_json::Value {
        serde_json::json!({
            "MODULE": self.module,
            "ENVIRONMENT": self.environment.as_str(),
            "VERSION": self.version,
            "BUILD_METHOD": self.build_method.as_str(),
            "TRIGGER_TYPE": self.trigger_type.as_str(),
            "ALWAYS_PULL_IMAGE": self.always_pull_image.to_string(),
            "SERVER_HOST": self.server_host,
            "USE_ROLLING_UPDATE": self.use_rolling_update.to_string(),
            "SERVER_B_HOST": self.server_b_host,
            "WAIT_TIME_AFTER_DOWN": self.wait_time_after_down
                .map(|v| v.to_string())
                .unwrap_or_default(),
        })
    }

    /// 获取部署计划摘要
    pub fn deployment_summary(&self) -> String {
        let service_name = get_module_service_name(&self.module);
        let port = get_module_port(&self.module);
        let image = get_image(self.environment, &self.module, &self.version);
        let eureka = get_eureka_url(self.environment);
        let wait_time = resolve_wait_time_after_down(
            self.environment,
            &self.module,
            self.wait_time_after_down,
        );

        format!(
            r#"═══════════════════════════════════════
 CI/CD 部署计划预览
═══════════════════════════════════════
 模块 (MODULE)          : {}
 服务名                 : {}
 端口                   : {}
 版本 (VERSION)         : {}
 环境 (ENVIRONMENT)     : {}
 触发方式 (TRIGGER_TYPE): {}
 构建方式 (BUILD_METHOD): {}
 服务器主机             : {}
 滚动更新               : {}
 服务器 B (滚动用)      : {}
 强制拉取镜像           : {}
 下线等待时间           : {}s
───────────────────────────────────────
 Docker 镜像            : {}
 Eureka URL             : {}
═══════════════════════════════════════"#,
            self.module,
            service_name,
            port,
            self.version,
            self.environment.display_name(),
            self.trigger_type.display_name(),
            self.build_method.display_name(),
            self.server_host,
            if self.use_rolling_update { "是" } else { "否" },
            if self.use_rolling_update {
                self.server_b_host.clone()
            } else {
                "N/A".to_string()
            },
            if self.always_pull_image { "是" } else { "否" },
            wait_time,
            image,
            eureka,
        )
    }
}
