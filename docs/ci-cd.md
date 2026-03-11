# CI/CD 指南（macOS 双架构 + Release 上传）

## 触发方式

- `pull_request`：校验构建流程与工作流约束
- `push` 到 `main/master`：执行双架构构建并上传 Actions Artifacts
- `push` tag（`v*`）：执行双架构构建并触发发布上传
- `release.published`：执行双架构构建并上传 Release Assets
- `workflow_dispatch`：支持手动触发，可选填写 features 和发布开关

## 产物命名

统一命名为：

`rust-lldb-visual-debugger-<version>-<target>.zip`

其中 `target` 为：

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

## 发布流程

1. build job 并行构建双架构并上传 Actions Artifacts
2. publish-release job 下载两个架构产物
3. 在上传前执行“资产存在性校验”
4. 任一资产缺失即失败并输出明确错误
5. 校验通过后上传到 GitHub Release Assets

## 结果检查

- Actions Summary 包含 commit、run id、产物路径与运行链接
- Release Summary 包含 x64/arm64 资产路径和 release url
- 如发布失败，优先查看“Verify release assets exist”步骤

## 常见问题排查

- **只看到单架构产物**：检查 matrix target 是否包含两个目标
- **发布权限不足**：确认 publish job 设置 `permissions: contents: write`
- **命名不符合约定**：检查 `scripts/build_macos_client.sh` 产物前缀与 output-dir 参数
