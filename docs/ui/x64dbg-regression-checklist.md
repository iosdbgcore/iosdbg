# x64dbg 同构重写回归检查清单

用于任务 3.4，验证 Phase A（过渡实现）与 Phase B（自研重写）一致性。

## A. 布局一致性

- [ ] 左侧反汇编区域宽度优先（> 50%）
- [ ] 右上寄存器、右下内存分区存在且比例稳定
- [ ] 顶部控制栏与底部状态栏仍可见

## B. 交互一致性

- [ ] PID 模式可附加
- [ ] 进程名模式可附加
- [ ] 附加成功后 Step/Continue 可用
- [ ] 断点切换交互保持一致

## C. 状态反馈一致性

- [ ] 附加中展示 Attaching
- [ ] 成功展示 Attached
- [ ] 失败展示 Failed + 错误分类
- [ ] 错误不会污染既有已附加会话

## D. CI 产物一致性

- [ ] x64/arm64 构建均成功
- [ ] 产物命名符合 `rust-lldb-visual-debugger-<version>-<target>.zip`
- [ ] 发布任务校验双资产缺一即失败

## E. 记录项

- [ ] 若 UI 行为有差异，已在 `docs/ui/x64dbg-reuse-feasibility.md` 更新
- [ ] 若新增限制，已同步更新 README 与 CI/CD 文档
