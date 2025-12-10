# Crater-OHOS API 使用指南

本文档详细说明 crater-ohos 提供的 REST API 接口。

## 目录

- [基础信息](#基础信息)
- [认证方式](#认证方式)
- [API 端点](#api-端点)
  - [健康检查](#健康检查)
  - [实验管理](#实验管理)
  - [Agent 管理](#agent-管理)
- [数据模型](#数据模型)
- [错误处理](#错误处理)
- [使用示例](#使用示例)

## 基础信息

### Base URL

```
http://localhost:3000
```

生产环境请替换为实际的服务器地址。

### API 版本

当前 API 版本：`v1`

所有端点前缀：`/api/v1`

### 内容类型

- 请求：`application/json`
- 响应：`application/json`

### 统一响应格式

**成功响应：**

```json
{
  "success": true,
  "data": {
    // 业务数据
  }
}
```

**错误响应：**

```json
{
  "success": false,
  "error": "错误描述信息"
}
```

## 认证方式

### Bearer Token 认证

除 `/health` 端点外，所有 API 端点都需要认证。

**请求头：**

```
Authorization: Bearer <token>
```

### 创建 Token

Token 需要通过代码或直接操作数据库创建：

```rust
use crater_ohos::db::Database;
use crater_ohos::server::tokens::{Permission, TokenManager};

let db = Database::open()?;
let token = db.create_token(
    "my-bot-token",
    vec![
        Permission::ReadExperiments,
        Permission::WriteExperiments,
    ]
)?;

println!("Token: {}", token.token);
```

### 权限说明

| 权限 | 说明 |
|------|------|
| `Admin` | 管理员权限，拥有所有权限 |
| `ReadExperiments` | 读取实验信息 |
| `WriteExperiments` | 创建、编辑、运行实验 |
| `DeleteExperiments` | 删除实验 |
| `ManageAgents` | 管理 Agent |

## API 端点

### 健康检查

#### GET /api/v1/health

获取服务健康状态。

**认证：** 不需要

**请求示例：**

```bash
curl http://localhost:3000/api/v1/health
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "status": "ok",
    "version": "0.1.0"
  }
}
```

#### GET /api/v1/config

获取服务配置信息。

**认证：** 需要（任意权限）

**请求示例：**

```bash
curl -H "Authorization: Bearer <token>" \
     http://localhost:3000/api/v1/config
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "sandbox": {
      "memory_limit": { "GIGABYTES": 2 },
      "build_log_max_size": { "MEGABYTES": 2 },
      "build_log_max_lines": 1000
    }
  }
}
```

### 实验管理

#### POST /api/v1/experiments

创建新实验。

**认证：** 需要 `WriteExperiments` 权限

**请求参数：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 实验名称，唯一标识 |
| `toolchains` | array[string] | 是 | 工具链列表，如 `["stable", "beta"]` |
| `mode` | string | 是 | 实验模式：`build-and-test`, `build-only`, `check-only`, `clippy`, `rustdoc` |
| `crate_select` | string | 是 | Crate 选择策略：`demo`, `full`, `local` 等 |
| `priority` | integer | 否 | 优先级，默认 0 |
| `callback_url` | string | 否 | Webhook 回调 URL |

**请求示例：**

```bash
curl -X POST http://localhost:3000/api/v1/experiments \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-experiment",
    "toolchains": ["stable", "beta"],
    "mode": "build-and-test",
    "crate_select": "demo",
    "priority": 0,
    "callback_url": "https://bot.example.com/webhook/crater"
  }'
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "name": "test-experiment",
    "toolchains": ["stable", "beta"],
    "mode": "BuildAndTest",
    "status": "Queued",
    "priority": 0,
    "created_at": "2024-12-10T12:00:00Z"
  }
}
```

#### GET /api/v1/experiments

列出所有实验。

**认证：** 需要 `ReadExperiments` 权限

**请求示例：**

```bash
curl -H "Authorization: Bearer <token>" \
     http://localhost:3000/api/v1/experiments
```

**响应示例：**

```json
{
  "success": true,
  "data": [
    {
      "name": "test-experiment",
      "status": "Queued",
      "created_at": "2024-12-10T12:00:00Z"
    },
    {
      "name": "another-experiment",
      "status": "Running",
      "created_at": "2024-12-10T11:00:00Z"
    }
  ]
}
```

#### GET /api/v1/experiments/{name}

获取指定实验的详细信息。

**认证：** 需要 `ReadExperiments` 权限

**路径参数：**

- `name` - 实验名称

**请求示例：**

```bash
curl -H "Authorization: Bearer <token>" \
     http://localhost:3000/api/v1/experiments/test-experiment
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "name": "test-experiment",
    "toolchains": ["stable", "beta"],
    "mode": "BuildAndTest",
    "status": "Running",
    "priority": 0,
    "created_at": "2024-12-10T12:00:00Z",
    "started_at": "2024-12-10T12:05:00Z"
  }
}
```

#### PUT /api/v1/experiments/{name}

编辑实验配置。

**认证：** 需要 `WriteExperiments` 权限

**限制：** 仅 `Queued` 状态的实验可以编辑

**路径参数：**

- `name` - 实验名称

**请求参数：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `toolchains` | array[string] | 否 | 新的工具链列表 |
| `mode` | string | 否 | 新的实验模式 |
| `priority` | integer | 否 | 新的优先级 |

**请求示例：**

```bash
curl -X PUT http://localhost:3000/api/v1/experiments/test-experiment \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "toolchains": ["stable", "nightly"],
    "priority": 5
  }'
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "name": "test-experiment",
    "toolchains": ["stable", "nightly"],
    "mode": "BuildAndTest",
    "status": "Queued",
    "priority": 5
  }
}
```

#### DELETE /api/v1/experiments/{name}

删除实验。

**认证：** 需要 `DeleteExperiments` 权限

**限制：** 仅 `Queued` 状态的实验可以删除

**路径参数：**

- `name` - 实验名称

**请求示例：**

```bash
curl -X DELETE http://localhost:3000/api/v1/experiments/test-experiment \
  -H "Authorization: Bearer <token>"
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "message": "Experiment deleted successfully"
  }
}
```

#### POST /api/v1/experiments/{name}/run

运行实验。

**认证：** 需要 `WriteExperiments` 权限

**路径参数：**

- `name` - 实验名称

**请求示例：**

```bash
curl -X POST http://localhost:3000/api/v1/experiments/test-experiment/run \
  -H "Authorization: Bearer <token>"
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "name": "test-experiment",
    "status": "Running",
    "started_at": "2024-12-10T12:05:00Z"
  }
}
```

#### POST /api/v1/experiments/{name}/abort

中止正在运行的实验。

**认证：** 需要 `WriteExperiments` 权限

**路径参数：**

- `name` - 实验名称

**请求示例：**

```bash
curl -X POST http://localhost:3000/api/v1/experiments/test-experiment/abort \
  -H "Authorization: Bearer <token>"
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "name": "test-experiment",
    "status": "Aborted",
    "aborted_at": "2024-12-10T12:10:00Z"
  }
}
```

### Agent 管理

#### POST /api/v1/agents/register

注册新 Agent。

**认证：** 需要 `ManageAgents` 权限

**请求参数：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | Agent 名称 |
| `capabilities` | array[string] | 是 | Agent 能力列表，如 `["build", "test"]` |

**请求示例：**

```bash
curl -X POST http://localhost:3000/api/v1/agents/register \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-agent",
    "capabilities": ["build", "test"]
  }'
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "my-agent",
    "status": "Idle",
    "registered_at": "2024-12-10T12:00:00Z"
  }
}
```

#### POST /api/v1/agents/{id}/heartbeat

Agent 心跳，更新最后活跃时间。

**认证：** 需要 `ManageAgents` 权限

**路径参数：**

- `id` - Agent ID（UUID）

**请求示例：**

```bash
curl -X POST http://localhost:3000/api/v1/agents/550e8400-e29b-41d4-a716-446655440000/heartbeat \
  -H "Authorization: Bearer <token>"
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "last_heartbeat": "2024-12-10T12:05:00Z"
  }
}
```

#### GET /api/v1/agents

列出所有 Agent。

**认证：** 需要 `ManageAgents` 权限

**请求示例：**

```bash
curl -H "Authorization: Bearer <token>" \
     http://localhost:3000/api/v1/agents
```

**响应示例：**

```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "my-agent",
      "status": "Idle",
      "last_heartbeat": "2024-12-10T12:05:00Z"
    }
  ]
}
```

#### GET /api/v1/agents/{id}

获取指定 Agent 的详细信息。

**认证：** 需要 `ManageAgents` 权限

**路径参数：**

- `id` - Agent ID（UUID）

**请求示例：**

```bash
curl -H "Authorization: Bearer <token>" \
     http://localhost:3000/api/v1/agents/550e8400-e29b-41d4-a716-446655440000
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "my-agent",
    "status": "Busy",
    "capabilities": ["build", "test"],
    "registered_at": "2024-12-10T12:00:00Z",
    "last_heartbeat": "2024-12-10T12:05:00Z",
    "current_task": "test-experiment"
  }
}
```

## 数据模型

### Experiment（实验）

```json
{
  "name": "string",
  "toolchains": ["string"],
  "mode": "BuildAndTest" | "BuildOnly" | "CheckOnly" | "Clippy" | "Rustdoc",
  "status": "Queued" | "Running" | "Completed" | "Aborted",
  "priority": 0,
  "crate_select": "string",
  "created_at": "2024-12-10T12:00:00Z",
  "started_at": "2024-12-10T12:05:00Z",
  "completed_at": "2024-12-10T13:00:00Z"
}
```

### Agent

```json
{
  "id": "uuid",
  "name": "string",
  "status": "Idle" | "Busy" | "Offline",
  "capabilities": ["string"],
  "registered_at": "2024-12-10T12:00:00Z",
  "last_heartbeat": "2024-12-10T12:05:00Z",
  "current_task": "string | null"
}
```

### Callback Payload

当实验状态变化时，crater-ohos 会向 `callback_url` 发送以下数据：

```json
{
  "experiment": "test-experiment",
  "status": "completed",
  "report_url": "https://crater.example.com/reports/test-experiment"
}
```

**事件类型：**

- `created` - 实验已创建
- `running` - 实验开始运行
- `completed` - 实验完成
- `failed` - 实验失败
- `aborted` - 实验中止

## 错误处理

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 400 | 请求参数错误 |
| 401 | 未授权（Token 无效或缺失） |
| 403 | 权限不足 |
| 404 | 资源不存在 |
| 409 | 冲突（如实验名称已存在） |
| 500 | 服务器内部错误 |

### 错误响应格式

```json
{
  "success": false,
  "error": "详细的错误描述信息"
}
```

### 常见错误

**实验名称已存在：**

```json
{
  "success": false,
  "error": "experiment test-experiment already exists"
}
```

**实验不存在：**

```json
{
  "success": false,
  "error": "experiment not found: test-experiment"
}
```

**实验状态不允许操作：**

```json
{
  "success": false,
  "error": "cannot edit experiment in running status"
}
```

**权限不足：**

```json
{
  "success": false,
  "error": "insufficient permissions"
}
```

**Token 无效：**

```json
{
  "success": false,
  "error": "invalid or expired token"
}
```

## 使用示例

### 示例 1: 创建并运行实验

```bash
# 1. 创建实验
EXPERIMENT_NAME="my-test-$(date +%s)"
curl -X POST http://localhost:3000/api/v1/experiments \
  -H "Authorization: Bearer ${API_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"${EXPERIMENT_NAME}\",
    \"toolchains\": [\"stable\", \"beta\"],
    \"mode\": \"build-and-test\",
    \"crate_select\": \"demo\",
    \"callback_url\": \"https://bot.example.com/webhook\"
  }"

# 2. 运行实验
curl -X POST "http://localhost:3000/api/v1/experiments/${EXPERIMENT_NAME}/run" \
  -H "Authorization: Bearer ${API_TOKEN}"

# 3. 查询实验状态
curl -H "Authorization: Bearer ${API_TOKEN}" \
     "http://localhost:3000/api/v1/experiments/${EXPERIMENT_NAME}"
```

### 示例 2: Agent 注册和心跳

```bash
# 1. 注册 Agent
RESPONSE=$(curl -X POST http://localhost:3000/api/v1/agents/register \
  -H "Authorization: Bearer ${API_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "worker-1",
    "capabilities": ["build", "test"]
  }')

AGENT_ID=$(echo $RESPONSE | jq -r '.data.id')

# 2. 发送心跳（每30秒）
while true; do
  curl -X POST "http://localhost:3000/api/v1/agents/${AGENT_ID}/heartbeat" \
    -H "Authorization: Bearer ${API_TOKEN}"
  sleep 30
done
```

### 示例 3: Python 客户端

```python
import requests

class CraterClient:
    def __init__(self, base_url, token):
        self.base_url = base_url
        self.headers = {
            'Authorization': f'Bearer {token}',
            'Content-Type': 'application/json'
        }
    
    def create_experiment(self, name, toolchains, mode='build-and-test', 
                         crate_select='demo', callback_url=None):
        data = {
            'name': name,
            'toolchains': toolchains,
            'mode': mode,
            'crate_select': crate_select,
        }
        if callback_url:
            data['callback_url'] = callback_url
        
        response = requests.post(
            f'{self.base_url}/api/v1/experiments',
            headers=self.headers,
            json=data
        )
        return response.json()
    
    def run_experiment(self, name):
        response = requests.post(
            f'{self.base_url}/api/v1/experiments/{name}/run',
            headers=self.headers
        )
        return response.json()
    
    def get_experiment(self, name):
        response = requests.get(
            f'{self.base_url}/api/v1/experiments/{name}',
            headers=self.headers
        )
        return response.json()
    
    def list_experiments(self):
        response = requests.get(
            f'{self.base_url}/api/v1/experiments',
            headers=self.headers
        )
        return response.json()

# 使用示例
client = CraterClient('http://localhost:3000', 'your-token-here')

# 创建实验
result = client.create_experiment(
    name='test-experiment',
    toolchains=['stable', 'beta'],
    callback_url='https://bot.example.com/webhook'
)
print(result)

# 运行实验
result = client.run_experiment('test-experiment')
print(result)

# 查询状态
result = client.get_experiment('test-experiment')
print(result)
```

### 示例 4: Rust 客户端

```rust
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateExperimentRequest {
    name: String,
    toolchains: Vec<String>,
    mode: String,
    crate_select: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    callback_url: Option<String>,
}

pub struct CraterClient {
    client: Client,
    base_url: String,
    token: String,
}

impl CraterClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }
    
    pub async fn create_experiment(
        &self,
        name: &str,
        toolchains: Vec<String>,
        callback_url: Option<String>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let request = CreateExperimentRequest {
            name: name.to_string(),
            toolchains,
            mode: "build-and-test".to_string(),
            crate_select: "demo".to_string(),
            callback_url,
        };
        
        let response = self.client
            .post(format!("{}/api/v1/experiments", self.base_url))
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .json(&request)
            .send()
            .await?;
        
        let result: ApiResponse<serde_json::Value> = response.json().await?;
        Ok(result.data.unwrap())
    }
    
    pub async fn run_experiment(
        &self,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .post(format!("{}/api/v1/experiments/{}/run", self.base_url, name))
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .send()
            .await?;
        
        Ok(())
    }
}

// 使用示例
#[tokio::main]
async fn main() {
    let client = CraterClient::new(
        "http://localhost:3000".to_string(),
        "your-token-here".to_string(),
    );
    
    // 创建实验
    let experiment = client.create_experiment(
        "test-experiment",
        vec!["stable".to_string(), "beta".to_string()],
        Some("https://bot.example.com/webhook".to_string()),
    ).await.unwrap();
    
    println!("Created: {:?}", experiment);
    
    // 运行实验
    client.run_experiment("test-experiment").await.unwrap();
    println!("Experiment started");
}
```

## 速率限制

当前版本暂无速率限制，但建议：

- 创建实验：每分钟不超过 10 次
- 查询请求：每秒不超过 10 次
- Agent 心跳：每 30 秒一次

未来版本可能会添加速率限制。

## 版本兼容性

crater-ohos 遵循语义化版本：

- **主版本号**：不兼容的 API 变更
- **次版本号**：向后兼容的功能新增
- **修订号**：向后兼容的问题修正

当前 API 版本为 `v1`，保证向后兼容。

## 获取帮助

- GitHub Issues: https://github.com/LuuuXXX/crater-ohos/issues
- 文档: [ARCHITECTURE.md](ARCHITECTURE.md)
- 示例: [examples/](../examples/)

## 更新日志

### v1.0.0 (2024-12)

- 初始版本
- 实验管理 API
- Agent 管理 API
- Bearer Token 认证
- Webhook 回调支持
