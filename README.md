# Crater-OHOS

用于鸿蒙环境的 Rust 三方库验证工具，基于 [rust-lang/crater](https://github.com/rust-lang/crater) 重构。

## 架构设计

Crater-OHOS 采用 **Core + Bot 解耦架构**：

```
┌─────────────────────────────────────────────────────────────┐
│                      API Layer (Phase 6)                     │
│            REST API / CLI 接口，供 Bot 或用户调用              │
├─────────────────────────────────────────────────────────────┤
│                   Service Layer (Phase 4)                    │
│     actions/  - 业务操作（创建/编辑/删除实验）                  │
│     server/   - 服务端逻辑（agent管理、callback通知）           │
├─────────────────────────────────────────────────────────────┤
│                   Domain Layer (Phase 2)                     │
│     experiments.rs - 实验领域模型                              │
│     results/       - 结果领域模型                              │
│     crates/        - Crate 领域模型                           │
│     toolchain.rs   - 工具链领域模型                            │
├─────────────────────────────────────────────────────────────┤
│                  Execution Layer (Phase 3)                   │
│     runner/        - 构建/测试执行引擎                         │
│     report/        - 报告生成引擎                              │
├─────────────────────────────────────────────────────────────┤
│                Infrastructure Layer (Phase 1)                │
│     db/            - 数据库访问                                │
│     config.rs      - 配置管理                                  │
│     utils/         - 通用工具                                  │
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

### 使用 CLI

```bash
# 准备环境
cargo run -- prepare-local

# 定义实验
cargo run -- define-ex --ex my-experiment stable beta --crate-select demo

# 运行实验
cargo run -- run-graph --ex my-experiment -t 4

# 生成报告
cargo run -- gen-report --ex my-experiment ./report
```

## 配置

创建 `config.toml` 文件：

```toml
[demo-crates]
crates = ["lazy_static", "serde"]
github-repos = []
local-crates = []

[sandbox]
memory-limit = "2G"
build-log-max-size = "2M"
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

### 🚧 Phase 5: API Layer（API 层）- 计划中

- [ ] REST API
- [ ] CLI 命令
- [ ] 认证和授权

### 🚧 Phase 6: Bot Integration（Bot 集成）- 计划中

- [ ] Gitee Bot
- [ ] GitHub Bot
- [ ] GitLab Bot

## 测试覆盖

项目包含全面的测试覆盖：

- **单元测试**：90+ 测试用例
  - 数据库操作测试
  - 领域模型测试
  - 工具函数测试
  - 序列化/反序列化测试
  
- **集成测试**：7 个集成测试
  - 数据库迁移测试
  - 配置加载测试
  - 实验工作流测试
  - 表结构验证测试

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
- `shas`：Git 提交 SHA
- `saved_names`：工具链名称映射
- `migrations`：数据库迁移记录

## API 设计（Phase 6 计划）

### 创建实验

```http
POST /api/v1/experiments
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "test-experiment",
  "toolchains": ["stable", "beta"],
  "mode": "build-and-test",
  "crate-select": "demo",
  "platform-issue": {
    "platform": "gitcode",
    "api_url": "https://api.gitcode.com/issues/1",
    "html_url": "https://gitcode.com/issues/1",
    "identifier": "1"
  },
  "callback-url": "https://bot.example.com/callback"
}
```

### 查询实验状态

```http
GET /api/v1/experiments/{name}
Authorization: Bearer <token>
```

### Webhook 回调

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
