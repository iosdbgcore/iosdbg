# rust-lldb-visual-debugger

> 基于 Rust + LLDB + egui 的可视化调试器

[English](./README.en.md)

一个现代化的可视化调试工具，提供直观的图形界面来调试二进制程序。通过集成 LLDB 调试引擎和 egui 图形框架，为开发者提供流畅的调试体验。

## 功能特性

- **二进制加载** - 支持加载和解析可执行文件
- **断点管理** - 设置、删除和管理断点
- **执行控制** - 单步执行、继续运行、暂停调试
- **寄存器查看** - 实时查看和监控 CPU 寄存器状态
- **内存检查** - 查看和分析进程内存内容
- **汇编显示** - 显示反汇编代码
- **执行可视化** - 图形化展示程序执行流程
- **LLDB 集成** - 基于强大的 LLDB 调试引擎
- **现代 UI** - 使用 egui 构建的响应式图形界面

## 安装

### 依赖要求

- Rust 工具链（edition 2021 或更高版本）
- Cargo 包管理器
- LLDB 开发库（可选，用于 real-lldb feature）

### 构建步骤

1. 克隆仓库：
```bash
git clone <repository-url>
cd iosDbg
```

2. 构建项目（使用 mock-lldb，无需 LLDB 依赖）：
```bash
cargo build --release
```

3. 或使用真实 LLDB（需要安装 LLDB 开发库）：
```bash
cargo build --release --features real-lldb
```

## 快速开始

运行调试器：

```bash
cargo run --release
```

启动后，使用图形界面：
1. 点击"加载二进制"按钮选择要调试的可执行文件
2. 设置断点并开始调试
3. 使用控制面板进行单步执行、继续运行等操作

## 使用文档

详细的技术规范和使用说明请参考：

- [二进制加载](./openspec/specs/binary-loading/spec.md)
- [断点管理](./openspec/specs/breakpoint-management/spec.md)
- [执行控制](./openspec/specs/execution-control/spec.md)
- [寄存器查看](./openspec/specs/register-inspection/spec.md)
- [内存检查](./openspec/specs/memory-inspection/spec.md)
- [汇编显示](./openspec/specs/assembly-display/spec.md)
- [执行可视化](./openspec/specs/execution-visualization/spec.md)
- [LLDB 集成](./openspec/specs/lldb-integration/spec.md)
- [UI 框架](./openspec/specs/ui-framework/spec.md)

## 贡献指南

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支（`git checkout -b feature/your-feature`）
3. 提交更改（`git commit -m 'Add some feature'`）
4. 推送到分支（`git push origin feature/your-feature`）
5. 提交 Pull Request

**注意**：修改 README 时，请同步更新 `README.md` 和 `README.en.md` 两个文件。

## 许可证

MIT License
