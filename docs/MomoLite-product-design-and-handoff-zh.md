# MomoLite 产品设计与工程交接文档

更新日期：2026-04-27

本文档用于把当前对话中的产品判断、工程现状、已实现功能、用户真实诉求、下一阶段开发计划一次性交接给新的 AI 对话。新的对话请先读本文档，再读代码，不要从零开始猜。

## 1. 一句话定位

MomoLite 是一个本地 Windows 桌面英语学习软件。它的核心不是普通背单词，而是把用户当天或近期背过的词，转化成高质量母语者日常场景对话，再通过沉浸阅读、听力、中译英、听写、跟读和复习，让单词真正变成可以听懂、读懂、打出来、说出来的英语。

产品核心承诺：

```text
把背过的单词，变成真实语境中的英语能力。
```

## 2. 当前项目位置与运行方式

项目目录：

```text
C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite
```

本地数据库：

```text
C:\Users\Administrator\AppData\Roaming\cn.local.momolite\momolite.sqlite
```

当前桌面可执行文件：

```text
C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite\src-tauri\target\release\momolite.exe
```

当前安装包：

```text
C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite\src-tauri\target\release\bundle\nsis\momolite_0.1.0_x64-setup.exe
```

重要说明：

- 用户希望使用桌面软件打开，而不是网页。
- Vite 网页预览不能直接访问 Tauri 的 `invoke`、SQLite 和 Piper。
- 如果在网页里打开，只能作为 UI 预览，不能代表真实桌面能力。
- 后续验收必须以 Tauri 桌面应用为准。

## 3. 当前技术栈

- 桌面框架：Tauri 2
- 前端：React 19 + TypeScript + Vite
- 图标：lucide-react
- 后端：Rust
- 数据库：SQLite + rusqlite
- 离线 TTS：Piper TTS
- 当前英文模型：en_US-lessac-medium
- 平台目标：Windows 本地个人学习工具

当前不做：

- 账号系统
- 会员
- 支付
- 云同步
- 多用户
- 课程市场

## 4. 当前数据库和数据状态

最近观察到的本机数据：

- 课程包：8 个
- 课时：52 个
- 句子：444 条
- 学习记录：1 条
- 每日统计：1 条

当前核心表：

- `course_packs`：课程包
- `lessons`：课时
- `sentence_items`：句子
- `review_logs`：学习记录
- `daily_stats`：每日统计

已有数据库设计原则：

- 句子只属于课时。
- 课时属于课程。
- 学习记录属于句子。
- 不重复保存可以从关系推导出来的课程归属。
- 使用外键防止孤儿数据。
- 删除课程会级联删除课时、句子和学习记录。
- 多表 join 查询必须显式限定列名，例如 `r.id`、`s.id`、`l.id`，否则 SQLite 可能报 `ambiguous column name: id`。

## 5. 已完成的产品功能

### 5.1 信息架构重构

当前前端视图已经从早期工程后台式结构，重构为更接近产品的结构：

- 首页
- 课程包
- 课程详情
- 学习页
- 统计页

当前没有引入 React Router，主要用本地 React state 控制页面。

### 5.2 课程与课时

已支持：

- 创建课程包。
- 一个课程包包含多个课时。
- 一个课时包含多条句子。
- 在课程详情页查看课时卡片。
- 点击课时卡片后查看该课时下的句子。
- 从某个课时直接开始学习。

### 5.3 批量导入

已支持用标题识别课时，例如：

```text
## 第一天
Yeah, the economy is developing faster than most people expect.=是的，经济发展比大多数人预期的要快。

## 第二天
Hey, did you receive my text yesterday?=嘿，你收到我昨天的短信了吗？
```

也支持：

```text
英文=中文=音标
```

当前导入已有前端去重：

- 按当前课程内 `english + chinese` 判断重复。
- 展示新增数量和跳过重复数量。
- 不自动合并或删除已经存在的重复课程名。

### 5.4 中译英句子训练

当前核心学习流程：

1. 进入学习页。
2. 自动朗读英文。
3. 显示中文。
4. 英文标点固定展示。
5. 用户只填写英文单词。
6. 空格跳到下一个词。
7. 左右方向键可以移动到前一个或后一个词。
8. 提交答案。
9. 如果有错，错词位置标红，正确词锁定，只重填错词。
10. 如果全对，先显示完整英文答案。
11. 用户点击“下一句”后记录并进入下一句。

用户明确要求并已修正：

- 不要在横线上显示很多 `_ _ _`。
- 打字到第 6 个词时，可以用左右方向键控制具体输入第几个词。
- 全部正确后要显示原句答案，增强记忆，而不是立刻跳走。

### 5.5 离线发音

当前使用：

- Piper TTS
- en_US-lessac-medium 英文模型

实现方式：

- Rust 后端调用 Piper。
- 生成 wav 音频。
- 同一句话音频本地缓存。
- Piper 失败时回退系统语音。

### 5.6 已修复的重要问题

已修复：

- 网页预览下 `invoke undefined` 报错，改为友好提示用户使用桌面软件。
- SQLite 查询 `ambiguous column name: id`，通过限定 `r.id` 等列名解决。
- 学习页方向键切换词位。
- 答对后展示完整答案。
- 课程详情支持点进课时看句子，再学习该课时。

## 6. 当前 Git 里程碑

已有关键提交：

- `bffee9b` baseline: current MomoLite MVP
- `3570d21` refactor: productize MomoLite study experience
- `c5a42e9` fix: guard Tauri calls outside desktop runtime
- `9851589` fix: qualify review log query columns
- `1ff0f95` feat: improve lesson study flow
- `0d5b2c5` docs: add product handoff and roadmap

后续继续保持：

- 每完成一个清晰阶段就提交一次。
- 不要大改一堆文件却没有中间提交。
- 不要破坏已有 SQLite 数据。

## 7. 用户真实目标

用户不是只想做一个工具页面，而是想学习如何和 AI 一起开发一个真正可用的软件产品。

用户希望 AI 扮演：

- 20 年经验产品经理
- 20 年经验全栈工程师
- 老师和带教者

协作方式要求：

- 边开发边解释产品判断。
- 边开发边解释代码结构。
- 教用户如何给 AI 提需求、如何拆任务、如何验收。
- 不要只给抽象理论，要落到代码、数据、UI、测试、打包。
- 用户是小白实习生视角，需要把思维过程讲清楚。

推荐教学主线：

```text
用户痛点 -> 产品行为 -> 数据模型 -> UI 流程 -> 后端命令 -> 前端状态 -> 测试验证 -> 打包发布
```

## 8. 新产品愿景

用户提出的新核心模式：

```text
背单词 app 导出的词
-> 选取当天或近期背过的词
-> LLM 生成母语者日常高质量场景对话
-> 形成课程
-> 沉浸式阅览复习
-> 中译英训练
-> 听力听写训练
-> 跟读和长期复习
```

这个模式的价值：

- 单词不再孤立。
- 用户背过的词会进入真实语境。
- 复习不只是看释义，而是在读、听、打、说中反复遇到。
- 对话可以覆盖日常生活，帮助英语逐渐成为第二语言。

产品差异点：

- 不是普通词典。
- 不是普通背单词。
- 不是简单 AI 造句。
- 重点是“词汇到高质量完整场景”的闭环。

## 9. 最难也最核心的问题：高质量对话

产品的护城河不是有多少按钮，而是能否稳定生成高质量对话。

高质量对话定义：

- 像母语者日常生活中真的会说的话。
- 有完整场景，不是孤立例句。
- 有上下文、人物意图和自然情绪流动。
- 目标词自然出现，不生硬塞词。
- 句子适合听力、口语和打字训练，不能过长。
- 中文翻译自然。
- 覆盖目标词，且能展示覆盖率。
- 提取常用表达和口语 chunks。

用户补充的重要要求：

- 一个场景可以包含 6 到 12 个目标词，甚至更多。
- 不要死板限制为 6 个单词或 6 个句子。
- 重点是完整场景和高质量对话。
- 默认偏日常生活，但作为产品要允许更多受众选择风格。

## 10. spoken-english-builder Skill 参考

用户本地已有相关 skill：

```text
C:\Users\Administrator\.codex\skills\spoken-english-builder\SKILL.md
```

该 skill 的核心规则：

- 提取并去重词汇或短语。
- 按日常生活场景分组。
- 生成自然母语者日常对话。
- 覆盖所有用户提供的目标词。
- 输出沉浸式复习 Markdown。
- 输出可导入听写软件的英文/中文成对文本。
- 生成后做覆盖率检查。

相关参考文件：

```text
C:\Users\Administrator\.codex\skills\spoken-english-builder\references\生成规则.md
C:\Users\Administrator\.codex\skills\spoken-english-builder\references\格式规范.md
C:\Users\Administrator\.codex\skills\spoken-english-builder\references\词汇分配算法.md
```

MomoLite 后续应把这个 skill 中的理念产品化，而不是简单复制 Markdown 输出。

应吸收的结构：

- 全景词汇索引表。
- Story 标题和场景。
- 核心词汇列表。
- 中英对照对话。
- 英文目标词高亮。
- 关键表达。
- 覆盖率报告。
- 听写导入流。

需要产品化的地方：

- 从 Markdown 变成可交互的应用页面。
- 从一次性生成变成可保存、可学习、可复习的课程。
- 从手工复制导入变成一键生成训练数据。

## 11. 目标用户和主要使用场景

第一阶段目标用户：

- 想把背过的单词真正用起来的人。
- 有一定词汇输入，但缺少语境和输出训练的人。
- 想练听力、口语、阅读和中译英表达的人。
- 愿意使用本地桌面工具进行个人学习的人。

典型场景：

1. 用户从背单词 app 导出今天背过的 30 个词。
2. 导入 MomoLite。
3. 选择“日常生活 / B1 / 自然口语”。
4. MomoLite 生成 3 到 5 个完整场景对话。
5. 用户先看沉浸式复习页。
6. 再做中译英句子训练。
7. 再做无中文提示的听写。
8. 错句和生词进入复习队列。
9. 第二天继续学习最近词汇和薄弱句子。

## 12. 产品信息架构规划

当前已有：

- 首页
- 课程包
- 课程详情
- 学习页
- 统计页

建议下一阶段逐步演进为：

- 首页
  - 今日继续学习
  - 今日词汇批次
  - 最近生成场景
  - 错句复习
  - 少量关键统计
- 词汇
  - 导入词汇
  - 词汇批次
  - 单词卡片复习
  - 熟悉度管理
- 场景生成
  - 选择词汇批次
  - 设置场景风格和难度
  - LLM 生成预览
  - 覆盖率检查
  - 保存为场景或课程
- 课程
  - 课程包列表
  - 课程详情
  - 课时列表
  - 句子列表
  - 从课时开始学习
- 复习
  - 沉浸式阅读
  - 中译英
  - 听力听写
  - 错句复习
  - 跟读练习
- 统计
  - 今日完成
  - 连续学习
  - 正确率
  - 薄弱词句
- 设置
  - LLM 配置
  - TTS 配置
  - 数据库位置
  - 备份与恢复

## 13. 核心学习闭环

推荐长期闭环：

```text
词汇输入
-> 词汇批次
-> 场景生成
-> 沉浸阅读
-> 中译英主动回忆
-> 听力听写
-> 跟读
-> 错词错句复习
-> 下次生成时再次复现薄弱词
```

其中最重要的两个训练：

- 中译英：训练主动表达和语法组织。
- 听力听写：训练听音识别、拼写、语块和弱读感知。

背单词模块的定位：

- 不是和墨墨背单词正面竞争。
- 它是 MomoLite 的输入源和复习辅助。
- 它的价值是把词汇输送到场景和句子训练中。

## 14. 模块设计

### 14.1 词汇导入模块

目标：

- 支持用户从背单词 app 导出的内容中导入词汇。
- 把当天或最近背过的词形成一个可复用批次。

MVP 功能：

- 粘贴文本或导入 CSV。
- 自动识别英文词、短语、中文释义。
- 预览识别结果。
- 用户确认后保存。
- 按 `word + meaning + batch` 做去重。
- 记录来源、日期、熟悉度。

需要用户提供：

- 一个真实背单词 app 导出样例，至少 10 到 20 行。
- 最好包含词、中文释义、日期、熟悉度或学习状态。

建议数据表：

```sql
vocabulary_batches
- id
- name
- source
- imported_at
- note

vocabulary_items
- id
- batch_id
- word
- meaning
- phonetic
- example
- familiarity
- source_status
- created_at
- updated_at

vocabulary_reviews
- id
- vocabulary_item_id
- rating
- reviewed_at
```

### 14.2 背单词与轻复习模块

参考方向：

- 墨墨背单词：熟悉度反馈和复习节奏。
- 无痛英语：低压力、低摩擦、让用户愿意开始。

MVP 设计：

- 单词正面：英文词或短语。
- 背面：中文释义、发音、来自哪个场景。
- 反馈按钮：忘了 / 模糊 / 认识 / 熟悉。
- 每天只推少量关键词，不制造压力。
- 优先服务于后续场景生成，不要先做复杂复习算法。

后续可加：

- FSRS 或简化间隔复习。
- 薄弱词重新进入场景生成。
- 一个词在多个场景中复现的历史。

### 14.3 LLM 设置模块

LLM 是生成高质量对话的关键。

因为 MomoLite 是本地桌面软件，不能内置私人 API Key。应该让用户自己配置。

MVP 配置项：

- Provider 名称。
- OpenAI-compatible base URL。
- API key。
- Model 名称。
- 连接测试。
- 默认场景风格。
- 默认难度。

产品策略：

- 优先支持 OpenAI-compatible API。
- 允许用户自己填 OpenAI、兼容服务、代理网关或本地服务。
- 后续可支持 Ollama 等本地模型。
- 网上免费 API 的可用性不稳定，可以提供文档说明，不应硬编码。

安全原则：

- API Key 只保存在本地。
- 不上传用户词汇和学习数据，除非用户主动调用 LLM。
- 调用 LLM 前明确告诉用户哪些词会被发送。

### 14.4 场景对话生成模块

输入：

- 词汇批次。
- 目标词数量。
- 场景风格。
- 难度。
- 句子长度偏好。
- 是否偏日常口语。

输出：

- 场景标题。
- 场景描述。
- 对话行。
- 每行英文。
- 每行中文翻译。
- 涵盖的目标词。
- 关键表达。
- 覆盖率报告。

质量控制：

- 目标词覆盖率必须接近或等于 100%。
- 若未覆盖，要提示缺失词并允许重新生成。
- 允许同一个词的变形出现，但要能映射回原目标词。
- 如果对话显得生硬，要支持重新生成。

建议数据表：

```sql
generated_scenes
- id
- batch_id
- title
- title_cn
- scene_type
- difficulty
- prompt
- raw_response
- coverage_rate
- created_at

scene_lines
- id
- scene_id
- speaker
- english
- chinese
- sort_order

scene_vocabulary_links
- id
- scene_id
- vocabulary_item_id
- matched_text
```

### 14.5 沉浸式复习模块

目标：

- 在软件内实现类似飞书沉浸式复习文档的美观阅览体验。
- 用户先通过阅读和听音吸收，再进入训练。

页面结构：

- 词汇索引表。
- 场景介绍。
- 中英对照对话。
- 目标词高亮。
- 关键表达。
- 播放整段音频。
- 单句播放。
- 一键进入中译英。
- 一键进入听写。

注意：

- 不要只渲染一大段 Markdown。
- 要做成真正的学习界面。
- 阅读体验要安静、清晰、耐看。

### 14.6 训练模块

已有模式：

- 中译英填空。

下一步应加：

- 听力听写：只听英文，不显示中文，用户输入英文。
- 整句听写：适合高阶。
- 单词空格听写：适合中低阶。
- 错句复习：只练曾经错过的句子。
- 场景训练：按生成场景顺序练习。
- 跟读模式：先听、再读、自己确认是否通过。

听写模式建议流程：

1. 显示播放按钮和进度。
2. 自动播放英文。
3. 不显示中文。
4. 用户输入英文。
5. 可多次播放。
6. 提交后显示原文、中文、差异。
7. 错句进入复习。

### 14.7 首页模块

首页应该回答一个问题：

```text
我今天打开软件，下一步该学什么？
```

推荐首页内容：

- 今日继续学习。
- 最近词汇批次。
- 最近生成场景。
- 错句复习入口。
- 今日目标进度。
- 连续学习天数。
- 少量统计，不要像后台报表。

### 14.8 统计模块

统计不是为了炫技，而是为了帮助用户继续学。

保留：

- 今日完成句子。
- 正确率。
- 连续学习天数。
- 近期薄弱词。
- 近期薄弱句。
- 场景覆盖词数。

不要过早做：

- 复杂排行榜。
- 社交排名。
- 过多图表。

## 15. 后端命令规划

继续使用 Tauri command 作为前后端边界。

建议新增命令：

```text
import_vocabulary_batch
list_vocabulary_batches
get_vocabulary_batch
review_vocabulary_item
save_llm_settings
get_llm_settings
test_llm_connection
generate_scene_preview
save_generated_scene
create_course_from_scene
list_generated_scenes
get_generated_scene
start_dictation_session
submit_dictation_answer
```

工程原则：

- 前端负责交互和预览。
- Rust 后端负责 SQLite、文件、TTS、LLM 调用封装。
- 复杂导入解析可以先放前端，稳定后再沉到 Rust。
- 所有写数据库的操作都要可测试。

## 16. UI 和体验原则

整体风格：

- 清爽。
- 安静。
- 有轻微游戏化反馈。
- 不要像工程后台。
- 不要把所有数据堆出来。

学习页原则：

- 低干扰。
- 只保留当前训练需要的信息。
- 不显示数据库路径。
- 不显示全局课程选择器。
- 右侧队列默认收起。
- 重点是听、看、打、反馈。

课程详情原则：

- 用户能看到课程有多少课。
- 能点进某一课看到句子。
- 能从课程或课时开始学习。

词汇和生成原则：

- 导入前要预览。
- 生成前要让用户知道会用哪些词。
- 生成后要显示覆盖率和缺失词。
- 保存前可以预览和重新生成。

## 17. 下一阶段开发路线图

### v0.3：词汇导入与基础词汇库

目标：

- 建立“背单词 app -> MomoLite”的输入管道。

任务：

- 新增词汇模块入口。
- 支持粘贴或导入词汇文本。
- 根据真实导出样例写解析器。
- 做导入预览。
- 新增 `vocabulary_batches` 和 `vocabulary_items`。
- 保存词汇批次。
- 展示词汇批次详情。
- 做最简单的单词卡片复习。

验收：

- 用户能导入 10 到 20 行真实导出词汇。
- 能看到批次。
- 能看到每个词和释义。
- 重复词不会无限新增。

### v0.4：LLM 设置与单场景生成

目标：

- 从一个词汇批次生成一个高质量场景预览。

任务：

- 新增 LLM 设置页。
- 支持 OpenAI-compatible 配置。
- 保存 API Key、base URL、model。
- 测试连接。
- 选择一个词汇批次生成场景。
- 展示覆盖率。
- 支持重新生成。
- 保存生成结果。

验收：

- 用户能配置自己的 LLM。
- 能用 6 到 12 个词生成一个自然场景。
- 能看到哪些词被覆盖。
- 生成失败时有清楚错误提示。

### v0.5：场景转课程

目标：

- 让生成内容进入现有学习系统。

任务：

- 把 generated scene 转为 course / lesson / sentence_items。
- 保留 scene 和 vocabulary 的关联。
- 课程详情中标注来源：词汇生成。
- 从场景开始中译英训练。

验收：

- 用户能从一批词生成课程。
- 生成的句子能进入当前学习页。
- 不破坏已有课程数据。

### v0.6：沉浸式复习页

目标：

- 把 skill 中的飞书沉浸式复习体验产品化。

任务：

- 词汇索引表。
- 中英对照对话。
- 目标词高亮。
- 关键表达。
- 单句播放。
- 进入中译英。
- 进入听写。

验收：

- 用户可以像阅读一篇漂亮复习文档一样学习场景。
- 从复习页能进入训练。

### v0.7：听力听写

目标：

- 加入无中文提示的听力输入训练。

任务：

- 新增训练模式：听写。
- 播放英文，不显示中文。
- 用户输入英文。
- 提交后对比原文。
- 复用或扩展现有答案规则。
- 记录听写结果。

验收：

- 听写流程完整可用。
- 错误反馈清楚。
- 错句能进入复习队列。

### v0.8：错词错句复习与轻量间隔

目标：

- 让用户每天知道该复习什么。

任务：

- 错句队列。
- 薄弱词队列。
- 今日复习入口。
- 简化间隔复习。
- 连续学习和今日目标。

验收：

- 首页能清楚提示今天该继续什么。
- 错过的内容会被再次练到。

## 18. 验证命令

每个阶段完成后运行：

```powershell
npm.cmd run build
npm.cmd run test:answer-rules
$env:Path = 'C:\Users\Administrator\.cargo\bin;' + $env:Path; cargo test --lib -- --nocapture
$env:Path = 'C:\Users\Administrator\.cargo\bin;' + $env:Path; npm.cmd run tauri -- build
```

注意：

- 如果正在运行 `momolite.exe`，Windows 可能锁住 release 文件，打包前先关闭软件。
- 桌面功能要用 Tauri app 验收，不要只看浏览器。
- 文档变更不需要跑完整构建，但代码变更需要。

## 19. 数据安全和迁移原则

因为本机已有真实数据，后续开发要谨慎：

- 不要删除用户现有 SQLite 数据。
- schema migration 要可重复执行。
- 新表用 `CREATE TABLE IF NOT EXISTS`。
- 新字段要考虑默认值。
- 对已有重复课程只提醒，不自动删除。
- 做导入功能时优先预览，再写入。
- 大改数据库前建议备份：

```text
C:\Users\Administrator\AppData\Roaming\cn.local.momolite\momolite.sqlite
```

## 20. AI 协作开发方法

用户后续和新 AI 对话时，建议这样提需求：

```text
请先阅读：
C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite\docs\MomoLite-product-design-and-handoff-zh.md

你是 20 年经验产品经理和 20 年全栈工程师，同时要像老师一样带我开发。
请不要直接大改，先用 5 到 10 分钟摸清代码结构，然后按小阶段实现。
每个阶段都要解释：
1. 为什么这么设计
2. 要改哪些文件
3. 数据结构怎么变化
4. 怎么验证
5. 我作为小白应该学到什么

当前目标是实现 v0.3：词汇导入与基础词汇库。
请先让我提供一个真实背单词 app 导出样例，然后基于样例设计 parser、数据库和 UI。
```

给 AI 的好需求格式：

```text
背景：
我在做 MomoLite，本地 Windows 桌面英语学习软件。

目标：
这次只实现词汇导入，不做 LLM 生成。

输入样例：
粘贴 10 到 20 行真实导出数据。

约束：
不能破坏已有课程、课时、句子数据。
必须支持 Tauri 桌面运行。
先预览再写数据库。

验收：
npm build 通过。
Rust 测试通过。
桌面软件能看到导入的词汇批次。
```

## 21. 新对话的首要任务

新的 AI 对话应该先做这几步：

1. 读取本文档。
2. 读取项目代码结构。
3. 确认当前 git 状态。
4. 让用户提供真实背单词 app 导出样例。
5. 基于样例设计 v0.3 的词汇导入。
6. 先做数据库 migration 和后端命令。
7. 再做前端导入预览 UI。
8. 最后做基础词汇批次详情和单词卡片。

不要优先做：

- 大规模视觉重构。
- 复杂 FSRS。
- 多用户系统。
- 云同步。
- 课程市场。
- 复杂社交功能。

## 22. 给新对话的完整交接提示词

可以把下面这段直接复制给新的 AI：

```text
你现在接手 MomoLite 项目。

请你扮演 20 年经验产品经理、20 年全栈开发高级工程师，同时像老师一样带一个实习生小白开发。

项目路径：
C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite

请先阅读交接文档：
C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite\docs\MomoLite-product-design-and-handoff-zh.md

当前产品：
MomoLite 是本地 Windows 桌面英语学习软件，技术栈是 Tauri + React + TypeScript + Rust + SQLite + Piper TTS。

当前已实现：
1. 课程包、课时、句子导入。
2. 首页 / 课程包 / 课程详情 / 学习页 / 统计页。
3. 课程详情可点进课时看句子，并从课时开始学习。
4. 中译英训练：听英文、看中文、填英文词、空格跳词、方向键切词、错词重填、答对后显示完整答案再进入下一句。
5. Piper 离线发音。
6. Web preview 下 Tauri invoke 异常已做友好提示。
7. SQLite ambiguous column id 问题已修复。

当前数据库：
C:\Users\Administrator\AppData\Roaming\cn.local.momolite\momolite.sqlite
最近观察到 8 个课程、52 个课时、444 条句子、1 条学习记录。

用户新愿景：
从背单词 app 导出当天或近期背过的词，导入 MomoLite，然后用 LLM 生成高质量母语者日常场景对话，再形成沉浸式复习页和可训练课程。训练模式包括中译英、听力听写、跟读、错词错句复习。目标是增强口语、听力和阅读，让英语逐渐成为第二语言。

重要 skill：
C:\Users\Administrator\.codex\skills\spoken-english-builder\SKILL.md
它定义了从词汇生成自然日常对话、沉浸式复习 Markdown、听写导入流和覆盖率检查的规则。

下一阶段目标：
实现 v0.3：词汇导入与基础词汇库。

请先不要写代码，先快速摸清项目结构并让我提供一个真实背单词 app 导出样例。拿到样例后，再设计：
1. parser
2. SQLite 新表
3. Tauri commands
4. 前端导入预览
5. 词汇批次详情
6. 基础单词卡片复习

每一步都要边做边教我：
为什么这么做、改了哪些文件、如何验证、我应该学到什么开发思维。

验证命令：
npm.cmd run build
npm.cmd run test:answer-rules
$env:Path = 'C:\Users\Administrator\.cargo\bin;' + $env:Path; cargo test --lib -- --nocapture
$env:Path = 'C:\Users\Administrator\.cargo\bin;' + $env:Path; npm.cmd run tauri -- build

注意：
这是桌面软件，不是网页产品。不要破坏已有 SQLite 数据。大改前先确认 git 状态，完成小阶段后提交。
```

## 23. 最重要的产品判断

MomoLite 后续不要变成“功能很多但不知道每天干什么”的大杂烩。

它应该围绕一个极简但强大的主线：

```text
今天背过的词
-> 变成真实对话
-> 看懂
-> 听懂
-> 打出来
-> 说出来
-> 过几天还能想起来
```

只要这条主线做得足够顺，软件就有真实价值。
