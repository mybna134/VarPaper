# Design

## Context

动机见 proposal.md。当前 Flutter 持久化状态并通过 FRB 调度 Rust service；engine 可依赖 library 的无状态项目/资源 API。现有 quality 分别上传 Rust 与 Flutter coverage，Codecov 两个 flag 都允许 carryforward。用户已移除 package/manifest 与 native fixture 两个 CI 步骤，本变更保留这一决定。

只读检查发现：

| 位置 | 观察 | 处理原则 |
| --- | --- | --- |
| `engine/session.rs::render_frame` | 注释明确已不用，函数只返回成功；仓库搜索未发现调用 | 确认导出和测试后删除空入口 |
| `wayland/layer_shell.rs` | 独立 SCTK 封装，与 engine 内原生 layer surface 路径并存，crate root 仍导出 | 跟踪构建、示例和导出，确认无消费者后删除重复封装及专属依赖 |
| `wayland/output.rs` | `output_at` 忽略坐标并返回首个输出；`OutputManager` 本身由 engine 实际使用 | 删除未使用辅助方法，保留实际输出跟踪 |
| `rust/src/bridge.rs` | `apply_wallpaper` 与 `apply_wallpaper_source` 并存；Flutter 生产调用后者；新版入口在锁内复查组件状态 | 合并重复内部调度，保留约定入口及竞态保护 |
| `library/scanner.rs` | `path.is_file()` 分支内再次判断 `path.exists()`；debouncer 虽有 dead_code 标注仍负责 watcher 所有权 | 区分冗余判定与删除竞态，保留 watcher 生命周期 |
| `coverage/lcov.info` | 本地旧报告 1203/2128；其中生成 FRB 文件占大量零覆盖行，手写 main 为 993/1033、store 为 192/206 | 该报告未绑定当前提交，不能作为约 64% 的来源或当前基线 |
| `codecov.yml` | 已排除生成代码及部分依赖显示/GPU 的模块；patch target 为 80% | 保持既有过滤范围；报告原始与过滤后统计 |

以上是候选与观测，不代表每项都可删除。尤其未覆盖分支、公共导出和 RAII 字段需要进一步证据。

## Goals / Non-Goals

**Goals:**

- 每项删除可追溯到调用、契约与构建证据；相同职责尽量保留一个实现。
- 将有效业务判定抽为小型纯函数，真实运行代码使用这些函数，以确定性测试覆盖保留分支。
- 同一工具链、命令、过滤范围下，收尾 Rust 和 Flutter 各自覆盖率不低于基线，未覆盖手写代码行数下降；记录具体差值与剩余热点。

**Non-Goals:**

- 不设未经基线支持的整体覆盖率百分比目标，不将未执行的 native fixture 算作验证成功。
- 不删除五类型功能、unsupported 诊断、路径安全校验、超时、重试、RAII 清理或组件卸载竞态保护来提高比例。
- 不新增旧数据兼容，不完成其他 change 中延期的 Scene 功能，不恢复 daemon 架构。
- 不重新启用已移除的 CI 步骤，不扩大 coverage ignore 或降低 Codecov 门槛。

## Decisions

### 1. 删除依据与覆盖率分开记录

在本 change 的 `validation.md` 记录每个候选的定义、生产调用、测试/脚本/导出引用、结论和验证命令。结论分为删除、合并、保留并补测。零覆盖仅用于排序，不能证明不可达。相比按覆盖率直接删代码，这能保留硬件依赖路径和异常处理。

Rust 基线使用 workflow 的 llvm-cov 命令；Flutter 使用 `flutter test --coverage`。记录 commit、工作区状态、工具链、覆盖/总行、文件热点及时间。使用相同现有 Codecov ignore 生成可比汇总，同时保留原始 lcov。两个 flag 分别比较；只有两者来自同一提交、非 carryforward 时才称为合并结果。用户所述约 64% 暂为未核实背景，不作为验收阈值。

### 2. 优先删除已被主路径替代的内部实现

先清理 session 空函数、独立 layer-shell 封装及未用输出辅助方法，再处理调用链相连的字段、导出、文档与依赖。`OutputManager` 仍是生产依赖，不整体移除。`FileWatcher.debouncer` 和协议对象 Drop 负责资源生命周期，即使不直接读取也保留。外部导出若有明确消费者或现有契约要求，保留并记录；有效 API 删除需要另行变更范围。

### 3. 合并边界逻辑时保持验证时序

FRB 调用使用相同内部应用流程，路径解析和 typed source 属性准备保留各自语义。组件可用性必须在与卸载互斥的锁内复查。生成桥接文件通过项目既有生成方式更新，不手改生成产物。Flutter fake 不应将已弃用接口继续作为唯一测试路径；测试应模拟真实 source 调用及事件确认。

### 4. 针对有效未覆盖分支补测

优先验证 scanner 事件分类、无效/缺失项目和路径边界、Web 组件缺失/错误/取消/卸载状态、controller 恢复失败和 store 新格式往返。复用现有 boundary、lifecycle 和 controller/store tests；按实际 coverage 热点选取具体遗漏。对于 `is_file` 后文件被删除的时序，不假设文件系统状态永远不变；只合并语义等价的判定。保留供手动验收的 native fixture；仅在改到原生生命周期时定向执行相关 fixture，不恢复完整 native CI 门禁。

## Risks / Trade-offs

- [公共导出未在仓库内调用，但可能有消费者] → 搜索示例、脚本及现有契约；无法证明可移除时保留。
- [将未读字段视为死代码破坏 Drop 或异步生命周期] → 明确所有权作用，测试关闭、取消、卸载及失败清理。
- [合并校验引入 TOCTOU 或改变旧赋值保留行为] → 保留锁内检查，为失败候选和组件卸载增加定向断言。
- [本地与远程覆盖率因生成代码或 carryforward 不一致] → 固定统计范围，分 flag 报告；缺少当前提交报告时明确限制。
- [现有主规格仍含旧架构要求] → 本次不改变需求，使用用户已批准的当前 change 设计核对行为，历史规格同步另行处理。

## Migration Plan

无数据或部署迁移。按 Task 分批落地，先记录基线，再删除/合并，再补测，最后运行当前 quality job 的全部本地检查并比较结果。保留用户现有改动，不自动提交/推送；回退以本 change 的独立补丁为单位。远程 Codecov 若需提交后验证，明确列为待验收，不用旧 run 代替。
