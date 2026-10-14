# 后续 Scene 功能

2026-10-15 用户明确要求移出当前交付，后续另行实现。以下全部未完成，不计为当前功能通过。已有部分内部实现不等于完整兼容验收。

- [ ] 3.3 实现 animated texture 帧选择和 Rust 粒子更新，随机种子与时间步可注入；验证连续帧变化、确定性、pause 冻结及 resume 无大跳步。
- [ ] 4.2 在 Rust 宿主实现内存/stack/执行期限、timer/audio registration 数量预算，限制 pending jobs 和禁用任意 host I/O；以死循环、内存耗尽、timer 取消/重入、超量注册及 I/O 探测验证可控失败和后续输出可用。
- [ ] 4.3 直接将 Rust AudioSnapshot 连接到 shader/effect/particle 与稳定 Float32Array left/right/average，保留 16/32/64 下采样和 RMS 扩展；以非恒定频带、非法/过期输入、脚本文字和双 Scene fixture 验证声道/类型/归零与回收。
- [ ] 4.4 迁移真实 private monitor→FFT→Rust Scene 验证；播放私有 null sink 的低/高频音检查实际 shader/效果/粒子/脚本响应、wallpaper mute 独立及采集断线恢复，记录 consumer/native/GPU 基线，禁止操作用户音频设备。
- [ ] 4.5 建立可注入 monotonic/wall-clock/timezone 输入；以跨分钟/日期/时区和暂停恢复 fixture 验证 clock text、timer、首帧当前日期及 delta 无暂停大跳步。

- [ ] Scene 多布局支持和实时 GUI 属性编辑／持久化。
