
## Push 前质量检查

每次 push 前，必须通过 `.github/workflows/ci.yml` 中 `quality` job 的全部检查；检查失败时先修复问题并重新检查，通过后再 push。
