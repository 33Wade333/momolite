# MomoLite Product And Engineering Handoff

Updated: 2026-04-27

## 1. One-Line Product Direction

MomoLite is a local Windows desktop English learning app that turns vocabulary the user has memorized into high-quality native daily-scene dialogues, then converts those dialogues into immersive review pages, listening dictation, Chinese-to-English active recall, and long-term review.

The product should not become a bloated English platform. Its core promise is:

> Make words the user has memorized become English they can hear, read, type, and eventually say naturally.

## 2. Current Project State

Project path:

`C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite`

Desktop executable:

`C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite\src-tauri\target\release\momolite.exe`

Installer:

`C:\Users\Administrator\Documents\Codex\2026-04-26\new-chat\momolite\src-tauri\target\release\bundle\nsis\momolite_0.1.0_x64-setup.exe`

SQLite database:

`C:\Users\Administrator\AppData\Roaming\cn.local.momolite\momolite.sqlite`

Current database counts observed:

- `course_packs`: 8
- `lessons`: 52
- `sentence_items`: 444
- `review_logs`: 1
- `daily_stats`: 1

Current tech stack:

- Desktop: Tauri 2
- Frontend: React 19 + TypeScript + Vite
- UI icons: `lucide-react`
- Backend: Rust
- Database: SQLite via `rusqlite`
- Offline TTS: Piper, `en_US-lessac-medium`

## 3. Important Git History

- `bffee9b` baseline: current MomoLite MVP
- `3570d21` refactor: productize MomoLite study experience
- `c5a42e9` fix: guard Tauri calls outside desktop runtime
- `9851589` fix: qualify review log query columns
- `1ff0f95` feat: improve lesson study flow

The working tree was clean after the latest implementation.

## 4. Implemented Features

### Course And Lesson Management

- Course packages can be created.
- Imported text can be split into lessons using headings such as `## 第一天` or `## 第1课 经济讨论`.
- A course contains lessons, and lessons contain sentence items.
- Course detail now supports clicking a lesson card to view only that lesson's sentences.
- A selected lesson can be studied directly via "学习本课".

### Import

Supported import formats:

```text
## 第一天
Yeah, the economy is developing faster than most people expect.=是的，经济发展比大多数人预期的要快。

## 第二天
Hey, did you receive my text yesterday?=嘿，你收到我昨天的短信了吗？
```

Also supports:

```text
English=中文=音标
```

Frontend import deduplicates by `english + chinese` within the active course and reports added/skipped counts.

### Study Flow

Current Chinese-to-English flow:

- Enter study page.
- English audio plays automatically.
- Chinese prompt is shown.
- English punctuation is fixed and displayed.
- User fills only word blanks.
- Space moves to the next editable word.
- Left/right arrow keys move between word blanks.
- On wrong answer: wrong word blanks are marked red; correct words are locked.
- On correct answer: the full English answer is shown first; user clicks "下一句" to record and continue.
- Study queue is available but collapsed by default.

### Desktop Runtime

- The app is intended to be used as a Tauri desktop app, not a web page.
- Web/Vite preview cannot access SQLite/Tauri `invoke`.
- A guard was added so web preview shows a friendly message instead of `Cannot read properties of undefined (reading 'invoke')`.

### Offline TTS

- Rust backend calls Piper.
- Generated WAV is cached.
- If Piper fails, frontend falls back to system speech synthesis.

## 5. Database Design Summary

Existing core tables:

- `course_packs`
- `lessons`
- `sentence_items`
- `review_logs`
- `daily_stats`

Important principles:

- Sentence belongs to lesson.
- Lesson belongs to course.
- Review belongs to sentence.
- Foreign keys prevent orphan records.
- Course deletion cascades to lessons, sentences, and reviews.
- SQL queries that join multiple tables must qualify columns with aliases, e.g. `r.id`, `s.id`, `l.id`.

## 6. Validation Commands

Use these after meaningful changes:

```powershell
npm.cmd run build
npm.cmd run test:answer-rules
$env:Path = 'C:\Users\Administrator\.cargo\bin;' + $env:Path; cargo test --lib -- --nocapture
$env:Path = 'C:\Users\Administrator\.cargo\bin;' + $env:Path; npm.cmd run tauri -- build
```

Rust/Cargo was installed via rustup and is available at:

`C:\Users\Administrator\.cargo\bin`

## 7. Current Product Vision

The user wants a novel pattern:

> Vocabulary memorization -> high-quality native-scene dialogue -> immersive review -> active training.

This is different from ordinary vocabulary apps. It uses vocabulary as raw material for real language scenes.

The ideal learning loop:

1. Import words memorized today or recently from a vocabulary app.
2. Generate complete daily-life scenes using those words.
3. Read an immersive bilingual review page with target words highlighted.
4. Listen to native-like audio.
5. Do Chinese-to-English active recall.
6. Do listening dictation with no Chinese prompt.
7. Optionally do follow-and-repeat practice.
8. Wrong words and wrong sentences enter review queues.

## 8. Product Modules To Build

### A. Vocabulary Module

Purpose: bring "背单词" into MomoLite.

Recommended features:

- Import vocabulary exported from a vocabulary app.
- Store batches: today's words, recent words, custom sets.
- Store vocabulary items: word/phrase, Chinese meaning, source, familiarity, last reviewed date.
- Basic word card review inspired by memory apps:
  - forgot
  - vague
  - know
  - familiar
- Do not overbuild at first. Vocabulary exists to feed scene generation.

Future tables:

- `vocabulary_batches`
- `vocabulary_items`
- `vocabulary_reviews`

### B. Scene Dialogue Generation Module

Purpose: convert vocabulary into high-quality native daily dialogues.

Important rule change:

- Do not rigidly limit one scene to 6 words or 6 sentences.
- A scene may contain 6-12 target words or more.
- Quality and naturalness are more important than fixed count.
- A scene should be a complete situation, not a list of example sentences.

Scene generation should support:

- default style: daily life
- optional styles: work, travel, social, study, family, emotions, shopping, food, health
- difficulty: A2/B1/B2/C1
- output: title, scene description, dialogue lines, target word coverage, key expressions, Chinese translation

Future tables:

- `generated_scenes`
- `scene_vocabulary_links`
- `scene_lines`

### C. Immersive Review Module

Purpose: recreate the "Feishu immersive review" feeling inside the app.

The skill `spoken-english-builder` already defines a useful format:

- vocabulary index table
- story/scene title
- core vocabulary list
- scene description
- bilingual dialogue
- target words highlighted
- key expressions
- story separators
- coverage report

MomoLite should render this as a beautiful in-app reading page, not just raw Markdown.

Future table:

- `review_documents`

### D. Training Module

Existing:

- Chinese-to-English sentence fill-in.

Add:

- listening dictation: no Chinese prompt, only audio, user types English.
- selected lesson training.
- wrong sentence training.
- generated-scene training.
- follow-and-repeat mode, initially manual confirmation; speech scoring can come later.

### E. LLM Settings Module

LLM is essential for scene quality.

Since this is a local desktop app, do not hard-code private keys.

Recommended configuration UI:

- provider preset
- custom OpenAI-compatible base URL
- API key
- model name
- test connection
- default style
- default difficulty

Provider strategy:

- support OpenAI-compatible APIs first
- allow user-owned API keys
- allow local model endpoints later, such as Ollama-compatible usage
- "free online APIs" can be documented as examples, but should not be hard-coded because availability changes

Future table:

- `app_settings` or a local settings JSON file

## 9. Next Development Plan

### v0.3 Vocabulary Import And Basic Word Review

Goal: establish the source material pipeline.

Build:

- Vocabulary import page.
- Parser for exported word app text/CSV.
- Preview recognized words and meanings.
- Create `vocabulary_batches` and `vocabulary_items`.
- Basic word review card with familiarity feedback.

Need from user:

- A real export sample from their vocabulary app, at least 10-20 rows.

### v0.4 LLM Settings And Scene Preview

Goal: generate one high-quality scene from a vocabulary batch.

Build:

- LLM settings page.
- Connection test.
- Generate scene preview.
- Show target word coverage.
- Regenerate scene button.
- Save generated scene.

### v0.5 Scene-To-Course

Goal: turn generated scene into lessons and trainable sentences.

Build:

- Convert scene lines into lessons and sentence_items.
- Create a course from a vocabulary batch.
- Link scene lines to target vocabulary.
- Add "study this scene" entry.

### v0.6 Immersive Review Page

Goal: let users browse and absorb before drilling.

Build:

- Render vocabulary index.
- Render bilingual dialogue.
- Highlight target words.
- Show key expressions.
- Add "start dictation" and "start Chinese-to-English" buttons from the review page.

### v0.7 Listening Dictation

Goal: strengthen listening and spelling without Chinese prompt.

Build:

- Audio-only prompt.
- Type full sentence or word blanks.
- Replay controls.
- Compare answer with existing answer rules.
- Log dictation results separately or extend review logs with mode.

## 10. Key Product Principle

High-quality dialogue generation is the hard part.

Define "high quality" as:

- the scene feels like a real daily conversation
- target words are used naturally, not forced
- dialogue has context, intention, and emotional flow
- sentences are short enough for listening and speaking practice
- Chinese translation is natural
- target word coverage is complete
- useful collocations and spoken chunks are extracted

Do not build many scattered features before this pipeline works.

## 11. How The User Wants To Learn Development

The user is a beginner/intern-style learner and wants to learn how to build a real product with AI.

Recommended collaboration style:

- Explain the product/engineering decision before implementing.
- Make small, testable iterations.
- Keep Git commits after each meaningful milestone.
- Always verify with build/tests/package when possible.
- Teach the thinking pattern:

```text
User pain -> product behavior -> data model -> UI flow -> implementation -> tests -> packaged app
```

The user responds well to concrete, product-minded explanations, not abstract lectures.

## 12. Immediate Next Ask For New Conversation

Ask the user to provide a sample export from their vocabulary app.

Minimum useful sample:

- 10-20 lines
- include word/phrase
- include Chinese meaning if available
- include memorized date or familiarity if available

Then implement v0.3:

Vocabulary import -> preview -> database schema -> basic vocabulary batch page.

