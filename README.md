# crater-ohos

一个用于鸿蒙（OHOS）环境的第三方库验证工具，基于 [rust-lang/crater](https://github.com/rust-lang/crater) 重构为平台无关的 Core + Bot 解耦架构。

## 项目简介

crater-ohos 是一个自动化的第三方库测试工具，用于验证 Rust 库在鸿蒙环境中的兼容性和稳定性。本项目采用了模块化的架构设计，将核心功能与特定平台的机器人（Bot）分离，使得系统更易于扩展和维护。

## 架构设计

本项目采用 **Core + Bot 解耦架构**：

### Core（核心层）
- **Infrastructure Layer（基础设施层）**：数据存储、配置管理、工具函数
- **Domain Layer（领域层）**：实验模型、构建配置、工具链管理
- **Execution Layer（执行层）**：任务调度、构建执行、结果收集
- **Service Layer（服务层）**：HTTP API、Webhook 回调

### Bot（机器人层）
- 平台特定的交互逻辑（如 Gitee、GitHub 等）
- 通过 HTTP API 与 Core 通信
- 可独立部署和扩展

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
- ✅ `experiment_metadata` 表用于存储实验元数据（callback URL、平台、触发者等）

#### 3. 配置模块 (`src/config.rs`)
- ✅ 配置文件解析（TOML 格式）
- ✅ ACL（访问控制列表）配置
- ✅ Callback 配置（超时、重试）
- ✅ 沙箱配置（内存限制、日志大小）
- ✅ 移除了 GitHub 特定配置，保持平台无关性

#### 4. 工具模块 (`src/utils/`)
- ✅ `size.rs`：内存大小处理（Bytes、KB、MB、GB）
- ✅ `http.rs`：HTTP 客户端封装（GET/POST）
- ✅ `hex.rs`：十六进制编码/解码

#### 5. 通用模块
- ✅ `src/lib.rs`：库入口
- ✅ `src/prelude.rs`：常用类型定义
- ✅ `src/dirs.rs`：工作目录管理
- ✅ `src/main.rs`：程序入口

## 构建和运行

### 环境要求

- Rust 1.70+
- SQLite 3.x（已通过 `rusqlite` bundled 特性内置）

### 构建项目

```bash
# 克隆仓库
git clone https://github.com/LuuuXXX/crater-ohos.git
cd crater-ohos

# 构建
cargo build

# 运行测试
cargo test

# 运行程序
cargo run
```

### 配置文件

项目使用 `config.toml` 进行配置。示例配置：

```toml
[demo-crates]
crates = ["lazy_static"]
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

## 数据库架构

项目使用 SQLite 作为数据存储，主要表结构：

- `experiments`：实验配置和状态
- `experiment_metadata`：实验元数据（callback URL、平台等）
- `results`：构建和测试结果
- `experiment_crates`：实验包含的 crate 列表
- `shas`：Git 提交 SHA
- `saved_names`：工具链名称映射

## 下一步计划

- [ ] Phase 2: Domain Layer - 实验模型、工具链管理
- [ ] Phase 3: Execution Layer - 任务调度、构建执行
- [ ] Phase 4: Service Layer - HTTP API、Webhook
- [ ] Phase 5: Bot Integration - Gitee/GitHub 机器人

## 贡献

欢迎贡献！请提交 Issue 或 Pull Request。

## 许可证

MIT OR Apache-2.0

## 参考

- 上游项目：[rust-lang/crater](https://github.com/rust-lang/crater)
