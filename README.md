# MomoLite

MomoLite 是一个本地 Windows 桌面英语学习软件，目标是把背过的单词和句子变成可读、可听、可训练的真实语境。

当前方向：

- 课程包、课时和句子导入
- 中译英原位填空训练
- 错词重填和学习记录
- Piper 离线英文朗读
- 单词书、AI 场景课和沉浸式复习探索

技术栈：

- Tauri 2
- React 19 + TypeScript + Vite
- Rust
- SQLite
- Piper TTS

常用命令：

```powershell
npm.cmd install
npm.cmd run build
npm.cmd run test:answer-rules
npm.cmd run tauri -- build
```

项目文档在 `docs/` 目录中。
