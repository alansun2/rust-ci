# 🛠 CI/CD 部署客户端

基于 **Rust + GPUI** 的 Jenkins CI/CD 参数化构建桌面客户端。

从 Jenkinsfile 中提取所有参数定义、级联依赖、端口/服务名映射，以图形界面形式呈现，支持实时预览和一键触发 Jenkins 构建。

## ✨ 功能特性

| 特性 | 说明 |
|------|------|
| **可视化参数配置** | 对应 Jenkinsfile 中所有 parameters，提供友好的下拉选择和输入框 |
| **级联联动** | 正确还原 Jenkinsfile 中的 `CascadeChoiceParameter` 逻辑，如 ENVIRONMENT 变化自动重置 BUILD_METHOD / SERVER_HOST / USE_ROLLING_UPDATE 等 |
| **内置映射表** | 从 Jenkinsfile 的 switch 语句提取模块-端口、模块-服务名映射，自动填充 |
| **Docker 镜像预览** | 根据 ENVIRONMENT + MODULE + VERSION 实时计算完整镜像地址（pro 用 eu-central-1，其他用 cn-shanghai） |
| **Eureka URL 预览** | 根据环境自动选择 eureka.pro / thinkserver-asl / eureka.dev |
| **下线等待时间计算** | 复刻 Jenkinsfile `resolveWaitTimeAfterDown`：test/dev=30s, pro 默认 60s, declaration-server=120s |
| **一键触发 Jenkins** | 生成 `buildWithParameters` HTTP 请求（含 Basic Auth） |
| **JSON 参数导出** | 复制可用于 API 调用的完整参数字典 |

## 🏗 项目结构

```
cicd-client/
├── Cargo.toml                 # 依赖声明（gpui, serde, reqwest, anyhow）
├── README.md
└── src/
    ├── main.rs                # 入口
    ├── models.rs              # 数据模型（Environment, BuildMethod, TriggerType 等枚举 + 映射函数）
    ├── app.rs                 # GPUI 主视图（表单 + 预览面板）
    └── jenkins_client.rs      # Jenkins REST API 封装
```

## 📐 参数映射关系（来自 Jenkinsfile）

### 模块列表
```
declaration-server, addr-server, setting-server, message-server,
warehouse-server, operation-data, hscode-server, category-server,
waybill, sysafariview-server, job-allocation, file-sy-server, kt-util
```

### 环境 → 服务器主机
| ENVIRONMENT | SERVER_HOST 候选 |
|-------------|------------------|
| **pro**     | `mq.pro.asl.com`, `vault.pro.asl.com`, `env.pro.asl.com`, `basic.pro.asl.com`, `ser.pro.asl.com` |
| **dev**     | `thinkserver-asl` |
| **test**    | `env.dev.asl.com`, `ser.dev.asl.com` |

### BUILD_METHOD 级联
- ENVIRONMENT == "pro" → 候选顺序：`github-actions`, `local`
- 其他 → 候选顺序：`local`, `github-actions`

### USE_ROLLING_UPDATE 默认值
- **pro** → 默认 `true`
- **dev/test** → 默认 `false`

### Eureka URL
| ENV | URL |
|-----|-----|
| pro | `http://eureka.pro.sysafari.com/eureka` |
| dev | `http://thinkserver-asl:8801/eureka` |
| test | `http://eureka.dev.sysafari.cn/eureka` |

### 模块 → 端口 / 服务名（节选）
| 模块 | 端口 | Eureka 服务名 |
|------|------|--------------|
| declaration-server | 8501 | declaration-service |
| addr-server | 8001 | address-service |
| warehouse-server | 8901 | warehouse-service |
| ... | ... | ... |

### Docker 镜像仓库
| ENV | Registry |
|-----|----------|
| pro | `registry.eu-central-1.aliyuncs.com/asl/sysafari-server:{MODULE}-{VERSION}` |
| dev/test | `registry.cn-shanghai.aliyuncs.com/asl/sysafari-server:{MODULE}-{VERSION}` |

## 🚀 构建与运行

### 前置条件

- **Rust 1.80+**
- **GPUI 构建依赖**（Linux）：
  ```bash
  sudo apt-get install build-essential cmake pkg-config \
      libxkbcommon-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev \
      librsvg2-dev libwebkit2gtk-4.1-dev
  ```
- **macOS**：Xcode Command Line Tools + WebKit 框架（系统自带）

### 构建

```bash
cd cicd-client
cargo build --release
```

### 运行

```bash
cargo run --release
```

## 🔌 Jenkins API 集成

填写 Jenkins 地址、用户名、API Token 后，客户端会向：
```
POST {JENKINS_URL}/job/{YOUR_JOB}/buildWithParameters
```
发送表单请求。

支持的额外功能（在 `jenkins_client.rs` 中）：
- 查询构建队列 `/queue/api/json`
- 查询最新构建信息 `/job/{name}/lastBuild/api/json`
- 拉取控制台日志 `/job/{name}/{number}/consoleText`

## 📋 生成的 Jenkins 参数示例

```json
{
  "MODULE": "declaration-server",
  "ENVIRONMENT": "test",
  "VERSION": "develop",
  "BUILD_METHOD": "local",
  "TRIGGER_TYPE": "manual",
  "ALWAYS_PULL_IMAGE": "true",
  "SERVER_HOST": "env.dev.asl.com",
  "USE_ROLLING_UPDATE": "false",
  "SERVER_B_HOST": "N/A",
  "WAIT_TIME_AFTER_DOWN": ""
}
```

与 Jenkinsfile 中的 parameters 定义 **1:1 对应**，可直接用于 API 调用。

## 🔮 后续扩展方向

- [ ] 实时监控 Jenkins 构建状态 + 控制台日志流
- [ ] GitHub Actions Run 查询（对应 Jenkinsfile `waitForGithubActions`）
- [ ] Eureka 实例健康检查（对应 `waitForEurekaInstances`）
- [ ] SSH 远程部署测试连接（对应 `createRemote` / `sshCommand`）
- [ ] 配置持久化（保存常用参数组合）
- [ ] 钉钉通知 Webhook 集成（对应 `sendDingTalkNotification`）

## 📄 License

MIT
