# Crater-OHOS

用于鸿蒙环境的 Rust 三方库验证工具，基于 [rust-lang/crater](https://github.com/rust-lang/crater) 重构。

## 架构设计

Crater-OHOS 采用 **Core + Bot 解耦架构**：

```
┌─────────────────────────────────────────────────────────────┐
│                      API Layer (Phase 6)                     │
│            REST API / CLI 接口，供 Bot 或用户调用              │
├─────────────────────────────────────────────────────────────┤
│                   Service Layer (Phase 4) ✅                 │
│     actions/  - 业务操作（创建/编辑/删除实验）                  │
│     server/   - 服务端逻辑（agent管理、callback通知）           │
├─────────────────────────────────────────────────────────────┤
│                   Domain Layer (Phase 2) ✅                  │
│     experiments.rs - 实验领域模型                              │
│     results/       - 结果领域模型                              │
│     crates/        - Crate 领域模型                           │
│     toolchain.rs   - 工具链领域模型                            │
├─────────────────────────────────────────────────────────────┤
│                  Execution Layer (Phase 3) ✅                │
│     runner/        - 构建/测试执行引擎                         │
│     report/        - 报告生成引擎                              │
├─────────────────────────────────────────────────────────────┤
│                Infrastructure Layer (Phase 1) ✅             │
│     db/            - 数据库访问                                │
│     config.rs      - 配置管理                                  │
│     utils/         - 通用工具                                  │
├─────────────────────────────────────────────────────────────┤
│            Platform Abstraction (Phase 5) ✅                 │
│     platforms/     - 平台适配器（GitHub, Gitee, GitLab）       │
└─────────────────────────────────────────────────────────────┘
```

## 与上游 Crater 的主要区别

| 特性 | rust-lang/crater | crater-ohos |
|------|------------------|-------------|
| 平台依赖 | GitHub 特定 | 平台无关 |
| Issue 结构 | `GitHubIssue` | `PlatformIssue` |
| Bot 集成 | 内置 GitHub bot | 外部 bot 通过 API 调用 |
| Callback | 无 | 支持 webhook 回调 |

## 快速开始

### 构建

```bash
cargo build --release
```

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
# 运行 Clippy 检查
cargo clippy

# 生成文档
cargo doc
```

## 快速使用

### CLI 命令

crater-ohos 提供了完整的命令行界面：

```bash
# 查看帮助
crater-ohos --help

# 准备本地环境
crater-ohos prepare-local

# 定义实验
crater-ohos define-ex --ex my-experiment stable beta --crate-select demo

# 运行实验
crater-ohos run-graph --ex my-experiment -t 4

# 列出所有实验
crater-ohos list-ex

# 生成报告
crater-ohos gen-report --ex my-experiment ./report

# 删除实验
crater-ohos delete-ex --ex my-experiment

# 中止实验
crater-ohos abort-ex --ex my-experiment

# 启动 API 服务器
crater-ohos server --port 3000 --config config.toml
```

### REST API

启动 API 服务器后，可以使用以下端点：

#### 健康检查（无需认证）

```bash
# 健康检查
curl http://localhost:3000/api/v1/health

# 响应示例
{
  "success": true,
  "data": {
    "status": "ok",
    "version": "0.1.0"
  }
}
```

#### 实验管理（需要认证）

```bash
# 创建实验
curl -X POST http://localhost:3000/api/v1/experiments \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-experiment",
    "toolchains": ["stable", "beta"],
    "mode": "build-and-test",
    "crate_select": "demo",
    "priority": 0
  }'

# 列出所有实验
curl http://localhost:3000/api/v1/experiments \
  -H "Authorization: Bearer <token>"

# 获取实验详情
curl http://localhost:3000/api/v1/experiments/test-experiment \
  -H "Authorization: Bearer <token>"

# 运行实验
curl -X POST http://localhost:3000/api/v1/experiments/test-experiment/run \
  -H "Authorization: Bearer <token>"

# 中止实验
curl -X POST http://localhost:3000/api/v1/experiments/test-experiment/abort \
  -H "Authorization: Bearer <token>"

# 删除实验
curl -X DELETE http://localhost:3000/api/v1/experiments/test-experiment \
  -H "Authorization: Bearer <token>"
```

#### Agent 管理（需要认证）

```bash
# 注册 Agent
curl -X POST http://localhost:3000/api/v1/agents/register \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-agent",
    "capabilities": ["build", "test"]
  }'

# Agent 心跳
curl -X POST http://localhost:3000/api/v1/agents/{agent-id}/heartbeat \
  -H "Authorization: Bearer <token>"

# 列出所有 Agent
curl http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer <token>"
```

#### 认证

API 使用 Bearer Token 认证。Token 需要通过数据库中的 `api_tokens` 表管理，或使用 `TokenManager` trait 创建：

```rust
use crater_ohos::db::Database;
use crater_ohos::server::tokens::{Permission, TokenManager};

let db = Database::open()?;
let token = db.create_token(
    "my-token",
    vec![Permission::ReadExperiments, Permission::WriteExperiments]
)?;
println!("Token: {}", token.token);
```

## 配置

创建 `config.toml` 文件：

```toml
[demo-crates]
crates = ["lazy_static", "serde"]
github-repos = []
local-crates = []

[sandbox]
memory-limit = { "GIGABYTES" = 2 }
build-log-max-size = { "MEGABYTES" = 2 }
build-log-max-lines = 1000

[server.acl]
allowed-users = []

[server.callback]
timeout-secs = 30
retry-count = 3
```

## 当前进度

### ✅ Phase 1: Infrastructure Layer（基础设施层）

已完成以下模块：

#### 1. 项目结构
- ✅ `Cargo.toml`：包含所有必要依赖
- ✅ 模块化的源代码结构

#### 2. 数据库模块 (`src/db/`)
- ✅ SQLite 连接池管理
- ✅ 事务处理
- ✅ `QueryUtils` trait 提供便捷的数据库操作
- ✅ 数据库迁移系统
- ✅ `experiment_metadata` 表用于存储实验元数据

#### 3. 配置模块 (`src/config.rs`)
- ✅ 配置文件解析（TOML 格式）
- ✅ ACL（访问控制列表）配置
- ✅ Callback 配置（超时、重试）
- ✅ 沙箱配置（内存限制、日志大小）
- ✅ 平台无关设计

#### 4. 工具模块 (`src/utils/`)
- ✅ `size.rs`：内存大小处理
- ✅ `http.rs`：HTTP 客户端封装
- ✅ `hex.rs`：十六进制编码/解码

### ✅ Phase 2: Domain Layer（领域层）

已完成以下模块：

#### 1. 实验模块 (`src/experiments.rs`)
- ✅ `Experiment` 结构体：实验配置和状态
- ✅ `Status` 枚举：实验状态（queued, running, completed 等）
- ✅ `Mode` 枚举：实验模式（build-and-test, build-only 等）
- ✅ `PlatformIssue`：平台无关的 Issue 抽象
- ✅ `CrateSelect`：Crate 选择策略

#### 2. Crate 模块 (`src/crates/`)
- ✅ `Crate` 枚举：支持多种 crate 来源
  - Registry（crates.io）
  - GitHub
  - Local
  - Path
  - Git
- ✅ Crate 列表管理
- ✅ Crate 源抽象

#### 3. 工具链模块 (`src/toolchain.rs`)
- ✅ `Toolchain` 结构体
- ✅ `RustwideToolchain` 支持多种工具链类型
- ✅ 工具链解析和序列化

#### 4. 结果模块 (`src/results/`)
- ✅ `TestResult` 枚举：测试结果类型
- ✅ `FailureReason` 枚举：失败原因
- ✅ `EncodedLog`：日志编码（Plain/Gzip）
- ✅ 结果数据库存储

### ✅ Phase 3: Execution Layer（执行层）

已完成以下模块：

#### 1. 运行器模块 (`src/runner/`)
- ✅ `tasks.rs`：任务定义和管理
- ✅ `test.rs`：测试执行逻辑
- ✅ `worker.rs`：工作线程和资源监控
- ✅ 磁盘空间监控

#### 2. 报告模块 (`src/report/`)
- ✅ `analyzer.rs`：结果分析
- ✅ `html.rs`：HTML 报告生成
- ✅ `markdown.rs`：Markdown 报告生成
- ✅ `display.rs`：显示工具
- ✅ `archives.rs`：归档处理

### 🚧 Phase 4: Service Layer（服务层）- ✅ 已完成

已完成以下模块：

#### 1. Actions 模块 (`src/actions/`)
- ✅ `experiments.rs`：实验生命周期管理
  - `CreateExperiment`：创建实验请求
  - `EditExperiment`：编辑实验请求  
  - `ExperimentActions` trait：实验操作接口
    - `create()` - 创建新实验
    - `edit()` - 编辑实验（仅限 queued 状态）
    - `delete()` - 删除实验（仅限 queued 状态）
    - `get()` - 获取实验详情
    - `list()` - 列出所有实验
    - `run()` - 运行实验
    - `complete()` - 完成实验
    - `abort()` - 中止实验

#### 2. Server 模块 (`src/server/`)
- ✅ `agents.rs`：Agent 管理
  - `Agent` 结构体和 `AgentStatus` 枚举
  - `RegisterAgent` 请求结构
  - `AgentManager` trait：Agent 管理接口
    - 注册、心跳、任务分配、状态管理
- ✅ `callback.rs`：Callback 通知
  - `CallbackEvent` 枚举：事件类型
  - `CallbackPayload` 结构：回调数据
  - `CallbackService`：HTTP 回调服务（带重试）
- ✅ `tokens.rs`：API Token 管理
  - `ApiToken` 结构和 `Permission` 枚举
  - `TokenManager` trait：Token 管理接口

#### 3. 数据库支持
- ✅ `agents` 表：Agent 信息存储
- ✅ `api_tokens` 表：API Token 存储
- ✅ 数据库迁移：自动创建新表

### ✅ Phase 5: Platform Abstraction（平台抽象层）- 已完成

已完成以下模块：

#### 1. 平台模块 (`src/platforms/`)
- ✅ `mod.rs`：平台抽象 trait 和工厂模式
  - `PlatformType` 枚举：支持 GitHub、Gitee、GitLab、GitCode
  - `PlatformAdapter` trait：统一的平台操作接口
  - `PlatformFactory`：平台适配器工厂
  - `PlatformConfig`：平台配置结构
- ✅ `github.rs`：GitHub 适配器实现
  - 支持 Issue 获取、评论发表、Webhook 签名验证
- ✅ `gitee.rs`：Gitee 适配器实现
  - 适配 Gitee API v5
- ✅ `gitlab.rs`：GitLab 适配器实现
  - 支持 GitLab 和 GitCode（基于 GitLab）

#### 2. 配置支持
- ✅ 多平台配置 (`PlatformsConfig`)
  - GitHub、Gitee、GitLab 独立配置
  - API 基础 URL、Token、Webhook Secret

#### 3. 测试覆盖
- ✅ 平台类型序列化测试
- ✅ 平台工厂测试
- ✅ GitHub Issue URL 生成测试
- ✅ Gitee Issue URL 生成测试
- ✅ GitLab Issue URL 生成测试
- ✅ Webhook 签名验证测试

### ✅ Phase 6: API Layer（API 层）- 已完成

已完成以下模块：

#### 1. REST API (`src/api/`)
- ✅ `mod.rs`：API 路由构建器
- ✅ `error.rs`：统一错误处理
- ✅ `response.rs`：统一响应格式
- ✅ `middleware/auth.rs`：Bearer Token 认证中间件
- ✅ `routes/experiments.rs`：实验管理端点
  - `POST /api/v1/experiments` - 创建实验
  - `GET /api/v1/experiments` - 列出所有实验
  - `GET /api/v1/experiments/{name}` - 获取实验详情
  - `PUT /api/v1/experiments/{name}` - 编辑实验
  - `DELETE /api/v1/experiments/{name}` - 删除实验
  - `POST /api/v1/experiments/{name}/run` - 运行实验
  - `POST /api/v1/experiments/{name}/abort` - 中止实验
- ✅ `routes/agents.rs`：Agent 管理端点
  - `POST /api/v1/agents/register` - 注册 Agent
  - `POST /api/v1/agents/{id}/heartbeat` - Agent 心跳
  - `GET /api/v1/agents` - 列出所有 Agent
  - `GET /api/v1/agents/{id}` - 获取 Agent 详情
- ✅ `routes/health.rs`：健康检查端点
  - `GET /api/v1/health` - 健康检查
  - `GET /api/v1/config` - 获取配置信息

#### 2. CLI 命令 (`src/cli/`)
- ✅ `args.rs`：命令行参数定义（基于 clap）
- ✅ `commands/prepare.rs`：`prepare-local` - 准备本地环境
- ✅ `commands/define.rs`：`define-ex` - 定义实验
- ✅ `commands/run.rs`：`run-graph` - 运行实验
- ✅ `commands/report.rs`：`gen-report` - 生成报告
- ✅ `commands/server.rs`：`server` - 启动 API 服务器
- ✅ `commands/manage.rs`：实验管理命令
  - `list-ex` - 列出所有实验
  - `delete-ex` - 删除实验
  - `abort-ex` - 中止实验

#### 3. 认证和授权
- ✅ Bearer Token 认证中间件
- ✅ 基于 Permission 的权限控制
- ✅ 支持 Admin 权限

#### 4. 测试覆盖
- ✅ API 模块集成测试（5 个测试）
- ✅ 所有 Phase 1-6 测试通过（121 个测试）

### 🚧 Phase 7: Bot Integration（Bot 集成）- 计划中

- [ ] Gitee Bot
- [ ] GitHub Bot
- [ ] GitLab Bot

## 支持的平台

- **GitHub**：通过 GitHub API 支持
- **Gitee**：通过 Gitee API v5 支持
- **GitLab**：通过 GitLab API 支持
- **GitCode**：基于 GitLab 适配器支持

## 添加新平台支持

如需添加新平台支持，请遵循以下步骤：

1. 在 `src/platforms/` 下创建新的适配器文件（如 `custom.rs`）
2. 实现 `PlatformAdapter` trait，提供以下功能：
   - `platform_type()` - 返回平台类型
   - `check_permission()` - 权限检查
   - `get_issue()` - 获取 Issue 信息
   - `post_comment()` - 发表评论
   - `update_comment()` - 更新评论
   - `get_repo()` - 获取仓库信息
   - `get_user()` - 获取用户信息
   - `verify_webhook_signature()` - Webhook 签名验证
3. 在 `PlatformType` 枚举中添加新平台
4. 在 `PlatformFactory::create()` 中注册新适配器
5. 在 `config.rs` 的 `PlatformsConfig` 中添加平台配置
6. 编写相应的测试用例

示例：

```rust
// src/platforms/custom.rs
use super::*;

pub struct CustomAdapter {
    config: PlatformConfig,
}

impl CustomAdapter {
    pub fn new(config: PlatformConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PlatformAdapter for CustomAdapter {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Custom("my-platform".to_string())
    }
    
    async fn get_issue(&self, repo: &str, number: &str) -> Fallible<PlatformIssue> {
        // 实现自定义平台的 Issue 获取逻辑
        Ok(PlatformIssue {
            platform: "my-platform".to_string(),
            api_url: format!("https://my-platform.com/api/repos/{}/issues/{}", repo, number),
            html_url: format!("https://my-platform.com/{}/issues/{}", repo, number),
            identifier: number.to_string(),
        })
    }
    
    // 实现其他必需的方法...
}
```

## 测试覆盖

项目包含全面的测试覆盖：

- **单元测试**：103 测试用例
  - 数据库操作测试
  - 领域模型测试
  - 平台适配器测试
  - 工具函数测试
  - 序列化/反序列化测试
  - Token 管理测试
  - Agent 管理测试
  
- **集成测试**：18 个集成测试
  - 数据库迁移测试
  - 配置加载测试
  - 实验工作流测试
  - 表结构验证测试
  - API 模块测试
  - Service 层集成测试

运行测试：

```bash
# 运行所有测试
cargo test

# 运行指定测试
cargo test test_database_migrations

# 查看测试输出
cargo test -- --nocapture
```

## 数据库架构

项目使用 SQLite 作为数据存储，主要表结构：

- `experiments`：实验配置和状态
- `experiment_metadata`：实验元数据（callback URL、平台等）
- `results`：构建和测试结果
- `experiment_crates`：实验包含的 crate 列表
- `agents`：Agent 信息和状态（Phase 4）
- `api_tokens`：API Token 管理（Phase 4）
- `shas`：Git 提交 SHA
- `saved_names`：工具链名称映射
- `migrations`：数据库迁移记录

## Webhook 回调

当实验状态变化时，系统会向配置的 callback URL 发送通知：

```json
{
  "experiment": "test-experiment",
  "status": "completed",
  "report_url": "https://crater.example.com/reports/test-experiment"
}
```

## 贡献

欢迎贡献！请提交 Issue 或 Pull Request。

### 开发指南

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/my-feature`
3. 提交更改：`git commit -am 'Add my feature'`
4. 推送分支：`git push origin feature/my-feature`
5. 创建 Pull Request

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 确保所有测试通过：`cargo test`
- 添加适当的文档注释

## 许可证

MIT OR Apache-2.0

## 参考

- 上游项目：[rust-lang/crater](https://github.com/rust-lang/crater)
- 设计文档：[ARCHITECTURE.md](docs/ARCHITECTURE.md)（待添加）
