## Why

当前调试器界面的顶部控制面板和状态栏文字颜色对比度不足，在 macOS 深色模式下可读性较差。关键信息（状态徽章、目标名称、按钮标签）使用的灰色调（RGB 120-210）在深色背景（RGB 26,30,36）上不够醒目，影响快速识别和操作效率。

## What Changes

- 提升顶部控制面板（control panel）中状态徽章和目标信息的文字亮度
- 增强底部状态栏（status bar）中生命周期徽章和消息文字的对比度
- 统一各 UI 组件的文字颜色策略，确保关键信息使用高对比度颜色
- 保持现有 x64dbg 主题风格，仅调整文字亮度而不改变色相

**BREAKING**: 无破坏性变更

## Capabilities

### New Capabilities
无新增能力

### Modified Capabilities
- `ui-text-contrast`: 修改 UI 文字对比度规范，提升关键文字的可读性标准

## Impact

**受影响的代码**：
- `src/ui/control_panel.rs` - 控制面板文字颜色
- `src/ui/status_bar.rs` - 状态栏文字颜色
- `src/ui/style/mod.rs` - 可能需要更新调色板定义

**不受影响**：
- 寄存器面板和内存查看器（已有较好对比度）
- 核心调试逻辑
- 数据结构和 API

