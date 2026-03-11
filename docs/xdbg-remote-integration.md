# x64dbg 远程接口接入说明

本文档说明如何在当前项目中启用 x64dbg 远程接入能力，以及如何触发核心调试动作。

## 1. 连接参数

控制栏新增 Remote 接入参数：

- `endpoint`：远程地址（默认建议 `127.0.0.1:27400`）
- `token`：可选鉴权令牌（如远程端要求）
- `timeout`：超时毫秒数（建议 500~5000）
- `retry`：连接失败重试次数

说明：
- 调试/测试场景可使用 `mock://xdbg` 作为本地模拟 endpoint。
- token 为 `bad-token` 时会触发鉴权失败分类（用于测试错误路径）。

## 2. 连接与断开

1. 在顶部控制栏填写 endpoint、token、timeout、retry。
2. 点击 `Connect` 发起连接。
3. 状态栏显示 `Remote: Connected` 后表示会话可用。
4. 点击 `Disconnect` 可主动断开会话并回到 `Remote: Disconnected`。

## 3. 核心动作触发方式（UI -> 远程命令）

| UI 操作 | 远程方法名 | 说明 |
|---|---|---|
| Continue | `debug.continue` | 继续执行 |
| Step Over | `debug.step_over` | 单步越过 |
| Step In | `debug.step_in` | 单步进入 |
| Pause | `debug.pause` | 暂停执行 |
| Read Registers | `debug.read_registers` | 读取寄存器快照 |
| Read Memory | `debug.read_memory` | 读取内存片段 |

## 4. 验证建议

- 使用 `mock://xdbg` 连接并执行 Step/Continue/Pause，确认状态更新链路可达。
- 使用 `bad-token` 验证 `auth_failed` 错误分类。
- 设置环境变量 `IOSDBG_REMOTE_FORCE_TIMEOUT=1` 验证超时处理与降级提示。

## 5. 常见故障排查

- **连接失败（connection_failed）**
  - 检查 endpoint 是否可达（IP/端口/防火墙）。
  - 检查远程服务是否已启动。

- **鉴权失败（auth_failed）**
  - 校验 token 是否正确。
  - 检查远程端是否启用了鉴权。

- **超时（timeout）**
  - 适当增加 timeout 参数。
  - 检查网络延迟与远程服务负载。

- **协议错误（protocol_error）**
  - 检查 endpoint 格式是否为 `host:port` 或测试用 `mock://...`。
  - 检查输入参数是否合法（timeout > 0）。

