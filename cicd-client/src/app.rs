use gpui::{
    AppContext, Context, IntoElement, Render, Styled, WindowContext, AnyElement,
};
use serde::{Deserialize, Serialize};

use crate::models::{
    BuildMethod, CicdConfig, Environment, MODULES, TriggerType,
    get_eureka_url, get_image, get_module_port, get_module_service_name,
    get_server_b_hosts, get_server_hosts, resolve_wait_time_after_down,
    use_rolling_update_default,
};

/// 应用主视图 - 基于 Zed GPUI 框架
pub struct CicdClientApp {
    config: CicdConfig,
}

impl CicdClientApp {
    pub fn new(cx: &mut WindowContext) -> Self {
        Self {
            config: CicdConfig::default(),
        }
    }

    /// 环境变化时重置所有依赖 ENVIRONMENT 的级联参数
    fn handle_environment_change(&mut self, cx: &mut WindowContext) {
        let env = self.config.environment;

        // BUILD_METHOD: pro 优先 github-actions，其他优先 local
        let available = BuildMethod::available_for(env);
        self.config.build_method = available[0];

        // SERVER_HOST: 环境切换后重置
        let hosts = get_server_hosts(env);
        if hosts.iter().all(|h| h != &self.config.server_host.as_str()) {
            self.config.server_host = hosts[0].to_string();
        }

        // USE_ROLLING_UPDATE: pro 默认 true，其他默认 false
        self.config.use_rolling_update = use_rolling_update_default(env);

        // SERVER_B_HOST
        let b_hosts = get_server_b_hosts(self.config.use_rolling_update);
        self.config.server_b_host = b_hosts[0].to_string();

        cx.notify();
    }

    /// 滚动更新选项变化时更新 SERVER_B_HOST
    fn handle_rolling_change(&mut self, cx: &mut WindowContext) {
        let b_hosts = get_server_b_hosts(self.config.use_rolling_update);
        self.config.server_b_host = b_hosts[0].to_string();
        cx.notify();
    }

    /// 渲染标题区域
    fn header(&self) -> AnyElement {
        gpui::div()
            .text_xl()
            .font_weight(gpui::FontWeight::Bold)
            .text_color(gpui::Hsla::from_hex("#ffd166"))
            .child("🛠 CI/CD 部署客户端")
            .into_any_element()
    }

    fn subtitle(&self) -> AnyElement {
        gpui::div()
            .mt_1()
            .text_sm()
            .text_color(gpui::Hsla::from_hex("#a0a0b0"))
            .child("基于 Jenkinsfile 可视化配置参数 · 支持级联联动 · 一键触发部署")
            .into_any_element()
    }

    /// 渲染预览面板
    fn summary_panel(&self) -> AnyElement {
        let cfg = &self.config;
        let service_name = get_module_service_name(&cfg.module);
        let port = get_module_port(&cfg.module);
        let image = get_image(cfg.environment, &cfg.module, &cfg.version);
        let eureka = get_eureka_url(cfg.environment);
        let wait_time = resolve_wait_time_after_down(
            cfg.environment,
            &cfg.module,
            cfg.wait_time_after_down,
        );

        let rolling = if cfg.use_rolling_update { "是" } else { "否" };
        let b_host = if cfg.use_rolling_update {
            cfg.server_b_host.clone()
        } else {
            "N/A".to_string()
        };

        let always_pull = if cfg.always_pull_image { "是" } else { "否" };

        gpui::div()
            .w_full()
            .mt_6()
            .rounded(gpui::px(8.0))
            .bg(gpui::Hsla::from_hex("#1a1a2a"))
            .border_1()
            .border_color(gpui::Hsla::from_hex("#3a3a4a"))
            .p_4()
            .child(
                gpui::div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::Bold)
                    .text_color(gpui::Hsla::from_hex("#8ecae6"))
                    .child("📋 部署计划预览"),
            )
            .child(
                gpui::div()
                    .mt_3()
                    .text_sm()
                    .text_color(gpui::Hsla::from_hex("#cfd8dc"))
                    .child(format!("模块           : {}\n", cfg.module))
                    .child(format!("服务名         : {} (端口 {})\n", service_name, port))
                    .child(format!("版本           : {}\n", cfg.version))
                    .child(format!("环境           : {}\n", cfg.environment.display_name()))
                    .child(format!("构建方式       : {}\n", cfg.build_method.display_name()))
                    .child(format!("触发类型       : {}\n", cfg.trigger_type.display_name()))
                    .child(format!("服务器主机     : {}\n", cfg.server_host))
                    .child(format!("滚动更新       : {}   服务器 B : {}\n", rolling, b_host))
                    .child(format!("强制拉取镜像   : {}   下线等待 : {}s\n", always_pull, wait_time)),
            )
            .child(
                gpui::div()
                    .mt_3()
                    .pt_3()
                    .border_t_1()
                    .border_color(gpui::Hsla::from_hex("#3a3a4a"))
                    .text_xs()
                    .text_color(gpui::Hsla::from_hex("#8ecae6"))
                    .child(format!("🐳 Docker 镜像 : {}\n", image))
                    .child(format!("🔗 Eureka URL  : {}", eureka)),
            )
            .into_any_element()
    }

    /// 渲染操作按钮区
    fn action_buttons(&self) -> AnyElement {
        gpui::div()
            .mt_6()
            .flex()
            .gap_3()
            .child(
                gpui::div()
                    .rounded(gpui::px(6.0))
                    .bg(gpui::Hsla::from_hex("#06d6a0"))
                    .px_5()
                    .py_2()
                    .text_sm()
                    .font_weight(gpui::FontWeight::Bold)
                    .text_color(gpui::Hsla::from_hex("#0f0f1a"))
                    .cursor_pointer()
                    .child("🚀 触发 Jenkins 构建"),
            )
            .child(
                gpui::div()
                    .rounded(gpui::px(6.0))
                    .bg(gpui::Hsla::from_hex("#118ab2"))
                    .px_5()
                    .py_2()
                    .text_sm()
                    .font_weight(gpui::FontWeight::Bold)
                    .text_color(gpui::Hsla::white)
                    .cursor_pointer()
                    .child("📋 复制 JSON 参数"),
            )
            .child(
                gpui::div()
                    .rounded(gpui::px(6.0))
                    .bg(gpui::Hsla::from_hex("#3a3a4a"))
                    .px_5()
                    .py_2()
                    .text_sm()
                    .text_color(gpui::Hsla::from_hex("#cfd8dc"))
                    .cursor_pointer()
                    .child("💾 保存为默认配置"),
            )
            .into_any_element()
    }

    /// 渲染模块选择
    fn module_field(&self) -> AnyElement {
        let options = MODULES.to_vec();
        let current = self.config.module.clone();
        label_with_values(
            "模块 (MODULE)",
            current,
            options.into_iter().map(|s| s.to_string()).collect(),
            move |this: &mut Self, v, cx| {
                this.config.module = v;
                cx.notify();
            },
        )
    }

    /// 环境选择
    fn environment_field(&self) -> AnyElement {
        let options = Environment::all();
        let labels: Vec<String> = options.iter().map(|e| e.display_name().to_string()).collect();
        let current = self.config.environment.display_name().to_string();

        let current_idx = options.iter().position(|&e| e == self.config.environment).unwrap_or(0);

        label_with_select(
            "环境 (ENVIRONMENT)",
            current_idx,
            labels,
            move |this: &mut Self, idx: usize, cx| {
                this.config.environment = options[idx];
                this.handle_environment_change(cx);
            },
        )
    }

    /// 版本号输入
    fn version_field(&self) -> AnyElement {
        label_with_text(
            "版本号 (VERSION)",
            self.config.version.clone(),
            "develop",
            move |this: &mut Self, v, _cx| {
                this.config.version = v;
            },
        )
    }

    /// 构建方式选择
    fn build_method_field(&self) -> AnyElement {
        let options = BuildMethod::available_for(self.config.environment);
        let labels: Vec<String> = options.iter().map(|m| m.display_name().to_string()).collect();
        let current_idx = options
            .iter()
            .position(|&m| m == self.config.build_method)
            .unwrap_or(0);

        label_with_select(
            "构建方式 (BUILD_METHOD)",
            current_idx,
            labels,
            move |this: &mut Self, idx: usize, cx| {
                this.config.build_method = options[idx];
                cx.notify();
            },
        )
    }

    /// 触发类型选择
    fn trigger_type_field(&self) -> AnyElement {
        let options = TriggerType::all();
        let labels: Vec<String> = options.iter().map(|t| t.display_name().to_string()).collect();
        let current_idx = options
            .iter()
            .position(|&t| t == self.config.trigger_type)
            .unwrap_or(0);

        label_with_select(
            "触发类型 (TRIGGER_TYPE)",
            current_idx,
            labels,
            move |this: &mut Self, idx: usize, cx| {
                this.config.trigger_type = options[idx];
                cx.notify();
            },
        )
    }

    /// 服务器主机选择
    fn server_host_field(&self) -> AnyElement {
        let options = get_server_hosts(self.config.environment);
        let labels: Vec<String> = options.iter().map(|s| s.to_string()).collect();
        let current = self.config.server_host.clone();

        label_with_values(
            "服务器主机 (SERVER_HOST)",
            current,
            labels,
            move |this: &mut Self, v, cx| {
                this.config.server_host = v;
                cx.notify();
            },
        )
    }

    /// 服务器 B 主机（仅滚动更新时显示）
    fn server_b_host_field(&self) -> AnyElement {
        let options = get_server_b_hosts(self.config.use_rolling_update);
        let labels: Vec<String> = options.iter().map(|s| s.to_string()).collect();
        let current = self.config.server_b_host.clone();

        label_with_values(
            "服务器 B (SERVER_B_HOST)",
            current,
            labels,
            move |this: &mut Self, v, cx| {
                this.config.server_b_host = v;
                cx.notify();
            },
        )
    }

    /// 下线等待时间
    fn wait_time_field(&self) -> AnyElement {
        label_with_text(
            "下线后等待时间 (秒, 留空自动)",
            self.config
                .wait_time_after_down
                .map(|v| v.to_string())
                .unwrap_or_default(),
            "自动 (test/dev=30, pro=60, declaration=120)",
            move |this: &mut Self, v, _cx| {
                this.config.wait_time_after_down = v.parse::<u32>().ok();
            },
        )
    }

    /// 强制拉取镜像开关
    fn always_pull_field(&self, cx: &mut WindowContext) -> AnyElement {
        checkbox_field(
            "强制拉取镜像 (ALWAYS_PULL_IMAGE)",
            self.config.always_pull_image,
            move |this: &mut Self, v, cx| {
                this.config.always_pull_image = v;
                cx.notify();
            },
            cx,
        )
    }

    /// 滚动更新开关
    fn rolling_field(&self, cx: &mut WindowContext) -> AnyElement {
        checkbox_field(
            "使用滚动更新 (USE_ROLLING_UPDATE)",
            self.config.use_rolling_update,
            move |this: &mut Self, v, cx| {
                this.config.use_rolling_update = v;
                this.handle_rolling_change(cx);
            },
            cx,
        )
    }
}

// ============================================================================
// 通用表单组件渲染器
// ============================================================================

/// 带标签和下拉选择的字段
fn label_with_values(
    label: &str,
    current: String,
    _options: Vec<String>,
    on_select: impl FnMut(&mut CicdClientApp, String, &mut WindowContext) + 'static,
) -> AnyElement {
    gpui::div()
        .w_full()
        .rounded(gpui::px(6.0))
        .border_1()
        .border_color(gpui::Hsla::from_hex("#3a3a4a"))
        .bg(gpui::Hsla::from_hex("#1a1a2a"))
        .p_3()
        .child(
            gpui::div()
                .text_xs()
                .text_color(gpui::Hsla::from_hex("#a0a0b0"))
                .child(label),
        )
        .child(
            gpui::div()
                .mt_1()
                .text_sm()
                .text_color(gpui::Hsla::white)
                .cursor_pointer()
                .child(current),
        )
        .into_any_element()
}

fn label_with_select(
    label: &str,
    _current_idx: usize,
    _options: Vec<String>,
) -> AnyElement {
    gpui::div()
        .w_full()
        .rounded(gpui::px(6.0))
        .border_1()
        .border_color(gpui::Hsla::from_hex("#3a3a4a"))
        .bg(gpui::Hsla::from_hex("#1a1a2a"))
        .p_3()
        .child(
            gpui::div()
                .text_xs()
                .text_color(gpui::Hsla::from_hex("#a0a0b0"))
                .child(label),
        )
        .child(
            gpui::div()
                .mt_1()
                .text_sm()
                .text_color(gpui::Hsla::white)
                .cursor_pointer(),
        )
        .into_any_element()
}

/// 带标签的文本输入字段
fn label_with_text(
    label: &str,
    value: String,
    placeholder: &str,
    on_change: impl FnMut(&mut CicdClientApp, String, &mut WindowContext) + 'static,
) -> AnyElement {
    let display = if value.is_empty() {
        placeholder.to_string()
    } else {
        value
    };
    let is_placeholder = value.is_empty();

    gpui::div()
        .w_full()
        .rounded(gpui::px(6.0))
        .border_1()
        .border_color(gpui::Hsla::from_hex("#3a3a4a"))
        .bg(gpui::Hsla::from_hex("#1a1a2a"))
        .p_3()
        .child(
            gpui::div()
                .text_xs()
                .text_color(gpui::Hsla::from_hex("#a0a0b0"))
                .child(label),
        )
        .child(
            gpui::div()
                .mt_1()
                .text_sm()
                .text_color(if is_placeholder {
                    gpui::Hsla::from_hex("#606070")
                } else {
                    gpui::Hsla::white
                })
                .cursor_text()
                .child(display),
        )
        .into_any_element()
}

fn checkbox_field(
    label: &str,
    checked: bool,
    _on_toggle: impl FnMut(&mut CicdClientApp, bool, &mut WindowContext) + 'static,
    cx: &mut WindowContext,
) -> AnyElement {
    gpui::div()
        .w_full()
        .rounded(gpui::px(6.0))
        .border_1()
        .border_color(gpui::Hsla::from_hex("#3a3a4a"))
        .bg(gpui::Hsla::from_hex("#1a1a2a"))
        .p_3()
        .flex()
        .items_center()
        .gap_3()
        .child(
            gpui::div()
                .size(gpui::px(18.0))
                .rounded(gpui::px(4.0))
                .border_1()
                .border_color(gpui::Hsla::from_hex("#5a5a6a"))
                .bg(if checked {
                    gpui::Hsla::from_hex("#06d6a0")
                } else {
                    gpui::Hsla::from_hex("#2a2a3a")
                }),
        )
        .child(
            gpui::div()
                .text_sm()
                .text_color(gpui::Hsla::from_hex("#cfd8dc"))
                .child(label),
        )
        .into_any_element()
}

// ============================================================================
// Render trait 实现
// ============================================================================

impl Render for CicdClientApp {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let cfg = &self.config;

        // 主容器：暗色主题
        gpui::div()
            .size_full()
            .bg(gpui::Hsla::from_hex("#0f0f1a"))
            .text_color(gpui::Hsla::white)
            .p_8()
            .overflow_y_scroll()
            .child(self.header())
            .child(self.subtitle())
            // 表单字段网格（2列）
            .child(
                gpui::div()
                    .mt_6()
                    .grid(gpui::GridPoints::repeat(
                        2,
                        |_| gpui::px(340.0),
                    ))
                    .gap_4()
                    .child(self.module_field())
                    .child(self.environment_field())
                    .child(self.version_field())
                    .child(self.build_method_field())
                    .child(self.trigger_type_field())
                    .child(self.always_pull_field(cx))
                    .child(self.server_host_field())
                    .child(self.rolling_field(cx))
                    .child_if(cfg.use_rolling_update, self.server_b_host_field())
                    .child(self.wait_time_field()),
            )
            .child(self.summary_panel())
            .child(self.action_buttons())
    }
}
