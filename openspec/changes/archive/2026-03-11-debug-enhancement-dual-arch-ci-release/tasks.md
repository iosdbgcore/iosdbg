## 1. 调试核心扩展（Attach 能力）

- [x] 1.1 在 `src/types/mod.rs` 定义 `AttachRequest`、`AttachResult` 与错误分类枚举（permission_denied/target_not_found/timeout/lldb_error），并在 `src/core/types.rs` 落地会话状态机
- [x] 1.2 在 `src/core/engine.rs` 实现按 PID/进程名附加的统一入口，并返回结构化结果
- [x] 1.3 在 `src/core/session.rs` 增加会话状态机（detached/attaching/attached/failed）与状态迁移校验
- [x] 1.4 为附加前置检查实现目标存在性、输入合法性与权限检查逻辑

## 2. x64dbg 同构 UI 基线（Phase A）

- [x] 2.1 产出 x64dbg 同构 UI 基线清单（窗口布局、面板分区、核心交互、状态样式）
- [x] 2.2 在 `src/ui/layout/` 实现同构停靠布局骨架（反汇编区、寄存器区、内存区、控制区）
- [x] 2.3 在 `src/ui/style/` 建立 x64dbg 风格 token 映射（颜色、边框、状态态）
- [x] 2.4 在 `src/ui/control_panel.rs` 接入 Attach Drawer，并对齐 x64dbg 操作路径与按钮位置
- [x] 2.5 在 `src/ui/status_bar.rs` 展示附加状态与错误分类文案，保证失败时不污染当前会话
- [x] 2.6 将附加成功后的线程/暂停态与现有断点、单步控制联动

## 3. 过渡依赖治理与自研重写（Phase B）

- [x] 3.1 评估可复用 x64dbg UI 相关组件/资源的可行性与边界（技术、兼容性、许可）
- [x] 3.2 抽象 UI 适配层接口，隔离过渡依赖与业务逻辑
- [x] 3.3 完成同构 UI 的自研重写实现，替换过渡依赖并保持交互一致
- [x] 3.4 增加回归检查清单，验证重写前后布局、交互与状态反馈一致

## 4. CI 双架构构建能力

- [x] 4.1 更新 `.github/workflows/build-macos-client.yml`，使用 matrix 同时构建 `x86_64-apple-darwin` 与 `aarch64-apple-darwin`
- [x] 4.2 在 workflow 中统一产物命名规则 `rust-lldb-visual-debugger-<version>-<target>.zip`
- [x] 4.3 调整 `scripts/build_macos_client.sh` 以支持 CI 参数注入和架构化输出路径
- [x] 4.4 在 workflow summary 输出 commit/run 信息与双架构产物链接

## 5. Release 自动上传能力

- [x] 5.1 在发布触发路径中增加双产物上传到 GitHub Release Assets 的逻辑
- [x] 5.2 设置并校验发布任务 `permissions: contents: write`
- [x] 5.3 增加上传失败的显式失败机制与日志定位信息（标记失败资产及原因）

## 6. 测试与验证

- [x] 6.1 为附加请求参数校验、错误映射、状态迁移添加单元测试
- [x] 6.2 增加附加成功/失败场景的集成测试（含权限不足与目标不存在，见 `tests/attach_integration.rs`）
- [x] 6.3 增加 x64dbg 同构 UI 验收测试（布局分区、关键交互路径、状态提示，见 `src/ui/layout/mod.rs`、`src/ui/status_bar.rs`、`src/ui/control_panel.rs` 的测试）
- [x] 6.4 为 workflow 增加静态校验（触发条件、matrix 目标、资产命名）与冒烟验证
- [x] 6.5 验证 release 上传阶段在“单资产失败”时整体失败并保留定位日志

## 7. 文档与可维护性

- [x] 7.1 更新 `README.md`：补充附加调试前置条件、使用步骤与常见错误
- [x] 7.2 补充 CI/CD 文档：触发方式、产物命名规则、发布结果检查方法
- [x] 7.3 新增 UI 同构设计说明：与 x64dbg 的对应关系与差异说明
- [x] 7.4 为关键模块补充必要注释与排障说明（附加失败排查、权限链路说明）
