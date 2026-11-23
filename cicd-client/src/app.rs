use gpui::{
    AppContext, Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};

use crate::models::{
    BuildMethod, CicdConfig, Environment, MODULES, TriggerType, get_eureka_url, get_image,
    get_module_port, get_module_service_name, get_server_b_hosts, get_server_hosts,
    resolve_wait_time_after_down, use_rolling_update_default,
};

// ============================================================================
// View
// ============================================================================

pub struct CicdClientApp {
    config: CicdConfig,
}

impl CicdClientApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            config: CicdConfig::default(),
        }
    }

    fn handle_env_change(&mut self) {
        let env = self.config.environment;
        let methods = BuildMethod::available_for(env);
        self.config.build_method = methods[0];

        let hosts = get_server_hosts(env);
        if hosts.iter().all(|h| h != &self.config.server_host.as_str()) {
            self.config.server_host = hosts[0].to_string();
        }
        self.config.use_rolling_update = use_rolling_update_default(env);
        let b = get_server_b_hosts(self.config.use_rolling_update);
        self.config.server_b_host = b[0].to_string();
    }

    fn handle_rolling_change(&mut self) {
        let b = get_server_b_hosts(self.config.use_rolling_update);
        self.config.server_b_host = b[0].to_string();
    }
}

// ============================================================================
// Render
// ============================================================================

impl Render for CicdClientApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cfg = &self.config;

        div()
            .size_full()
            .bg(rgb(0x0f0f1a))
            .text_color(rgb(0xffffff))
            .p_8()
            // 标题
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0xffd166))
                    .child("🛠 CI/CD 部署客户端"),
            )
            .child(
                div()
                    .mt_1()
                    .text_sm()
                    .text_color(rgb(0xa0a0b0))
                    .child("基于 Jenkinsfile 可视化配置参数 · 支持级联联动"),
            )
            // 预览面板
            .child(self.summary_panel())
    }
}

// ============================================================================
// Preview Panel
// ============================================================================

impl CicdClientApp {
    fn summary_panel(&self) -> impl IntoElement {
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

        let rolling_str = if cfg.use_rolling_update { "是" } else { "否" };
        let b_host = if cfg.use_rolling_update {
            cfg.server_b_host.clone()
        } else {
            "N/A".to_string()
        };
        let pull_str = if cfg.always_pull_image { "是" } else { "否" };

        div()
            .w_full()
            .mt_6()
            .rounded(px(8.0))
            .bg(rgb(0x1a1a2a))
            .border_1()
            .border_color(rgb(0x3a3a4a))
            .p_4()
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0x8ecae6))
                    .child("📋 部署计划预览"),
            )
            .child(
                div()
                    .mt_3()
                    .text_sm()
                    .text_color(rgb(0xcfd8dc))
                    .child(format!("模块           : {}\n", cfg.module))
                    .child(format!("服务名         : {} (端口 {})\n", service_name, port))
                    .child(format!("版本           : {}\n", cfg.version))
                    .child(format!(
                        "环境           : {}\n",
                        cfg.environment.display_name()
                    ))
                    .child(format!(
                        "构建方式       : {}\n",
                        cfg.build_method.display_name()
                    ))
                    .child(format!(
                        "触发类型       : {}\n",
                        cfg.trigger_type.display_name()
                    ))
                    .child(format!("服务器主机     : {}\n", cfg.server_host))
                    .child(format!(
                        "滚动更新       : {}   服务器 B : {}\n",
                        rolling_str, b_host
                    ))
                    .child(format!(
                        "强制拉取镜像   : {}   下线等待 : {}s\n",
                        pull_str, wait_time
                    )),
            )
            .child(
                div()
                    .mt_3()
                    .pt_3()
                    .border_t_1()
                    .border_color(rgb(0x3a3a4a))
                    .text_xs()
                    .text_color(rgb(0x8ecae6))
                    .child(format!("🐳 Docker 镜像 : {}\n", image))
                    .child(format!("🔗 Eureka URL  : {}", eureka)),
            )
    }
}
