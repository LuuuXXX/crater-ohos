# Crater-OHOS 架构设计文档

## 1. 项目定位

crater-ohos 是一个**独立的服务端应用**，专为鸿蒙（OpenHarmony）生态系统设计，用于自动化验证 Rust 三方库的兼容性。

### 核心特性

- **独立服务端**：crater-ohos 本身是完整的服务端应用，不依赖任何外部 Bot
- **平台无关**：通过平台抽象层支持多种代码托管平台（GitHub、Gitee、GitLab、GitCode 等）
- **API 优先**：提供完整的 REST API，供外部 Bot 或其他客户端调用
- **Bot 外接**：Bot 是独立的外部组件，通过 API 与服务端交互，不内置于 crater-ohos
- **Webhook 支持**：服务端可主动通知 Bot 实验状态变化
- **独立部署**：服务端与 Bot 可以独立部署、独立扩展

### 与上游 rust-lang/crater 的区别

| 维度 | rust-lang/crater | crater-ohos |
|------|------------------|-------------|
| 架构模式 | 单体应用 | 服务端 + 外部 Bot |
| 平台支持 | 仅 GitHub | 多平台（GitHub、Gitee、GitLab、GitCode 等） |
| API 接口 | 无独立 API | 完整 REST API |
| Bot 集成 | 内置 GitHub Bot | 外部独立 Bot 通过 API 调用 |
| 部署方式 | 一体化部署 | 服务端与 Bot 独立部署 |
| 扩展性 | 需修改核心代码 | 通过 API 和平台适配器扩展 |
| Webhook | 无 | 支持 webhook 回调 |

## 2. 整体架构

### 2.1 系统架构图

```
┌────────────────────────────────────────────────────────────────────────┐
│                        外部 Bot 层（独立组件）                           │
│                                                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │  Gitee Bot   │  │  GitHub Bot  │  │  GitLab Bot  │  │ 自定义 Bot │ │
│  │  (独立仓库)   │  │  (独立仓库)   │  │  (独立仓库)   │  │           │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘ │
│         │                 │                 │                 │       │
│         └─────────────────┴─────────────────┴─────────────────┘       │
│                                  │                                     │
│                        HTTP REST API (Bearer Token)                   │
│                                  │                                     │
│                           ┌──────▼──────┐                             │
│                           │   Webhook   │                             │
│                           │  Callbacks  │                             │
│                           └──────┬──────┘                             │
└──────────────────────────────────┼────────────────────────────────────┘
                                   │
┌──────────────────────────────────▼────────────────────────────────────┐
│                     crater-ohos 服务端（本仓库）                        │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                    API Layer (Phase 6)                            │ │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────────────────┐│ │
│  │  │ REST API    │  │ CLI Commands │  │ Auth Middleware          ││ │
│  │  │ - 实验管理   │  │ - 本地执行    │  │ - Bearer Token           ││ │
│  │  │ - Agent管理 │  │ - 报告生成    │  │ - Permission Control     ││ │
│  │  │ - 健康检查   │  │              │  │                          ││ │
│  │  └─────────────┘  └──────────────┘  └──────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                   │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                   Service Layer (Phase 4)                         │ │
│  │  ┌──────────────┐  ┌───────────────┐  ┌─────────────────────────┐│ │
│  │  │ Actions      │  │ Agent Manager │  │ Callback Service        ││ │
│  │  │ - 创建实验    │  │ - 注册/心跳    │  │ - Webhook 通知          ││ │
│  │  │ - 编辑/删除   │  │ - 任务分配     │  │ - 重试机制              ││ │
│  │  │ - 运行/中止   │  │ - 状态监控     │  │ - 超时控制              ││ │
│  │  └──────────────┘  └───────────────┘  └─────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                   │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                    Domain Layer (Phase 2)                         │ │
│  │  ┌──────────────┐  ┌─────────────┐  ┌──────────────────────────┐│ │
│  │  │ Experiments  │  │ Crates      │  │ Toolchains               ││ │
│  │  │ - 实验模型    │  │ - Crate源   │  │ - 工具链管理              ││ │
│  │  │ - 状态管理    │  │ - 列表管理   │  │ - 版本解析                ││ │
│  │  └──────────────┘  └─────────────┘  └──────────────────────────┘│ │
│  │  ┌──────────────┐  ┌─────────────┐                              │ │
│  │  │ Results      │  │ Platform    │                              │ │
│  │  │ - 测试结果    │  │ - Issue抽象 │                              │ │
│  │  └──────────────┘  └─────────────┘                              │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                   │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                  Execution Layer (Phase 3)                        │ │
│  │  ┌──────────────┐  ┌─────────────┐  ┌──────────────────────────┐│ │
│  │  │ Runner       │  │ Report      │  │ Worker                   ││ │
│  │  │ - 构建执行    │  │ - HTML生成  │  │ - 任务调度                ││ │
│  │  │ - 测试执行    │  │ - MD生成    │  │ - 资源监控                ││ │
│  │  └──────────────┘  └─────────────┘  └──────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                   │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │               Infrastructure Layer (Phase 1)                      │ │
│  │  ┌──────────────┐  ┌─────────────┐  ┌──────────────────────────┐│ │
│  │  │ Database     │  │ Config      │  │ Utils                    ││ │
│  │  │ - SQLite     │  │ - TOML解析  │  │ - HTTP Client            ││ │
│  │  │ - 迁移系统    │  │ - ACL配置   │  │ - Size处理               ││ │
│  │  │ - 连接池      │  │             │  │ - Hex编码                ││ │
│  │  └──────────────┘  └─────────────┘  └──────────────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                   │                                    │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │            Platform Abstraction (Phase 5)                         │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐│ │
│  │  │ GitHub   │  │ Gitee    │  │ GitLab   │  │ GitCode          ││ │
│  │  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Adapter          ││ │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘│ │
│  └──────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.2 部署架构

```
┌─────────────────────────────────────────────────────────────┐
│                        用户请求                              │
│        (在 GitHub/Gitee/GitLab/GitCode 发布命令)             │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────┐
        │      代码托管平台 Webhook      │
        │    (Issue Comment, PR等)      │
        └───────────────┬───────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────┐
│                    Bot 服务                             │
│  ┌──────────────────────────────────────────────────┐  │
│  │  1. 接收平台 webhook                              │  │
│  │  2. 解析用户命令                                  │  │
│  │  3. 调用 crater-ohos API                          │  │
│  │  4. 监听 crater-ohos webhook 回调                 │  │
│  │  5. 发布结果到平台                                │  │
│  └──────────────────────────────────────────────────┘  │
└───────────┬────────────────────────────┬───────────────┘
            │ API Request                │ Webhook
            │ (Bearer Token)             │ Callback
            ▼                            ▼
┌────────────────────────────────────────────────────────┐
│              crater-ohos 服务端                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │  REST API Server                                 │  │
│  │  - 端口: 3000 (可配置)                            │  │
│  │  - 认证: Bearer Token                            │  │
│  │  - 实验管理                                       │  │
│  │  - Agent 管理                                     │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Callback Service                                │  │
│  │  - 实验状态变化通知                               │  │
│  │  - 重试机制（默认3次）                            │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  SQLite Database                                 │  │
│  │  - 实验数据                                       │  │
│  │  - 测试结果                                       │  │
│  │  - Agent 状态                                     │  │
│  │  - API Tokens                                    │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

## 3. 核心组件说明

### 3.1 API Layer（API 层）

API 层是 crater-ohos 与外部交互的主要接口，提供 REST API 和 CLI 两种方式。

#### REST API

**端点列表：**

| 端点 | 方法 | 功能 | 认证 |
|------|------|------|------|
| `/api/v1/health` | GET | 健康检查 | 否 |
| `/api/v1/config` | GET | 获取配置 | 是 |
| `/api/v1/experiments` | POST | 创建实验 | 是 |
| `/api/v1/experiments` | GET | 列出实验 | 是 |
| `/api/v1/experiments/{name}` | GET | 获取实验详情 | 是 |
| `/api/v1/experiments/{name}` | PUT | 编辑实验 | 是 |
| `/api/v1/experiments/{name}` | DELETE | 删除实验 | 是 |
| `/api/v1/experiments/{name}/run` | POST | 运行实验 | 是 |
| `/api/v1/experiments/{name}/abort` | POST | 中止实验 | 是 |
| `/api/v1/agents/register` | POST | 注册 Agent | 是 |
| `/api/v1/agents/{id}/heartbeat` | POST | Agent 心跳 | 是 |
| `/api/v1/agents` | GET | 列出 Agent | 是 |
| `/api/v1/agents/{id}` | GET | 获取 Agent 详情 | 是 |

**统一响应格式：**

```json
{
  "success": true,
  "data": { /* 业务数据 */ }
}
```

或错误响应：

```json
{
  "success": false,
  "error": "错误信息"
}
```

#### CLI 命令

| 命令 | 功能 |
|------|------|
| `prepare-local` | 准备本地环境 |
| `define-ex` | 定义实验 |
| `run-graph` | 运行实验 |
| `gen-report` | 生成报告 |
| `list-ex` | 列出实验 |
| `delete-ex` | 删除实验 |
| `abort-ex` | 中止实验 |
| `server` | 启动 API 服务器 |

#### 认证机制

- **Bearer Token 认证**：所有需要认证的端点使用 Bearer Token
- **权限控制**：基于 Permission 枚举的细粒度权限控制
  - `Admin`：管理员权限
  - `ReadExperiments`：读取实验
  - `WriteExperiments`：创建/编辑实验
  - `DeleteExperiments`：删除实验
  - `ManageAgents`：管理 Agent

### 3.2 Service Layer（服务层）

服务层实现核心业务逻辑。

#### ExperimentActions（实验操作）

实验的完整生命周期管理：

- **create()**：创建新实验，状态为 `queued`
- **edit()**：编辑实验（仅 `queued` 状态可编辑）
- **delete()**：删除实验（仅 `queued` 状态可删除）
- **run()**：运行实验，状态转为 `running`
- **complete()**：完成实验，状态转为 `completed`
- **abort()**：中止实验，状态转为 `aborted`
- **get()**：获取实验详情
- **list()**：列出所有实验

**状态转换：**

```
queued → running → completed
  ↓         ↓
  ↓       aborted
deleted
```

#### AgentManager（Agent 管理）

管理执行任务的 Agent：

- **register()**：注册新 Agent
- **heartbeat()**：Agent 心跳，更新最后活跃时间
- **assign_task()**：分配任务给 Agent
- **get()** / **list()**：查询 Agent 信息

#### CallbackService（回调服务）

实验状态变化时主动通知 Bot：

- **notify()**：发送 webhook 通知
- **重试机制**：失败时自动重试（默认3次）
- **超时控制**：HTTP 请求超时（默认30秒）

**回调事件类型：**

- `ExperimentCreated`：实验已创建
- `ExperimentCompleted`：实验已完成
- `ExperimentFailed`：实验失败
- `ExperimentAborted`：实验中止

### 3.3 Domain Layer（领域层）

领域层定义核心业务概念和规则。

#### Experiment（实验）

```rust
pub struct Experiment {
    pub name: String,
    pub toolchains: Vec<Toolchain>,
    pub mode: Mode,
    pub crates: CrateSelect,
    pub status: Status,
    pub priority: i32,
    // ...
}
```

**实验模式（Mode）：**

- `BuildAndTest`：构建并测试
- `BuildOnly`：仅构建
- `CheckOnly`：仅检查
- `Clippy`：运行 Clippy
- `Rustdoc`：生成文档

**实验状态（Status）：**

- `Queued`：等待执行
- `Running`：执行中
- `Completed`：已完成
- `Aborted`：已中止

#### Crate（包）

支持多种 crate 来源：

- `Registry`：crates.io
- `GitHub`：GitHub 仓库
- `Local`：本地路径
- `Git`：Git 仓库
- `Path`：指定路径

#### PlatformIssue（平台 Issue）

平台无关的 Issue 抽象：

```rust
pub struct PlatformIssue {
    pub platform: String,
    pub api_url: String,
    pub html_url: String,
    pub identifier: String,
}
```

### 3.4 Execution Layer（执行层）

执行层负责实际的构建和测试。

#### Runner（运行器）

- **任务执行**：执行构建和测试任务
- **资源监控**：监控磁盘空间、内存使用
- **沙箱隔离**：使用 Docker 容器隔离执行环境

#### Report（报告）

- **HTML 报告**：生成完整的 HTML 报告
- **Markdown 报告**：生成简洁的 Markdown 报告
- **结果分析**：分析测试结果，统计成功/失败率

### 3.5 Infrastructure Layer（基础设施层）

#### Database（数据库）

- **SQLite**：轻量级数据库
- **连接池**：使用 r2d2 管理连接池
- **迁移系统**：自动化数据库模式迁移
- **事务支持**：ACID 事务保证

**主要表：**

- `experiments`：实验配置和状态
- `experiment_metadata`：实验元数据（callback URL、平台等）
- `results`：构建和测试结果
- `agents`：Agent 信息
- `api_tokens`：API Token

#### Config（配置）

从 `config.toml` 加载配置：

- **demo-crates**：演示 crate 列表
- **sandbox**：沙箱配置（内存限制、日志大小）
- **server.acl**：访问控制
- **server.callback**：回调配置（超时、重试）
- **platforms**：平台配置（GitHub、Gitee、GitLab、GitCode）

### 3.6 Platform Abstraction（平台抽象层）

平台抽象层提供统一的平台操作接口。

#### PlatformAdapter Trait

```rust
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    fn platform_type(&self) -> PlatformType;
    async fn get_issue(&self, repo: &str, number: &str) -> Fallible<PlatformIssue>;
    async fn post_comment(&self, repo: &str, number: &str, body: &str) -> Fallible<()>;
    async fn update_comment(&self, repo: &str, id: &str, body: &str) -> Fallible<()>;
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> Fallible<bool>;
    // ...
}
```

#### 支持的平台

- **GitHub**：HMAC-SHA256 webhook 验证
- **Gitee**：Token-based webhook 验证
- **GitLab**：Token-based webhook 验证
- **GitCode**：Token-based webhook 验证（基于 GitLab API）

## 4. 服务端与 Bot 交互

### 4.1 交互模式

crater-ohos 服务端与 Bot 之间采用**双向交互**模式：

1. **Bot 主动调用**：Bot 通过 REST API 创建和管理实验
2. **服务端回调**：服务端通过 Webhook 主动通知 Bot 状态变化

### 4.2 API 调用流程

```
┌─────┐                    ┌──────────────┐                ┌─────────────┐
│ Bot │                    │ crater-ohos  │                │  Database   │
└──┬──┘                    └──────┬───────┘                └──────┬──────┘
   │                              │                               │
   │ 1. POST /api/v1/experiments  │                               │
   │ (Bearer Token)               │                               │
   ├─────────────────────────────>│                               │
   │                              │                               │
   │                              │ 2. 验证 Token                  │
   │                              │ (check Permission)            │
   │                              │                               │
   │                              │ 3. 创建实验记录                 │
   │                              ├──────────────────────────────>│
   │                              │                               │
   │                              │ 4. 返回实验 ID                 │
   │                              │<──────────────────────────────┤
   │                              │                               │
   │ 5. 返回实验详情               │                               │
   │<─────────────────────────────┤                               │
   │                              │                               │
   │ 6. POST /api/v1/experiments/ │                               │
   │    {name}/run                │                               │
   ├─────────────────────────────>│                               │
   │                              │                               │
   │                              │ 7. 更新状态为 running          │
   │                              ├──────────────────────────────>│
   │                              │                               │
   │                              │ 8. 启动执行任务                 │
   │                              │ (async)                       │
   │                              │                               │
   │ 9. 返回成功                   │                               │
   │<─────────────────────────────┤                               │
   │                              │                               │
```

### 4.3 Webhook Callback 流程

```
┌─────────────┐              ┌─────┐
│ crater-ohos │              │ Bot │
└──────┬──────┘              └──┬──┘
       │                        │
       │ 1. 实验执行完成          │
       │ (status: completed)    │
       │                        │
       │ 2. POST {callback_url} │
       │ {                      │
       │   "experiment": "...", │
       │   "status": "...",     │
       │   "report_url": "..."  │
       │ }                      │
       ├───────────────────────>│
       │                        │
       │                        │ 3. Bot 处理回调
       │                        │    - 获取报告
       │                        │    - 发布到 Issue
       │                        │
       │ 4. 返回 200 OK          │
       │<───────────────────────┤
       │                        │
       │ (如果失败，自动重试3次)  │
       │                        │
```

### 4.4 认证授权机制

#### Token 创建

Bot 需要先创建 API Token：

```rust
use crater_ohos::db::Database;
use crater_ohos::server::tokens::{Permission, TokenManager};

let db = Database::open()?;
let token = db.create_token(
    "gitee-bot-token",
    vec![
        Permission::ReadExperiments,
        Permission::WriteExperiments,
    ]
)?;
```

#### Token 使用

在 HTTP 请求中使用 Bearer Token：

```bash
curl -H "Authorization: Bearer <token>" \
     http://localhost:3000/api/v1/experiments
```

#### 权限验证

服务端自动验证 Token 并检查权限：

```rust
// 在中间件中自动处理
if !token.has_permission(&required_permission) {
    return Err(Error::Unauthorized);
}
```

## 5. Bot 开发指南

### 5.1 Bot 架构设计

一个典型的 Bot 应该包含以下组件：

```
┌─────────────────────────────────────────┐
│              Bot 应用                    │
├─────────────────────────────────────────┤
│  Webhook Listener                       │
│  - 监听平台 webhook                      │
│  - 解析用户命令                          │
├─────────────────────────────────────────┤
│  Command Parser                         │
│  - 提取实验参数                          │
│  - 验证命令格式                          │
├─────────────────────────────────────────┤
│  API Client                             │
│  - 调用 crater-ohos API                 │
│  - 处理认证和错误                        │
├─────────────────────────────────────────┤
│  Callback Handler                       │
│  - 接收 crater-ohos 回调                │
│  - 发布结果到平台                        │
├─────────────────────────────────────────┤
│  Platform Client                        │
│  - 发布评论                             │
│  - 更新 Issue                           │
└─────────────────────────────────────────┘
```

### 5.2 开发步骤

#### 步骤 1: 监听平台事件

以 Gitee 为例：

```rust
// 监听 Gitee webhook
#[post("/webhook/gitee")]
async fn gitee_webhook(
    body: String,
    signature: HeaderMap,
) -> Result<HttpResponse> {
    // 验证 webhook 签名
    verify_gitee_signature(&body, &signature)?;
    
    // 解析 webhook payload
    let event: GiteeEvent = serde_json::from_str(&body)?;
    
    // 处理评论事件
    if let GiteeEvent::NoteHook(note) = event {
        handle_comment(note).await?;
    }
    
    Ok(HttpResponse::Ok().finish())
}
```

#### 步骤 2: 解析用户命令

```rust
async fn handle_comment(note: NoteEvent) {
    let comment = note.comment.body;
    
    // 检查是否是 bot 命令
    if !comment.starts_with("@crater-bot") {
        return;
    }
    
    // 解析命令
    // 例如: "@crater-bot run stable beta"
    let parts: Vec<&str> = comment.split_whitespace().collect();
    
    if parts.len() < 3 {
        post_error_comment("命令格式错误").await;
        return;
    }
    
    let command = parts[1]; // "run"
    let toolchains: Vec<String> = parts[2..].iter()
        .map(|s| s.to_string())
        .collect();
    
    if command == "run" {
        create_experiment(toolchains).await;
    }
}
```

#### 步骤 3: 调用 crater-ohos API

```rust
async fn create_experiment(toolchains: Vec<String>) {
    let client = reqwest::Client::new();
    
    // 创建实验请求
    let request = CreateExperimentRequest {
        name: format!("gitee-issue-{}", issue_number),
        toolchains,
        mode: "build-and-test".to_string(),
        crate_select: "demo".to_string(),
        priority: 0,
        callback_url: Some("https://bot.example.com/webhook/crater".to_string()),
    };
    
    // 调用 API
    let response = client
        .post("http://crater-ohos:3000/api/v1/experiments")
        .header("Authorization", format!("Bearer {}", API_TOKEN))
        .json(&request)
        .send()
        .await?;
    
    if response.status().is_success() {
        // 发布成功评论
        post_comment("✅ 实验已创建，等待执行...").await;
        
        // 启动实验
        run_experiment(&request.name).await;
    } else {
        post_error_comment("创建实验失败").await;
    }
}
```

#### 步骤 4: 接收 Webhook 回调

```rust
#[post("/webhook/crater")]
async fn crater_callback(
    payload: Json<CallbackPayload>,
) -> Result<HttpResponse> {
    match payload.status.as_str() {
        "completed" => {
            // 获取报告
            let report_url = payload.report_url.as_ref()
                .ok_or("缺少报告 URL")?;
            
            // 发布结果到 Issue
            let message = format!(
                "✅ 实验已完成！\n\n查看报告: {}",
                report_url
            );
            post_comment(&message).await?;
        }
        "failed" => {
            post_comment("❌ 实验执行失败").await?;
        }
        _ => {}
    }
    
    Ok(HttpResponse::Ok().finish())
}
```

#### 步骤 5: 发布结果到平台

```rust
async fn post_comment(body: &str) {
    let client = reqwest::Client::new();
    
    // Gitee API
    client
        .post(format!(
            "https://gitee.com/api/v5/repos/{}/issues/{}/comments",
            repo, issue_number
        ))
        .header("Authorization", format!("Bearer {}", GITEE_TOKEN))
        .json(&serde_json::json!({
            "body": body
        }))
        .send()
        .await?;
}
```

### 5.3 完整交互示例

假设用户在 Gitee Issue #123 中评论：

```
@crater-bot run stable beta
```

完整流程：

1. **Gitee** 发送 webhook 到 Bot
2. **Bot** 接收 webhook，解析命令
3. **Bot** 调用 crater-ohos API：
   ```
   POST /api/v1/experiments
   {
     "name": "gitee-issue-123",
     "toolchains": ["stable", "beta"],
     "callback_url": "https://bot.example.com/webhook/crater"
   }
   ```
4. **crater-ohos** 创建实验，返回实验详情
5. **Bot** 在 Issue 中评论："✅ 实验已创建，等待执行..."
6. **Bot** 调用 crater-ohos API：
   ```
   POST /api/v1/experiments/gitee-issue-123/run
   ```
7. **crater-ohos** 开始执行实验
8. 实验完成后，**crater-ohos** 发送 webhook 到 Bot：
   ```
   POST https://bot.example.com/webhook/crater
   {
     "experiment": "gitee-issue-123",
     "status": "completed",
     "report_url": "https://crater.example.com/reports/gitee-issue-123"
   }
   ```
9. **Bot** 接收回调，在 Issue 中发布结果：
   ```
   ✅ 实验已完成！
   
   查看报告: https://crater.example.com/reports/gitee-issue-123
   ```

### 5.4 Bot 开发最佳实践

1. **错误处理**
   - 优雅处理 API 调用失败
   - 向用户反馈错误信息
   - 记录详细日志便于调试

2. **安全性**
   - 验证 webhook 签名
   - 安全存储 API Token
   - 限制 Bot 权限（最小权限原则）

3. **可靠性**
   - 实现重试机制
   - 处理并发请求
   - 超时控制

4. **用户体验**
   - 及时反馈命令执行状态
   - 提供清晰的错误消息
   - 支持取消/中止实验

5. **可维护性**
   - 模块化设计
   - 单元测试和集成测试
   - 详细的文档

### 5.5 Bot 参考实现

crater-ohos 项目计划提供以下 Bot 参考实现（独立仓库）：

- **Gitee Bot**：Gitee 平台 Bot 参考实现
- **GitHub Bot**：GitHub 平台 Bot 参考实现
- **GitLab Bot**：GitLab 平台 Bot 参考实现

每个参考实现将包含：

- 完整的源代码
- 部署指南
- 配置说明
- 开发文档

## 6. 扩展指南

### 6.1 添加新平台支持

如需支持新的代码托管平台，请按以下步骤操作：

#### 步骤 1: 定义平台类型

在 `src/platforms/mod.rs` 中添加新平台：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformType {
    GitHub,
    Gitee,
    GitLab,
    GitCode,
    NewPlatform,  // 新平台
}
```

#### 步骤 2: 实现平台适配器

创建 `src/platforms/newplatform.rs`：

```rust
use super::*;

pub struct NewPlatformAdapter {
    config: PlatformConfig,
}

impl NewPlatformAdapter {
    pub fn new(config: PlatformConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PlatformAdapter for NewPlatformAdapter {
    fn platform_type(&self) -> PlatformType {
        PlatformType::NewPlatform
    }
    
    async fn get_issue(&self, repo: &str, number: &str) 
        -> Fallible<PlatformIssue> {
        // 实现获取 Issue 逻辑
    }
    
    async fn post_comment(&self, repo: &str, number: &str, body: &str) 
        -> Fallible<()> {
        // 实现发布评论逻辑
    }
    
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) 
        -> Fallible<bool> {
        // 实现 webhook 签名验证
    }
    
    // 实现其他必需方法...
}
```

#### 步骤 3: 注册到工厂

在 `PlatformFactory::create()` 中添加：

```rust
PlatformType::NewPlatform => {
    Box::new(NewPlatformAdapter::new(config))
}
```

#### 步骤 4: 添加配置

在 `config.toml` 中添加平台配置：

```toml
[platforms.newplatform]
api_base_url = "https://newplatform.com/api"
token = "your-token"
webhook_secret = "your-secret"
```

#### 步骤 5: 编写测试

```rust
#[tokio::test]
async fn test_newplatform_adapter() {
    let config = PlatformConfig {
        api_base_url: "https://newplatform.com/api".to_string(),
        token: "test-token".to_string(),
        webhook_secret: Some("secret".to_string()),
    };
    
    let adapter = NewPlatformAdapter::new(config);
    assert_eq!(adapter.platform_type(), PlatformType::NewPlatform);
}
```

### 6.2 自定义实验模式

如需添加新的实验模式，请修改 `src/experiments.rs`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mode {
    BuildAndTest,
    BuildOnly,
    CheckOnly,
    Clippy,
    Rustdoc,
    CustomMode,  // 新模式
}
```

然后在执行层实现相应的执行逻辑。

### 6.3 自定义报告格式

如需添加新的报告格式，请在 `src/report/` 下创建新模块：

```rust
// src/report/json.rs
pub fn generate_json_report(results: &[TestResult]) -> String {
    serde_json::to_string_pretty(results).unwrap()
}
```

## 7. 部署指南

### 7.1 服务端部署

#### 使用 systemd

创建 `/etc/systemd/system/crater-ohos.service`：

```ini
[Unit]
Description=Crater-OHOS Server
After=network.target

[Service]
Type=simple
User=crater
WorkingDirectory=/opt/crater-ohos
ExecStart=/opt/crater-ohos/crater-ohos server --port 3000 --config /etc/crater-ohos/config.toml
Restart=always

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl enable crater-ohos
sudo systemctl start crater-ohos
```

#### 使用 Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/crater-ohos /usr/local/bin/
COPY config.toml /etc/crater-ohos/config.toml
CMD ["crater-ohos", "server", "--port", "3000", "--config", "/etc/crater-ohos/config.toml"]
```

### 7.2 Bot 部署

Bot 作为独立服务部署，可以：

- 与服务端部署在同一台服务器
- 部署在不同服务器
- 使用无服务器架构（如 AWS Lambda）

## 8. 性能优化

### 8.1 数据库优化

- 使用索引加速查询
- 定期清理旧数据
- 使用连接池

### 8.2 API 优化

- 实现缓存机制
- 支持分页查询
- 使用 gzip 压缩响应

### 8.3 执行优化

- 并行执行多个实验
- 优化 Docker 镜像
- 资源限制和监控

## 9. 安全考虑

### 9.1 认证授权

- 使用强随机 Token
- 定期轮换 Token
- 细粒度权限控制

### 9.2 输入验证

- 验证所有用户输入
- 防止 SQL 注入
- 防止路径遍历

### 9.3 Webhook 安全

- 验证 webhook 签名
- 使用 HTTPS
- 限制请求频率

## 10. 监控和日志

### 10.1 日志

crater-ohos 使用结构化日志：

```rust
log::info!("Experiment created: {}", experiment.name);
log::error!("Failed to run experiment: {}", err);
```

### 10.2 监控指标

建议监控以下指标：

- 实验创建/完成速率
- API 请求成功/失败率
- Agent 在线/离线数量
- 数据库查询延迟
- 磁盘空间使用

### 10.3 告警

设置告警规则：

- 实验失败率过高
- API 错误率过高
- Agent 长时间离线
- 磁盘空间不足

## 11. 故障排查

### 11.1 常见问题

**问题：API 返回 401 Unauthorized**

解决：检查 Token 是否正确，是否有相应权限

**问题：Webhook 回调失败**

解决：检查 callback URL 是否可访问，Bot 是否正常运行

**问题：实验执行失败**

解决：检查工具链是否安装，crate 是否可下载

### 11.2 调试技巧

1. 启用详细日志：`RUST_LOG=debug crater-ohos server`
2. 检查数据库：`sqlite3 crater.db "SELECT * FROM experiments;"`
3. 测试 API：使用 curl 或 Postman 测试端点
4. 查看 Agent 状态：`curl http://localhost:3000/api/v1/agents`

## 12. 贡献指南

欢迎为 crater-ohos 贡献代码！请遵循以下步骤：

1. Fork 仓库
2. 创建特性分支：`git checkout -b feature/my-feature`
3. 编写代码并测试：`cargo test`
4. 运行代码检查：`cargo clippy`
5. 格式化代码：`cargo fmt`
6. 提交更改：`git commit -am 'Add my feature'`
7. 推送分支：`git push origin feature/my-feature`
8. 创建 Pull Request

### 代码规范

- 遵循 Rust 官方风格指南
- 编写单元测试和集成测试
- 添加文档注释（rustdoc）
- 保持向后兼容性

## 13. 参考资料

- [rust-lang/crater](https://github.com/rust-lang/crater) - 上游项目
- [Rust 官方文档](https://doc.rust-lang.org/)
- [Axum Web 框架](https://github.com/tokio-rs/axum)
- [SQLite 文档](https://www.sqlite.org/docs.html)

## 14. 许可证

MIT OR Apache-2.0

---

**文档版本**：1.0.0  
**最后更新**：2024-12  
**维护者**：crater-ohos 团队
