# Tasks

## 1. 固定覆盖率与删除证据

- [x] 1.1 使用当前 quality 的 Rust llvm-cov 与 Flutter test coverage 命令建立基线；在本 change 的 validation.md 记录提交、工作区状态、工具链、原始/现有过滤后覆盖行与总行、各 flag 前十个未覆盖热点及跳过的环境测试，确认报告可追溯且不把旧报告视为当前结果。
- [x] 1.2 对设计列出的 session 空入口、layer-shell 封装、OutputManager 辅助方法、FRB 两个应用入口、scanner 分支和 dead_code 字段建立证据表；通过仓库调用/导出/脚本/文档搜索及现有 change 契约核对，逐项注明删除、合并或保留原因。

## 2. 清理引擎遗留实现

- [x] 2.1 按证据删除 WallpaperSession::render_frame 空实现及无消费者的 Wayland layer-shell 封装、导出和专属依赖；保留实际 OutputManager、协议 Drop 与主引擎渲染路径，更新 crate 文档和过时迁移注释，使用 workspace all-targets check 与现有 Wayland 协议/生命周期测试验证无悬空引用及清理回归。
- [x] 2.2 删除无调用的 OutputManager 辅助方法、无意义状态或可等价简化的分支；为保留的输出 ready、scale、remove 和空输出行为补充适用的单元测试，在 validation.md 记录所有权字段的保留理由并通过 engine 测试。

## 3. 合并 Rust 边界与扫描分支

- [x] 3.1 合并 FRB 路径/source 应用入口的重复内部调度，保留各入口约定、typed properties 和卸载互斥锁内校验；扩展 boundary/lifecycle tests 验证无效 source、缺失组件、失败候选不覆盖旧状态和属性传递，必要时以既有生成器更新 bindings 并通过 bridge 测试。
- [x] 3.2 按证据简化 scanner 文件事件判断及无消费者事件分支，保留删除竞态处理和 debouncer 所有权；以临时目录及可确定的事件分类测试覆盖修改、删除、不支持文件和 watcher 释放，在 validation.md 记录行为等价依据并通过 library 测试。
- [x] 3.3 针对基线发现的 project 路径校验和 Web 组件状态有效遗漏补充定向测试，必要时抽取生产使用的纯判定函数；验证缺失/坏 manifest、越界路径、取消和卸载失败恢复，保留安全与资源清理分支，并将实际新增覆盖行及保留原因写入 validation.md。

## 4. 清理 Flutter 调用与补测

- [x] 4.1 将 controller/store tests 的 fake 对齐生产 typed source 调用，清理确认无用的转接和重复分支；覆盖 source 类型/属性传递、恢复失败保留持久化赋值及事件确认，更新相关测试说明并通过 controller_restore、controller_logic、tray_controls tests。
- [x] 4.2 根据基线补齐 settings_store 和 controller 的有效遗漏分支，保持新版存储格式、错误反馈、取消及组件状态行为；通过 settings_store 与相关 widget tests，记录清理/保留清单及覆盖率变化，不手改生成文件。

## 5. 综合验收

- [x] 5.1 完整运行当前 .github/workflows/ci.yml quality 的全部本地检查（MSRV、fmt、workspace Clippy、Rust llvm-cov、Flutter clean/pub get/analyze/test coverage），修复并重跑失败项；确认已移除的两个 CI 步骤没有恢复，记录命令和真实执行结果。
- [x] 5.2 使用与 1.1 相同工具链、命令和过滤口径比较 Rust/Flutter 覆盖率，确认各自不下降、未覆盖手写行数减少、现有 Codecov 门槛未降低且无新增生产代码排除；在 validation.md 列出删除量、合并项、新增覆盖及剩余环境限制，若已有当前提交远程报告则核对两个 flag，否则明确标注远程未验收。
