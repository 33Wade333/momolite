import { invoke } from "@tauri-apps/api/core";
import { appDataDir } from "@tauri-apps/api/path";
import { Stronghold, type Client } from "@tauri-apps/plugin-stronghold";
import {
  FormEvent,
  KeyboardEvent as ReactKeyboardEvent,
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  ArrowLeft,
  BarChart3,
  BookOpen,
  CheckCircle2,
  ChevronRight,
  Flame,
  Home,
  Import,
  LibraryBig,
  ListChecks,
  Menu,
  Play,
  Plus,
  RefreshCw,
  Sparkles,
  Target,
  Trash2,
  Trophy,
  Volume2,
  X,
  type LucideIcon,
} from "lucide-react";
import {
  buildAnswerFromWords,
  getAnswerWords,
  getWrongWordIndexes,
  parseAnswerParts,
} from "./answerRules";
import "./App.css";

type View =
  | "home"
  | "vocabulary"
  | "wordStudy"
  | "scenes"
  | "courses"
  | "courseDetail"
  | "study"
  | "stats"
  | "settings";
type SentenceStatus = "new" | "learning" | "mastered";
type Rating = "again" | "hard" | "good" | "easy";
type AnswerResult = "idle" | "correct" | "wrong";

interface CoursePack {
  id: string;
  name: string;
  language: string;
  dailyNewTarget: number;
  createdAt: string;
}

interface SentenceItem {
  id: string;
  coursePackId: string;
  lessonTitle: string;
  english: string;
  chinese: string;
  phonetic: string;
  note: string;
  status: SentenceStatus;
  favorite: boolean;
  showCount: number;
  reviewCount: number;
  errorCount: number;
  nextReviewAt: string | null;
  createdAt: string;
}

interface ReviewLog {
  id: string;
  sentenceId: string;
  coursePackId: string;
  userAnswer: string;
  isCorrect: boolean;
  wrongIndexes?: number[];
  rating: Rating;
  reviewedAt: string;
  intervalMinutes: number;
}

interface ReviewSubmission {
  id: string;
  sentenceId: string;
  userAnswer: string;
  isCorrect: boolean;
  wrongIndexes?: number[];
  rating: Rating;
  reviewedAt: string;
  intervalMinutes: number;
  nextReviewAt: string | null;
}

interface DailyStats {
  newCount: number;
  reviewCount: number;
  studyMinutes: number;
}

interface VocabularyBook {
  id: string;
  name: string;
  source: string;
  importedAt: string;
  note: string;
  itemCount: number;
}

interface VocabularyItem {
  id: string;
  text: string;
  normalizedText: string;
  primaryMeaning: string;
  phonetic: string;
  partOfSpeech: string;
  example: string;
  exampleCn: string;
  roots: string;
  wordFamily: string;
  synonyms: string;
  antonyms: string;
  memoryHint: string;
  tags: string;
  difficulty: string;
  familiarity: number;
  weakScore: number;
  reviewCount: number;
  wrongCount: number;
  lastReviewedAt: string | null;
  nextReviewAt: string | null;
  createdAt: string;
  updatedAt: string;
}

interface VocabularyBookDetail {
  book: VocabularyBook;
  items: VocabularyItem[];
}

interface ImportVocabularyBookResult {
  book: VocabularyBook;
  items: VocabularyItem[];
  importedCount: number;
  reusedCount: number;
  skippedCount: number;
}

interface AppState {
  activeCoursePackId: string;
  coursePacks: CoursePack[];
  sentences: SentenceItem[];
  reviews: ReviewLog[];
  stats: Record<string, DailyStats>;
}

interface ImportRow {
  lessonTitle: string;
  english: string;
  chinese: string;
  phonetic: string;
  note: string;
}

interface VocabularyImportPreviewItem {
  text: string;
  primaryMeaning: string;
  phonetic: string;
  partOfSpeech: string;
  example: string;
  exampleCn: string;
  roots: string;
  wordFamily: string;
  synonyms: string;
  antonyms: string;
  memoryHint: string;
  tags: string;
  difficulty: string;
}

interface NewLesson {
  id: string;
  coursePackId: string;
  title: string;
  sortOrder: number;
  createdAt: string;
}

interface TtsResponse {
  audioBase64: string;
  cached: boolean;
  engine: string;
}

interface SyncSettings {
  provider: string;
  baseUrl: string;
  username: string;
  remotePath: string;
  deviceId: string;
  autoSyncEnabled: boolean;
  lastSyncAt: string | null;
  syncStatus: string;
  lastError: string;
  updatedAt: string;
}

interface SyncActionResult {
  ok: boolean;
  message: string;
  remoteSnapshotUrl: string;
  uploadedBytes: number;
  mergedRows: number;
  syncedAt: string;
}

interface LlmSettings {
  provider: string;
  baseUrl: string;
  model: string;
  wireApi: "chat_completions" | "responses";
  reasoningEffort: "minimal" | "low" | "medium" | "high";
  disableResponseStorage: boolean;
  promptVersion: string;
  temperature: number;
  updatedAt: string;
}

interface LlmProfile extends LlmSettings {
  id: string;
  name: string;
  isDefault: boolean;
  createdAt: string;
}

interface EnrichedVocabularyEntry extends VocabularyImportPreviewItem {}

interface EnrichVocabularyResponse {
  entries: EnrichedVocabularyEntry[];
  requestJson: string;
  responseJson: string;
}

interface GeneratedScene {
  id: string;
  title: string;
  scenario: string;
  promptVersion: string;
  model: string;
  status: "generating" | "succeeded" | "failed";
  targetWordsSnapshot: string;
  requestJson: string;
  responseJson: string;
  errorMessage: string;
  batchId: string | null;
  plannedCoursePackId: string | null;
  lessonId: string | null;
  coverageJson: string;
  isAddedToCourse: boolean;
  createdAt: string;
  updatedAt: string;
}

interface SceneLine {
  id: string;
  sceneId: string;
  sortOrder: number;
  speaker: string;
  english: string;
  chinese: string;
}

interface GeneratedSceneDetail {
  scene: GeneratedScene;
  lines: SceneLine[];
}

interface SceneTargetWord {
  id: string;
  text: string;
  meaning: string;
  difficulty: string;
  weakScore: number;
}

interface SceneCoverageSummary {
  usedCoreWords: string[];
  missingCoreWords: string[];
  coreCoverageRate: number | null;
}

interface SceneGenerationBatch {
  id: string;
  title: string;
  coursePackId: string | null;
  selectedTopics: string;
  wordSource: string;
  coreWordCount: number;
  supportWordCount: number;
  plannedSceneCount: number;
  status: string;
  coverageSummary: string;
  errorMessage: string;
  createdAt: string;
  updatedAt: string;
}

interface SceneGenerationPlan {
  id: string;
  batchId: string;
  sortOrder: number;
  title: string;
  topic: string;
  coreWordIds: string;
  supportWordIds: string;
  coreWordsSnapshot: string;
  supportWordsSnapshot: string;
  status: string;
  generatedSceneId: string | null;
  errorMessage: string;
  createdAt: string;
  updatedAt: string;
}

interface SceneBatchPlanResponse {
  batch: SceneGenerationBatch;
  plans: SceneGenerationPlan[];
}

interface AddScenesToCourseResponse {
  createdLessons: number;
  createdSentences: number;
}

interface LearningPlanSettings {
  intensity: "light" | "standard" | "intensive";
  desiredRetention: number;
  dailyNewTarget: number;
  dailyReviewLimit: number;
  sceneLessonsTarget: number;
  recoveryDays: number;
  updatedAt: string;
}

interface DailyLearningPlan {
  id: string;
  planDate: string;
  intensity: string;
  newWordTarget: number;
  reviewLimit: number;
  sceneLessonTarget: number;
  dueCount: number;
  weakCount: number;
  backlogCount: number;
  planJson: string;
  explanation: string;
  createdAt: string;
  updatedAt: string;
}

interface CourseSummary {
  course: CoursePack;
  lessonCount: number;
  totalSentences: number;
  dueCount: number;
  masteredCount: number;
  progressPercent: number;
  duplicateCount: number;
  lastReviewedAt: string | null;
}

interface LessonSummary {
  title: string;
  index: number;
  total: number;
  mastered: number;
  due: number;
  progressPercent: number;
}

interface StudyQueueItem {
  id: string;
  lessonTitle: string;
  english: string;
  chinese: string;
  status: SentenceStatus;
  isCurrent: boolean;
}

interface StudyBookmark {
  coursePackId: string;
  sentenceId: string | null;
  savedAt: string;
}

const DEFAULT_LESSON = "默认课时";
const MANUAL_LESSON = "手动添加";
const LAST_STUDY_KEY = "momolite:last-study";
const WORD_BOOK_SAMPLE = `# CET4 核心词汇 Week 1

## abandon

- 中文释义：放弃；抛弃
- 音标：/əˈbændən/
- 词性：verb
- 例句：She had to abandon the plan because of the weather.
- 例句中文：因为天气原因，她不得不放弃这个计划。
- 词根词缀：a- 表示离开；bandon 表示控制、命令
- 同根词：abandoned, abandonment
- 近义词：give up, quit, desert
- 反义词：keep, continue, maintain
- 形象记忆：把一个计划丢在路边，不再带着它往前走。
- 场景标签：study, work, decision
- 难度：B1`;

const viewCopy: Record<View, { title: string; subtitle: string }> = {
  home: { title: "首页", subtitle: "今天继续一小步，英语句子更顺一点。" },
  vocabulary: { title: "单词书", subtitle: "管理词书和词条；背词时进入专注模式。" },
  wordStudy: { title: "背单词", subtitle: "专注复习当前队列。" },
  scenes: { title: "AI 场景", subtitle: "先规划场景课，再生成可阅读、可入课的草稿。" },
  courses: { title: "课程包", subtitle: "整理你的句子材料和训练路径。" },
  courseDetail: { title: "课程详情", subtitle: "课时、导入和继续学习都在这里。" },
  study: { title: "中译英训练", subtitle: "看中文，听英文，补全原句。" },
  stats: { title: "统计", subtitle: "查看课程进度和最近学习表现。" },
  settings: { title: "同步设置", subtitle: "配置坚果云 WebDAV、LLM API Key 和跨端同步。" },
};

const navItems: Array<{ view: View; label: string; icon: LucideIcon }> = [
  { view: "home", label: "首页", icon: Home },
  { view: "vocabulary", label: "单词书", icon: BookOpen },
  { view: "scenes", label: "AI 场景", icon: Sparkles },
  { view: "courses", label: "课程包", icon: LibraryBig },
  { view: "stats", label: "统计", icon: BarChart3 },
  { view: "settings", label: "同步设置", icon: Menu },
];

const ratingRules: Record<Rating, { minutes: number; status: SentenceStatus }> = {
  again: { minutes: 10, status: "learning" },
  hard: { minutes: 1440, status: "learning" },
  good: { minutes: 4320, status: "mastered" },
  easy: { minutes: 10080, status: "mastered" },
};

const vocabularyRatingRules: Record<
  Rating,
  { label: string; minutes: number; tone: string }
> = {
  again: { label: "忘了", minutes: 10, tone: "danger" },
  hard: { label: "模糊", minutes: 1440, tone: "warn" },
  good: { label: "认识", minutes: 4320, tone: "primary" },
  easy: { label: "熟悉", minutes: 10080, tone: "success" },
};

const initialState: AppState = {
  activeCoursePackId: "",
  coursePacks: [],
  sentences: [],
  reviews: [],
  stats: {},
};

const defaultSyncSettings: SyncSettings = {
  provider: "jianguoyun_webdav",
  baseUrl: "https://dav.jianguoyun.com/dav/",
  username: "",
  remotePath: "/MomoLite/sync/",
  deviceId: "",
  autoSyncEnabled: false,
  lastSyncAt: null,
  syncStatus: "idle",
  lastError: "",
  updatedAt: "",
};

const defaultLlmSettings: LlmSettings = {
  provider: "gpt2",
  baseUrl: "https://way.ydata.vip/v1",
  model: "gpt-5.4",
  wireApi: "responses",
  reasoningEffort: "high",
  disableResponseStorage: true,
  promptVersion: "momolite-scene-v1",
  temperature: 0.4,
  updatedAt: "",
};

const defaultLlmProfile: LlmProfile = {
  id: "default",
  name: "我的中转站 GPT-5.4",
  ...defaultLlmSettings,
  isDefault: true,
  createdAt: "",
};

const defaultLearningPlanSettings: LearningPlanSettings = {
  intensity: "standard",
  desiredRetention: 0.9,
  dailyNewTarget: 25,
  dailyReviewLimit: 160,
  sceneLessonsTarget: 3,
  recoveryDays: 3,
  updatedAt: "",
};

const defaultDailyLearningPlan: DailyLearningPlan = {
  id: "",
  planDate: "",
  intensity: "standard",
  newWordTarget: 25,
  reviewLimit: 160,
  sceneLessonTarget: 3,
  dueCount: 0,
  weakCount: 0,
  backlogCount: 0,
  planJson: "{}",
  explanation: "今天按标准强度推进：先背新词，再清理到期词，最后生成场景课。",
  createdAt: "",
  updatedAt: "",
};

const sceneTopicOptions = [
  "commute and errands",
  "work and project discussion",
  "study and class",
  "restaurant or cafe",
  "shopping and payment",
  "friends making plans",
  "family daily talk",
  "health and doctor",
  "apartment repair",
  "planning and decisions",
  "conflict and apology",
  "interview and networking",
  "special daily scenario",
];

const llmProviderPresets: Record<
  string,
  {
    label: string;
    baseUrl: string;
    model: string;
    wireApi: LlmSettings["wireApi"];
    reasoningEffort: LlmSettings["reasoningEffort"];
    disableResponseStorage: boolean;
    temperature: number;
  }
> = {
  gpt2: {
    label: "我的 gpt2 中转站（Responses）",
    baseUrl: "https://way.ydata.vip/v1",
    model: "gpt-5.4",
    wireApi: "responses",
    reasoningEffort: "high",
    disableResponseStorage: true,
    temperature: 0.4,
  },
  volcengine_ark: {
    label: "火山方舟 / 豆包（填 ep 接入点）",
    baseUrl: "https://ark.cn-beijing.volces.com/api/v3/chat/completions",
    model: "",
    wireApi: "chat_completions",
    reasoningEffort: "medium",
    disableResponseStorage: false,
    temperature: 0.4,
  },
  openai_compatible: {
    label: "OpenAI-compatible",
    baseUrl: "https://api.openai.com/v1/chat/completions",
    model: "gpt-4.1-mini",
    wireApi: "chat_completions",
    reasoningEffort: "medium",
    disableResponseStorage: false,
    temperature: 0.4,
  },
};

function hasTauriBridge() {
  return (
    typeof window !== "undefined" &&
    "__TAURI_INTERNALS__" in window &&
    Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)
  );
}

function safeInvoke<T>(command: string, args?: Record<string, unknown>) {
  if (!hasTauriBridge()) {
    return Promise.reject(
      new Error("当前是网页预览环境，请用 MomoLite 桌面软件打开以连接 SQLite 和离线朗读。"),
    );
  }
  return invoke<T>(command, args);
}

function joinAppPath(base: string, filename: string) {
  const separator = base.endsWith("/") || base.endsWith("\\") ? "" : "\\";
  return `${base}${separator}${filename}`;
}

async function openSecretStore() {
  const vaultKey = await safeInvoke<string>("get_secret_vault_key");
  const vaultPath = joinAppPath(await appDataDir(), "momolite-secrets.hold");
  const stronghold = await Stronghold.load(vaultPath, vaultKey);
  let client: Client;
  try {
    client = await stronghold.loadClient("momolite");
  } catch {
    client = await stronghold.createClient("momolite");
  }
  return { stronghold, store: client.getStore() };
}

async function readSecret(key: string) {
  const { store } = await openSecretStore();
  const value = await store.get(key);
  if (!value) return "";
  return new TextDecoder().decode(value);
}

async function writeSecret(key: string, value: string) {
  const { stronghold, store } = await openSecretStore();
  await store.insert(key, Array.from(new TextEncoder().encode(value)));
  await stronghold.save();
}

async function removeSecret(key: string) {
  const { stronghold, store } = await openSecretStore();
  await store.remove(key);
  await stronghold.save();
}

function llmSecretKey(profileId: string) {
  return `llm-api-key:${profileId || "default"}`;
}

function uid(prefix: string) {
  return `${prefix}_${Date.now()}_${Math.random().toString(16).slice(2)}`;
}

function todayKey() {
  const now = new Date();
  const month = `${now.getMonth() + 1}`.padStart(2, "0");
  const day = `${now.getDate()}`.padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
}

function splitCsvLine(line: string) {
  const parts: string[] = [];
  let current = "";
  let inQuote = false;

  for (const char of line) {
    if (char === '"') {
      inQuote = !inQuote;
    } else if (char === "," && !inQuote) {
      parts.push(current);
      current = "";
    } else {
      current += char;
    }
  }

  parts.push(current);
  return parts;
}

function isLessonHeading(line: string) {
  const normalized = line.trim();
  return (
    /^#{1,6}\s+/.test(normalized) ||
    /^\[.+\]$/.test(normalized) ||
    /^第\s*[\d一二三四五六七八九十百千万]+\s*[课天日]/.test(normalized) ||
    /^(lesson|story|day|unit|section)\s*\d*/i.test(normalized)
  );
}

function cleanLessonTitle(line: string) {
  return line
    .trim()
    .replace(/^#{1,6}\s+/, "")
    .replace(/^\[/, "")
    .replace(/\]$/, "")
    .trim();
}

function parseImport(text: string): ImportRow[] {
  const lines = text
    .replace(/\u200b/g, "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  let currentLesson = DEFAULT_LESSON;
  const rows: ImportRow[] = [];
  const hasCsvHeader = /^word\s*,\s*translation/i.test(lines[0] ?? "");
  const source = hasCsvHeader ? lines.slice(1) : lines;

  for (const rawLine of source) {
    if (/^\d+$/.test(rawLine) || /^-{3,}$/.test(rawLine)) {
      continue;
    }

    if (isLessonHeading(rawLine)) {
      currentLesson = cleanLessonTitle(rawLine);
      continue;
    }

    if (rawLine.includes("=")) {
      const [englishPart, chinesePart = "", phoneticPart = "", ...rest] =
        rawLine.split("=");
      const english = englishPart.trim();
      const chinese = chinesePart.trim();
      const looksLikeCourseTitle = /^(story|lesson|day|unit|section)\s*\d*/i.test(
        english,
      );

      if (looksLikeCourseTitle && chinese) {
        currentLesson = `${english} / ${chinese}`;
        continue;
      }

      if (english && chinese) {
        rows.push({
          lessonTitle: currentLesson,
          english,
          chinese,
          phonetic: phoneticPart.trim(),
          note: rest.join("=").trim(),
        });
      }
      continue;
    }

    const [english, chinese, phonetic = "", note = ""] = splitCsvLine(rawLine);
    if (english?.trim() && chinese?.trim()) {
      rows.push({
        lessonTitle: currentLesson,
        english: english.trim(),
        chinese: chinese.trim(),
        phonetic: phonetic.trim(),
        note: note.trim(),
      });
    }
  }

  return rows;
}

function emptyVocabularyPreviewItem(text: string): VocabularyImportPreviewItem {
  return {
    text,
    primaryMeaning: "",
    phonetic: "",
    partOfSpeech: "",
    example: "",
    exampleCn: "",
    roots: "",
    wordFamily: "",
    synonyms: "",
    antonyms: "",
    memoryHint: "",
    tags: "",
    difficulty: "",
  };
}

function applyVocabularyPreviewField(
  item: VocabularyImportPreviewItem,
  key: string,
  value: string,
) {
  const trimmedKey = key.trim();
  if (["中文释义", "释义", "中文", "意思"].includes(trimmedKey)) {
    item.primaryMeaning = value;
  } else if (["音标", "发音"].includes(trimmedKey)) {
    item.phonetic = value;
  } else if (trimmedKey === "词性") {
    item.partOfSpeech = value;
  } else if (["例句", "英文例句"].includes(trimmedKey)) {
    item.example = value;
  } else if (["例句中文", "中文例句", "例句翻译"].includes(trimmedKey)) {
    item.exampleCn = value;
  } else if (["词根词缀", "词根", "词缀"].includes(trimmedKey)) {
    item.roots = value;
  } else if (["同根词", "词族", "派生词"].includes(trimmedKey)) {
    item.wordFamily = value;
  } else if (["近义词", "同义词"].includes(trimmedKey)) {
    item.synonyms = value;
  } else if (trimmedKey === "反义词") {
    item.antonyms = value;
  } else if (["形象记忆", "记忆法", "记忆提示"].includes(trimmedKey)) {
    item.memoryHint = value;
  } else if (["场景标签", "标签", "场景"].includes(trimmedKey)) {
    item.tags = value;
  } else if (trimmedKey === "难度") {
    item.difficulty = value;
  }
}

function parsePlainWordList(text: string) {
  return text
    .split(/[\n,，;；]+/)
    .map((word) => word.trim())
    .filter(Boolean);
}

function vocabularyEntriesToMarkdown(title: string, entries: VocabularyImportPreviewItem[]) {
  const lines = [`# ${title || "LLM 补全词书"}`, ""];
  for (const entry of entries) {
    lines.push(`## ${entry.text}`, "");
    lines.push(`- 中文释义：${entry.primaryMeaning}`);
    if (entry.phonetic) lines.push(`- 音标：${entry.phonetic}`);
    if (entry.partOfSpeech) lines.push(`- 词性：${entry.partOfSpeech}`);
    if (entry.example) lines.push(`- 例句：${entry.example}`);
    if (entry.exampleCn) lines.push(`- 例句中文：${entry.exampleCn}`);
    if (entry.roots) lines.push(`- 词根词缀：${entry.roots}`);
    if (entry.wordFamily) lines.push(`- 同根词：${entry.wordFamily}`);
    if (entry.synonyms) lines.push(`- 近义词：${entry.synonyms}`);
    if (entry.antonyms) lines.push(`- 反义词：${entry.antonyms}`);
    if (entry.memoryHint) lines.push(`- 形象记忆：${entry.memoryHint}`);
    if (entry.tags) lines.push(`- 场景标签：${entry.tags}`);
    if (entry.difficulty) lines.push(`- 难度：${entry.difficulty}`);
    lines.push("");
  }
  return lines.join("\n");
}

function jsonArrayCount(value: string) {
  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? parsed.length : 0;
  } catch {
    return 0;
  }
}

function parseSceneTargetWords(value: string): SceneTargetWord[] {
  try {
    const parsed = JSON.parse(value);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .map((item) => ({
        id: String(item.id ?? ""),
        text: String(item.text ?? ""),
        meaning: String(item.meaning ?? item.primaryMeaning ?? ""),
        difficulty: String(item.difficulty ?? ""),
        weakScore: Number(item.weak_score ?? item.weakScore ?? 0),
      }))
      .filter((item) => item.text);
  } catch {
    return [];
  }
}

function parseSceneCoverage(value: string): SceneCoverageSummary {
  try {
    const parsed = JSON.parse(value);
    return {
      usedCoreWords: Array.isArray(parsed.usedCoreWords) ? parsed.usedCoreWords : [],
      missingCoreWords: Array.isArray(parsed.missingCoreWords)
        ? parsed.missingCoreWords
        : [],
      coreCoverageRate:
        typeof parsed.coreCoverageRate === "number" ? parsed.coreCoverageRate : null,
    };
  } catch {
    return {
      usedCoreWords: [],
      missingCoreWords: [],
      coreCoverageRate: null,
    };
  }
}

function formatCoverageRate(rate: number | null) {
  return rate === null ? "未计算" : `${Math.round(rate * 100)}%`;
}

function escapeRegex(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function highlightTargetWords(text: string, targets: SceneTargetWord[]) {
  const patterns = targets
    .map((target) => target.text.trim())
    .filter((target) => target.length > 1)
    .sort((a, b) => b.length - a.length)
    .map(escapeRegex);

  if (!patterns.length) return text;

  const matcher = new RegExp(`\\b(${patterns.join("|")})\\b`, "gi");
  return text.split(matcher).map((part, index) =>
    index % 2 === 1 ? (
      <mark className="target-highlight" key={`${part}-${index}`}>
        {part}
      </mark>
    ) : (
      part
    ),
  );
}

function parseVocabularyBookPreview(text: string) {
  const lines = text.replace(/\u200b/g, "").split(/\r?\n/);
  let title = "";
  let current: VocabularyImportPreviewItem | null = null;
  const items: VocabularyImportPreviewItem[] = [];
  let skipped = 0;

  function commitCurrent() {
    if (!current) return;
    if (current.text.trim() && current.primaryMeaning.trim()) {
      items.push(current);
    } else {
      skipped += 1;
    }
    current = null;
  }

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    if (trimmed.startsWith("# ") && !title) {
      title = trimmed.replace(/^#\s+/, "").trim();
      continue;
    }
    if (trimmed.startsWith("## ")) {
      commitCurrent();
      current = emptyVocabularyPreviewItem(trimmed.replace(/^##\s+/, "").trim());
      continue;
    }
    const match = trimmed.match(/^[-*]\s*([^：:]+)[：:]\s*(.+)$/);
    if (match && current) {
      applyVocabularyPreviewField(current, match[1], match[2].trim());
    }
  }
  commitCurrent();

  return { title, items, skipped };
}

function normalizeSentenceKey(english: string, chinese: string) {
  return `${english.trim().replace(/\s+/g, " ").toLocaleLowerCase()}|||${chinese.trim()}`;
}

function getDueSentences(sentences: SentenceItem[], coursePackId?: string) {
  const now = Date.now();
  return sentences.filter((sentence) => {
    if (coursePackId && sentence.coursePackId !== coursePackId) return false;
    if (sentence.status === "new") return true;
    return Boolean(
      sentence.nextReviewAt && new Date(sentence.nextReviewAt).getTime() <= now,
    );
  });
}

function getStreakDays(stats: Record<string, DailyStats>) {
  const learnedDays = Object.keys(stats)
    .filter((date) => stats[date].newCount + stats[date].reviewCount > 0)
    .sort()
    .reverse();

  let streak = 0;
  const cursor = new Date();
  for (const day of learnedDays) {
    const expected = [
      cursor.getFullYear(),
      `${cursor.getMonth() + 1}`.padStart(2, "0"),
      `${cursor.getDate()}`.padStart(2, "0"),
    ].join("-");
    if (day !== expected) break;
    streak += 1;
    cursor.setDate(cursor.getDate() - 1);
  }
  return streak;
}

function formatShortDate(value: string | null) {
  if (!value) return "还没有记录";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "时间未知";
  return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
}

function addMinutesIso(minutes: number) {
  return new Date(Date.now() + minutes * 60 * 1000).toISOString();
}

function statusLabel(status: SentenceStatus) {
  if (status === "new") return "新句";
  if (status === "learning") return "学习中";
  return "已掌握";
}

function loadLastStudy(): StudyBookmark | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.localStorage.getItem(LAST_STUDY_KEY);
    return raw ? (JSON.parse(raw) as StudyBookmark) : null;
  } catch {
    return null;
  }
}

function saveLastStudy(bookmark: StudyBookmark) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(LAST_STUDY_KEY, JSON.stringify(bookmark));
}

function getPreferredEnglishVoice() {
  const voices = window.speechSynthesis.getVoices();
  const preferredNames = [
    "Microsoft Jenny",
    "Microsoft Aria",
    "Microsoft Guy",
    "Microsoft Zira",
    "Google US English",
    "Google UK English Female",
    "Google UK English Male",
  ];

  return (
    preferredNames
      .map((name) => voices.find((voice) => voice.name.includes(name)))
      .find(Boolean) ??
    voices.find((voice) => voice.lang === "en-US") ??
    voices.find((voice) => voice.lang.startsWith("en-")) ??
    null
  );
}

function speakWithSystemVoice(text: string) {
  if (!text || typeof window === "undefined" || !("speechSynthesis" in window)) {
    return;
  }

  window.speechSynthesis.cancel();
  const utterance = new SpeechSynthesisUtterance(text);
  const voice = getPreferredEnglishVoice();
  if (voice) {
    utterance.voice = voice;
    utterance.lang = voice.lang;
  } else {
    utterance.lang = "en-US";
  }
  utterance.rate = 0.82;
  utterance.pitch = 1;
  window.speechSynthesis.speak(utterance);
}

function buildCourseSummaries(
  coursePacks: CoursePack[],
  sentences: SentenceItem[],
  reviews: ReviewLog[],
): CourseSummary[] {
  const nameCounts = coursePacks.reduce<Record<string, number>>((counts, course) => {
    const key = course.name.trim().toLocaleLowerCase();
    counts[key] = (counts[key] ?? 0) + 1;
    return counts;
  }, {});

  return coursePacks.map((course) => {
    const courseSentences = sentences.filter(
      (sentence) => sentence.coursePackId === course.id,
    );
    const lessonCount = new Set(
      courseSentences.map((sentence) => sentence.lessonTitle),
    ).size;
    const dueCount = getDueSentences(courseSentences).length;
    const masteredCount = courseSentences.filter(
      (sentence) => sentence.status === "mastered",
    ).length;
    const progressPercent = courseSentences.length
      ? Math.round((masteredCount / courseSentences.length) * 100)
      : 0;
    const sentenceIds = new Set(courseSentences.map((sentence) => sentence.id));
    const reviewedAtValues = reviews
      .filter((review) => sentenceIds.has(review.sentenceId))
      .map((review) => review.reviewedAt)
      .sort();
    const lastReviewedAt =
      reviewedAtValues.length > 0
        ? reviewedAtValues[reviewedAtValues.length - 1]
        : null;
    const duplicateCount = nameCounts[course.name.trim().toLocaleLowerCase()] ?? 1;

    return {
      course,
      lessonCount,
      totalSentences: courseSentences.length,
      dueCount,
      masteredCount,
      progressPercent,
      duplicateCount,
      lastReviewedAt,
    };
  });
}

function buildLessonSummaries(sentences: SentenceItem[]): LessonSummary[] {
  const titles = [...new Set(sentences.map((sentence) => sentence.lessonTitle))];
  return titles.map((title, index) => {
    const lessonSentences = sentences.filter(
      (sentence) => sentence.lessonTitle === title,
    );
    const mastered = lessonSentences.filter(
      (sentence) => sentence.status === "mastered",
    ).length;
    const due = getDueSentences(lessonSentences).length;
    const progressPercent = lessonSentences.length
      ? Math.round((mastered / lessonSentences.length) * 100)
      : 0;

    return {
      title,
      index: index + 1,
      total: lessonSentences.length,
      mastered,
      due,
      progressPercent,
    };
  });
}

function App() {
  const [state, setState] = useState<AppState>(initialState);
  const [view, setView] = useState<View>("home");
  const [courseName, setCourseName] = useState("");
  const [sentenceDraft, setSentenceDraft] = useState({
    english: "",
    chinese: "",
    phonetic: "",
    note: "",
  });
  const [importText, setImportText] = useState("");
  const [importMessage, setImportMessage] = useState(
    "粘贴课程文本后，会在下方预览课时和句子。",
  );
  const [showImport, setShowImport] = useState(false);
  const [sessionIndex, setSessionIndex] = useState(0);
  const [showingAnswer, setShowingAnswer] = useState(false);
  const [answerWords, setAnswerWords] = useState<Record<number, string>>({});
  const [answerResult, setAnswerResult] = useState<AnswerResult>("idle");
  const [wrongIndexes, setWrongIndexes] = useState<Set<number>>(new Set());
  const [lastSubmittedAnswer, setLastSubmittedAnswer] = useState("");
  const [databasePath, setDatabasePath] = useState("");
  const [databaseError, setDatabaseError] = useState("");
  const [databaseHydrated, setDatabaseHydrated] = useState(false);
  const [queueCollapsed, setQueueCollapsed] = useState(true);
  const [ttsStatus, setTtsStatus] = useState("");
  const [selectedLessonTitle, setSelectedLessonTitle] = useState<string | null>(null);
  const [studyLessonTitle, setStudyLessonTitle] = useState<string | null>(null);
  const [vocabularyBooks, setVocabularyBooks] = useState<VocabularyBook[]>([]);
  const [vocabularyItems, setVocabularyItems] = useState<VocabularyItem[]>([]);
  const [vocabularyQueue, setVocabularyQueue] = useState<VocabularyItem[]>([]);
  const [activeVocabularyBook, setActiveVocabularyBook] =
    useState<VocabularyBookDetail | null>(null);
  const [wordBookText, setWordBookText] = useState(WORD_BOOK_SAMPLE);
  const [wordBookName, setWordBookName] = useState("");
  const [wordBookMessage, setWordBookMessage] = useState(
    "粘贴 Markdown 单词书后，会在下方预览可导入词条。",
  );
  const [showWordBookImport, setShowWordBookImport] = useState(false);
  const [wordImportMode, setWordImportMode] = useState<"markdown" | "plain">("markdown");
  const [wordCardIndex, setWordCardIndex] = useState(0);
  const [wordCardRevealed, setWordCardRevealed] = useState(false);
  const [syncSettings, setSyncSettings] = useState<SyncSettings>(defaultSyncSettings);
  const [syncForm, setSyncForm] = useState<SyncSettings>(defaultSyncSettings);
  const [webdavPassword, setWebdavPassword] = useState("");
  const [syncMessage, setSyncMessage] = useState("首次配置请使用坚果云第三方应用密码。");
  const [llmSettings, setLlmSettings] = useState<LlmSettings>(defaultLlmSettings);
  const [llmForm, setLlmForm] = useState<LlmSettings>(defaultLlmSettings);
  const [llmProfiles, setLlmProfiles] = useState<LlmProfile[]>([defaultLlmProfile]);
  const [activeLlmProfileId, setActiveLlmProfileId] = useState(defaultLlmProfile.id);
  const [llmProfileName, setLlmProfileName] = useState(defaultLlmProfile.name);
  const [llmApiKey, setLlmApiKey] = useState("");
  const [llmMessage, setLlmMessage] = useState("API Key 会保存到本机 Stronghold，不写入 SQLite。");
  const [plainWordText, setPlainWordText] = useState("abandon\nmaintain\ncommute");
  const [enrichedVocabulary, setEnrichedVocabulary] = useState<EnrichedVocabularyEntry[]>([]);
  const [vocabularyEnrichMessage, setVocabularyEnrichMessage] = useState(
    "也可以只粘贴单词，让 LLM 先补全释义、例句和记忆信息。",
  );
  const [learningPlanSettings, setLearningPlanSettings] = useState<LearningPlanSettings>(
    defaultLearningPlanSettings,
  );
  const [dailyLearningPlan, setDailyLearningPlan] = useState<DailyLearningPlan>(
    defaultDailyLearningPlan,
  );
  const [generatedScenes, setGeneratedScenes] = useState<GeneratedScene[]>([]);
  const [activeScene, setActiveScene] = useState<GeneratedSceneDetail | null>(null);
  const [sceneTopic, setSceneTopic] = useState("daily work and study");
  const [sceneTitle, setSceneTitle] = useState("");
  const [sceneBatchTitle, setSceneBatchTitle] = useState("今日 AI 场景课");
  const [selectedSceneTopics, setSelectedSceneTopics] = useState<string[]>(
    sceneTopicOptions.slice(0, 5),
  );
  const [showSceneAdvanced, setShowSceneAdvanced] = useState(false);
  const [sceneCoursePackId, setSceneCoursePackId] = useState("");
  const [sceneBatchPlan, setSceneBatchPlan] = useState<SceneBatchPlanResponse | null>(null);
  const [sceneMessage, setSceneMessage] = useState("默认智能选择新词、弱词，先规划多节场景课。");
  const [lastStudy, setLastStudy] = useState<StudyBookmark | null>(() =>
    loadLastStudy(),
  );

  useEffect(() => {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) {
      return;
    }
    window.speechSynthesis.getVoices();
  }, []);

  function applyLoadedState(loadedState: AppState) {
    setState((current) => ({
      ...loadedState,
      activeCoursePackId:
        lastStudy?.coursePackId ||
        loadedState.activeCoursePackId ||
        loadedState.coursePacks[0]?.id ||
        current.activeCoursePackId,
    }));
    setDatabaseHydrated(true);
  }

  async function refreshVocabularyData(preferredBookId?: string) {
    if (!hasTauriBridge()) return;
    const [books, items, queue] = await Promise.all([
      safeInvoke<VocabularyBook[]>("list_vocabulary_books"),
      safeInvoke<VocabularyItem[]>("list_vocabulary_items"),
      safeInvoke<VocabularyItem[]>("get_today_vocabulary_queue", {
        now: new Date().toISOString(),
        limit: 20,
      }),
    ]);

    setVocabularyBooks(books);
    setVocabularyItems(items);
    setVocabularyQueue(queue);
    setWordCardIndex(0);
    setWordCardRevealed(false);

    const bookId = preferredBookId ?? books[0]?.id;
    if (bookId) {
      const detail = await safeInvoke<VocabularyBookDetail>("get_vocabulary_book", {
        bookId,
      });
      setActiveVocabularyBook(detail);
    } else {
      setActiveVocabularyBook(null);
    }
  }

  async function refreshCloudData() {
    if (!hasTauriBridge()) return;
    const [sync, llm, profiles, scenes, planSettings, todayPlan] = await Promise.all([
      safeInvoke<SyncSettings>("get_sync_settings"),
      safeInvoke<LlmSettings>("get_llm_settings"),
      safeInvoke<LlmProfile[]>("list_llm_profiles"),
      safeInvoke<GeneratedScene[]>("list_generated_scenes"),
      safeInvoke<LearningPlanSettings>("get_learning_plan_settings"),
      safeInvoke<DailyLearningPlan>("get_today_learning_plan", {
        planDate: todayKey(),
      }),
    ]);
    setSyncSettings(sync);
    setSyncForm(sync);
    setLlmSettings(llm);
    setLlmForm(llm);
    setLearningPlanSettings(planSettings);
    setDailyLearningPlan(todayPlan);
    const defaultProfile = profiles.find((profile) => profile.isDefault) ?? profiles[0];
    setLlmProfiles(profiles.length ? profiles : [defaultLlmProfile]);
    if (defaultProfile) {
      setActiveLlmProfileId(defaultProfile.id);
      setLlmProfileName(defaultProfile.name);
      setLlmForm({
        provider: defaultProfile.provider,
        baseUrl: defaultProfile.baseUrl,
        model: defaultProfile.model,
        wireApi: defaultProfile.wireApi,
        reasoningEffort: defaultProfile.reasoningEffort,
        disableResponseStorage: defaultProfile.disableResponseStorage,
        promptVersion: defaultProfile.promptVersion,
        temperature: defaultProfile.temperature,
        updatedAt: defaultProfile.updatedAt,
      });
    }
    setGeneratedScenes(scenes);
    if (scenes[0]) {
      const detail = await safeInvoke<GeneratedSceneDetail>("get_generated_scene", {
        sceneId: scenes[0].id,
      });
      setActiveScene(detail);
    }
    try {
      setWebdavPassword(await readSecret("webdav-password"));
      const profileKey = defaultProfile
        ? await readSecret(llmSecretKey(defaultProfile.id))
        : "";
      setLlmApiKey(profileKey || (await readSecret("llm-api-key")));
    } catch (error) {
      setSyncMessage(`安全凭据读取失败：${String(error)}`);
    }
  }

  useEffect(() => {
    if (!hasTauriBridge()) {
      setDatabaseHydrated(true);
      return;
    }

    safeInvoke<string>("init_database")
      .then((path) => {
        setDatabasePath(path);
        setDatabaseError("");
        return safeInvoke<AppState>("load_app_state");
      })
      .then((loadedState) => {
        applyLoadedState(loadedState);
        return Promise.all([refreshVocabularyData(), refreshCloudData()]);
      })
      .catch((error) => {
        setDatabaseError(String(error));
        setDatabaseHydrated(true);
      });
  }, []);

  const courseSummaries = useMemo(
    () => buildCourseSummaries(state.coursePacks, state.sentences, state.reviews),
    [state.coursePacks, state.reviews, state.sentences],
  );

  const activeCourse = useMemo(
    () =>
      state.coursePacks.find((course) => course.id === state.activeCoursePackId) ??
      state.coursePacks[0],
    [state.activeCoursePackId, state.coursePacks],
  );

  const activeSummary = useMemo(
    () =>
      activeCourse
        ? courseSummaries.find((summary) => summary.course.id === activeCourse.id)
        : undefined,
    [activeCourse, courseSummaries],
  );

  const activeSentences = useMemo(
    () =>
      activeCourse
        ? state.sentences.filter(
            (sentence) => sentence.coursePackId === activeCourse.id,
          )
        : [],
    [activeCourse, state.sentences],
  );

  const lessonSummaries = useMemo(
    () => buildLessonSummaries(activeSentences),
    [activeSentences],
  );
  const selectedLessonSummary = selectedLessonTitle
    ? lessonSummaries.find((lesson) => lesson.title === selectedLessonTitle)
    : undefined;
  const selectedLessonSentences = selectedLessonTitle
    ? activeSentences.filter((sentence) => sentence.lessonTitle === selectedLessonTitle)
    : activeSentences;

  const session = useMemo(
    () =>
      getDueSentences(state.sentences, activeCourse?.id)
        .filter(
          (sentence) => !studyLessonTitle || sentence.lessonTitle === studyLessonTitle,
        )
        .slice(0, 20),
    [activeCourse?.id, state.sentences, studyLessonTitle],
  );

  const currentSentence = session[sessionIndex];
  const answerParts = currentSentence ? parseAnswerParts(currentSentence.english) : [];
  const expectedWords = currentSentence ? getAnswerWords(currentSentence.english) : [];
  const todayStats = state.stats[todayKey()] ?? {
    newCount: 0,
    reviewCount: 0,
    studyMinutes: 0,
  };
  const dueCount = getDueSentences(state.sentences).length;
  const activeDueCount = activeCourse
    ? getDueSentences(state.sentences, activeCourse.id).length
    : 0;
  const streakDays = getStreakDays(state.stats);
  const importRows = useMemo(() => parseImport(importText), [importText]);
  const todayTarget = activeCourse?.dailyNewTarget || 20;
  const todayPercent = Math.min(
    100,
    Math.round((todayStats.reviewCount / Math.max(1, todayTarget)) * 100),
  );
  const remainingTodaySentences = Math.max(0, todayTarget - todayStats.reviewCount);
  const nextCourse =
    courseSummaries.find((summary) => summary.course.id === lastStudy?.coursePackId) ??
    courseSummaries.find((summary) => summary.dueCount > 0) ??
    courseSummaries[0];
  const vocabularyPreview = useMemo(
    () => parseVocabularyBookPreview(wordBookText),
    [wordBookText],
  );
  const currentVocabularyItem =
    vocabularyQueue[Math.min(wordCardIndex, Math.max(0, vocabularyQueue.length - 1))];
  const dueVocabularyCount = vocabularyQueue.length;
  const weakVocabularyCount = vocabularyItems.filter((item) => item.weakScore > 0).length;
  const activeBookReviewedCount =
    activeVocabularyBook?.items.filter((item) => item.reviewCount > 0).length ?? 0;
  const activeBookWeakCount =
    activeVocabularyBook?.items.filter((item) => item.weakScore > 0).length ?? 0;
  const sceneTargetWords = useMemo(() => {
    const merged = [...vocabularyQueue, ...vocabularyItems.filter((item) => item.weakScore > 0)];
    const seen = new Set<string>();
    return merged
      .filter((item) => {
        if (seen.has(item.id)) return false;
        seen.add(item.id);
        return true;
      })
      .slice(0, 15);
  }, [vocabularyItems, vocabularyQueue]);
  const activeSceneTargets = useMemo(
    () => parseSceneTargetWords(activeScene?.scene.targetWordsSnapshot ?? ""),
    [activeScene?.scene.targetWordsSnapshot],
  );
  const activeSceneCoverage = useMemo(
    () => parseSceneCoverage(activeScene?.scene.coverageJson ?? ""),
    [activeScene?.scene.coverageJson],
  );

  const importPreview = useMemo(() => {
    const existingKeys = new Set(
      activeSentences.map((sentence) =>
        normalizeSentenceKey(sentence.english, sentence.chinese),
      ),
    );
    const seenKeys = new Set<string>();
    let duplicateCount = 0;
    let newCount = 0;

    for (const row of importRows) {
      const key = normalizeSentenceKey(row.english, row.chinese);
      if (existingKeys.has(key) || seenKeys.has(key)) {
        duplicateCount += 1;
      } else {
        seenKeys.add(key);
        newCount += 1;
      }
    }

    return {
      duplicateCount,
      newCount,
      lessonCount: new Set(importRows.map((row) => row.lessonTitle)).size,
    };
  }, [activeSentences, importRows]);

  const studyQueue = useMemo<StudyQueueItem[]>(
    () =>
      session.map((sentence) => ({
        id: sentence.id,
        lessonTitle: sentence.lessonTitle,
        english: sentence.english,
        chinese: sentence.chinese,
        status: sentence.status,
        isCurrent: sentence.id === currentSentence?.id,
      })),
    [currentSentence?.id, session],
  );

  useEffect(() => {
    if (view === "study" && currentSentence) {
      speakEnglish(currentSentence.english);
    }
  }, [currentSentence?.id, view]);

  useEffect(() => {
    if (view !== "study" || !activeCourse || !currentSentence) return;
    const bookmark: StudyBookmark = {
      coursePackId: activeCourse.id,
      sentenceId: currentSentence.id,
      savedAt: new Date().toISOString(),
    };
    saveLastStudy(bookmark);
    setLastStudy(bookmark);
  }, [activeCourse, currentSentence, view]);

  useEffect(() => {
    function handleKeyDown(event: globalThis.KeyboardEvent) {
      if (view !== "study") return;

      if (event.ctrlKey && event.key === "'") {
        event.preventDefault();
        if (currentSentence) speakEnglish(currentSentence.english);
      }

      if (event.ctrlKey && event.key === ";") {
        event.preventDefault();
        setShowingAnswer(true);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [currentSentence, view]);

  async function speakEnglish(text: string) {
    if (!text) return;

    try {
      setTtsStatus("Piper 离线朗读");
      const response = await safeInvoke<TtsResponse>("synthesize_piper_tts", {
        request: { input: text },
      });
      const audio = new Audio(`data:audio/wav;base64,${response.audioBase64}`);
      await audio.play();
      setTtsStatus(response.cached ? "Piper 离线朗读（缓存）" : response.engine);
    } catch (error) {
      setTtsStatus("Piper 暂不可用，已切换本机语音");
      speakWithSystemVoice(text);
      console.warn(error);
    }
  }

  function updateState(updater: (current: AppState) => AppState) {
    setState((current) => updater(current));
  }

  function ensureTodayStats(current: AppState) {
    const key = todayKey();
    return {
      ...current.stats,
      [key]: current.stats[key] ?? {
        newCount: 0,
        reviewCount: 0,
        studyMinutes: 0,
      },
    };
  }

  function resetAnswerState() {
    setAnswerWords({});
    setAnswerResult("idle");
    setWrongIndexes(new Set());
    setLastSubmittedAnswer("");
    setShowingAnswer(false);
  }

  function openView(next: View) {
    setView(next);
    if (next !== "study") {
      setQueueCollapsed(true);
      setStudyLessonTitle(null);
    }
    if (next === "vocabulary") {
      setWordCardRevealed(false);
    }
  }

  function openCourse(coursePackId: string) {
    updateState((current) => ({ ...current, activeCoursePackId: coursePackId }));
    setShowImport(false);
    setSelectedLessonTitle(null);
    openView("courseDetail");
  }

  function openStudy(
    coursePackId = activeCourse?.id,
    preferredSentenceId?: string | null,
    lessonTitle?: string | null,
  ) {
    if (!coursePackId) return;
    const courseSession = getDueSentences(state.sentences, coursePackId)
      .filter((sentence) => !lessonTitle || sentence.lessonTitle === lessonTitle)
      .slice(0, 20);
    const bookmarkSentenceId =
      preferredSentenceId ??
      (!lessonTitle && lastStudy?.coursePackId === coursePackId
        ? lastStudy.sentenceId
        : null);
    const foundIndex = courseSession.findIndex(
      (sentence) => sentence.id === bookmarkSentenceId,
    );

    updateState((current) => ({ ...current, activeCoursePackId: coursePackId }));
    setStudyLessonTitle(lessonTitle ?? null);
    setSessionIndex(foundIndex >= 0 ? foundIndex : 0);
    resetAnswerState();
    setQueueCollapsed(true);
    setView("study");
  }

  async function persistCoursePack(coursePack: CoursePack) {
    if (!databasePath || !databaseHydrated) return;

    await safeInvoke("create_course_pack", {
      course: {
        id: coursePack.id,
        name: coursePack.name,
        language: coursePack.language,
        dailyNewTarget: coursePack.dailyNewTarget,
        createdAt: coursePack.createdAt,
      },
    });
  }

  async function persistLesson(lesson: NewLesson) {
    if (!databasePath || !databaseHydrated) return;
    await safeInvoke("create_lesson", { lesson });
  }

  async function persistSentence(sentence: SentenceItem, lessonId: string) {
    if (!databasePath || !databaseHydrated) return;

    await safeInvoke("create_sentence", {
      sentence: {
        id: sentence.id,
        lessonId,
        english: sentence.english,
        chinese: sentence.chinese,
        phonetic: sentence.phonetic,
        note: sentence.note,
        createdAt: sentence.createdAt,
      },
    });
  }

  async function loadLessons(coursePackId: string) {
    if (!databasePath || !databaseHydrated) return [] as NewLesson[];
    return safeInvoke<NewLesson[]>("list_lessons", { coursePackId });
  }

  async function ensureLessonId(
    coursePackId: string,
    title: string,
    createdAt: string,
  ) {
    const lessons = await loadLessons(coursePackId);
    const existing = lessons.find((lesson) => lesson.title === title);
    if (existing) return existing.id;

    const lesson: NewLesson = {
      id: uid("lesson"),
      coursePackId,
      title,
      sortOrder: lessons.length,
      createdAt,
    };
    await persistLesson(lesson);
    return lesson.id;
  }

  async function createCourse(event: FormEvent) {
    event.preventDefault();
    const name = courseName.trim();
    if (!name) return;

    const hasDuplicate = state.coursePacks.some(
      (course) => course.name.trim().toLocaleLowerCase() === name.toLocaleLowerCase(),
    );
    if (
      hasDuplicate &&
      !window.confirm(`已经有名为「${name}」的课程包，仍然创建一个新的？`)
    ) {
      return;
    }

    const coursePack: CoursePack = {
      id: uid("course"),
      name,
      language: "en",
      dailyNewTarget: 20,
      createdAt: new Date().toISOString(),
    };

    try {
      await persistCoursePack(coursePack);
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
      return;
    }

    updateState((current) => ({
      ...current,
      activeCoursePackId: coursePack.id,
      coursePacks: [coursePack, ...current.coursePacks],
    }));
    setCourseName("");
    setShowImport(true);
    setSelectedLessonTitle(null);
    setView("courseDetail");
  }

  async function deleteCourse(coursePackId: string) {
    const course = state.coursePacks.find((item) => item.id === coursePackId);
    if (!window.confirm(`删除「${course?.name ?? "这个课程"}」及其所有句子？`)) {
      return;
    }

    if (databasePath && databaseHydrated) {
      try {
        await safeInvoke("delete_course_pack", { coursePackId });
        setDatabaseError("");
      } catch (error) {
        setDatabaseError(String(error));
        return;
      }
    }

    updateState((current) => {
      const coursePacks = current.coursePacks.filter(
        (item) => item.id !== coursePackId,
      );
      return {
        ...current,
        activeCoursePackId: coursePacks[0]?.id ?? "",
        coursePacks,
        sentences: current.sentences.filter(
          (sentence) => sentence.coursePackId !== coursePackId,
        ),
        reviews: current.reviews.filter(
          (review) => review.coursePackId !== coursePackId,
        ),
      };
    });
    if (activeCourse?.id === coursePackId) {
      setView("courses");
    }
  }

  async function deleteSentence(sentenceId: string) {
    if (!window.confirm("删除这条句子？")) return;

    if (databasePath && databaseHydrated) {
      try {
        await safeInvoke("delete_sentence", { sentenceId });
        setDatabaseError("");
      } catch (error) {
        setDatabaseError(String(error));
        return;
      }
    }

    updateState((current) => ({
      ...current,
      sentences: current.sentences.filter(
        (sentence) => sentence.id !== sentenceId,
      ),
      reviews: current.reviews.filter((review) => review.sentenceId !== sentenceId),
    }));
  }

  async function addSentence(event: FormEvent) {
    event.preventDefault();
    if (
      !activeCourse ||
      !sentenceDraft.english.trim() ||
      !sentenceDraft.chinese.trim()
    ) {
      return;
    }

    const key = normalizeSentenceKey(sentenceDraft.english, sentenceDraft.chinese);
    const duplicate = activeSentences.some(
      (sentence) => normalizeSentenceKey(sentence.english, sentence.chinese) === key,
    );
    if (duplicate && !window.confirm("当前课程里已有相同句子，仍然添加？")) {
      return;
    }

    const createdAt = new Date().toISOString();
    const lessonId = await ensureLessonId(activeCourse.id, MANUAL_LESSON, createdAt);
    const sentence: SentenceItem = {
      id: uid("sentence"),
      coursePackId: activeCourse.id,
      lessonTitle: MANUAL_LESSON,
      english: sentenceDraft.english.trim(),
      chinese: sentenceDraft.chinese.trim(),
      phonetic: sentenceDraft.phonetic.trim(),
      note: sentenceDraft.note.trim(),
      status: "new",
      favorite: false,
      showCount: 0,
      reviewCount: 0,
      errorCount: 0,
      nextReviewAt: null,
      createdAt,
    };

    try {
      await persistSentence(sentence, lessonId);
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
      return;
    }

    updateState((current) => ({
      ...current,
      sentences: [...current.sentences, sentence],
    }));
    setSentenceDraft({ english: "", chinese: "", phonetic: "", note: "" });
  }

  async function importSentences() {
    if (!activeCourse || !importRows.length) return;

    const existingKeys = new Set(
      activeSentences.map((sentence) =>
        normalizeSentenceKey(sentence.english, sentence.chinese),
      ),
    );
    const seenKeys = new Set<string>();
    const rows: ImportRow[] = [];
    let skipped = 0;

    for (const row of importRows) {
      const key = normalizeSentenceKey(row.english, row.chinese);
      if (existingKeys.has(key) || seenKeys.has(key)) {
        skipped += 1;
      } else {
        seenKeys.add(key);
        rows.push(row);
      }
    }

    if (!rows.length) {
      setImportMessage(`没有新增句子，已跳过 ${skipped} 条重复内容。`);
      return;
    }

    const now = new Date().toISOString();
    const lessonMap = new Map<string, string>();

    try {
      await persistCoursePack(activeCourse);
      const existingLessons = await loadLessons(activeCourse.id);
      existingLessons.forEach((lesson) => lessonMap.set(lesson.title, lesson.id));

      const lessonTitles = [...new Set(rows.map((row) => row.lessonTitle))];
      let createdLessonCount = 0;
      for (const title of lessonTitles) {
        if (lessonMap.has(title)) continue;
        const lesson: NewLesson = {
          id: uid("lesson"),
          coursePackId: activeCourse.id,
          title,
          sortOrder: existingLessons.length + createdLessonCount,
          createdAt: now,
        };
        await persistLesson(lesson);
        lessonMap.set(title, lesson.id);
        createdLessonCount += 1;
      }

      const sentences = rows.map<SentenceItem>((row) => ({
        id: uid("sentence"),
        coursePackId: activeCourse.id,
        lessonTitle: row.lessonTitle,
        english: row.english,
        chinese: row.chinese,
        phonetic: row.phonetic,
        note: row.note,
        status: "new",
        favorite: false,
        showCount: 0,
        reviewCount: 0,
        errorCount: 0,
        nextReviewAt: null,
        createdAt: now,
      }));

      for (const sentence of sentences) {
        const lessonId = lessonMap.get(sentence.lessonTitle);
        if (!lessonId) {
          throw new Error(`找不到课时：${sentence.lessonTitle}`);
        }
        await persistSentence(sentence, lessonId);
      }

      updateState((current) => ({
        ...current,
        sentences: [...current.sentences, ...sentences],
      }));
      setImportText("");
      setImportMessage(
        `已导入 ${sentences.length} 句，跳过 ${skipped} 条重复内容，识别 ${lessonTitles.length} 个课时。`,
      );
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
      setImportMessage(`导入失败：${String(error)}`);
    }
  }

  async function importVocabularyBook() {
    if (!vocabularyPreview.items.length) {
      setWordBookMessage("没有识别到可导入词条。至少需要 `## 单词` 和 `- 中文释义：...`。");
      return;
    }

    try {
      const result = await safeInvoke<ImportVocabularyBookResult>("import_vocabulary_book", {
        request: {
          name: wordBookName.trim() || vocabularyPreview.title || "未命名单词书",
          source: "markdown",
          rawText: wordBookText,
          note: "",
          importedAt: new Date().toISOString(),
        },
      });
      await refreshVocabularyData(result.book.id);
      setWordBookMessage(
        `已导入 ${result.importedCount} 个新词，复用 ${result.reusedCount} 个旧词，跳过 ${result.skippedCount} 个不完整词条。`,
      );
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
      setWordBookMessage(`导入失败：${String(error)}`);
    }
  }

  async function openVocabularyBook(bookId: string) {
    try {
      const detail = await safeInvoke<VocabularyBookDetail>("get_vocabulary_book", {
        bookId,
      });
      setActiveVocabularyBook(detail);
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
    }
  }

  async function reviewVocabularyItem(rating: Rating) {
    if (!currentVocabularyItem) return;
    const rule = vocabularyRatingRules[rating];
    const reviewedAt = new Date().toISOString();

    try {
      const updated = await safeInvoke<VocabularyItem>("review_vocabulary_item", {
        review: {
          vocabularyItemId: currentVocabularyItem.id,
          mode: "card",
          rating,
          reviewedAt,
          nextReviewAt: addMinutesIso(rule.minutes),
        },
      });
      setVocabularyItems((items) =>
        items.map((item) => (item.id === updated.id ? updated : item)),
      );
      setVocabularyQueue((items) => items.filter((item) => item.id !== updated.id));
      setWordCardIndex((index) =>
        Math.max(0, Math.min(index, vocabularyQueue.length - 2)),
      );
      setWordCardRevealed(false);
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
    }
  }

  async function saveSyncConfig(event?: FormEvent) {
    event?.preventDefault();
    try {
      const saved = await safeInvoke<SyncSettings>("save_sync_settings", {
        request: {
          provider: syncForm.provider,
          baseUrl: syncForm.baseUrl,
          username: syncForm.username,
          remotePath: syncForm.remotePath,
          autoSyncEnabled: syncForm.autoSyncEnabled,
        },
      });
      if (webdavPassword.trim()) {
        await writeSecret("webdav-password", webdavPassword.trim());
      }
      setSyncSettings(saved);
      setSyncForm(saved);
      setSyncMessage("同步设置已保存，密码保存在本机 Stronghold。");
      setDatabaseError("");
      return saved;
    } catch (error) {
      setSyncMessage(`保存失败：${String(error)}`);
      setDatabaseError(String(error));
      return null;
    }
  }

  async function testWebDavConnection() {
    const saved = await saveSyncConfig();
    if (!saved) return;
    const password = webdavPassword.trim() || (await readSecret("webdav-password"));
    try {
      const result = await safeInvoke<SyncActionResult>("test_webdav_connection", {
        request: { password },
      });
      setSyncMessage(`连接成功：${result.remoteSnapshotUrl}`);
      await refreshCloudData();
    } catch (error) {
      setSyncMessage(`连接失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function runWebDavSync() {
    const saved = await saveSyncConfig();
    if (!saved) return;
    const password = webdavPassword.trim() || (await readSecret("webdav-password"));
    try {
      const result = await safeInvoke<SyncActionResult>("run_webdav_sync", {
        request: { password },
      });
      setSyncMessage(
        `同步完成：合并 ${result.mergedRows} 行，上传 ${result.uploadedBytes} bytes`,
      );
      await refreshCloudData();
    } catch (error) {
      setSyncMessage(`同步失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function clearSyncCredential() {
    try {
      await removeSecret("webdav-password");
      setWebdavPassword("");
      setSyncMessage("已清除本机 WebDAV 密码。");
    } catch (error) {
      setSyncMessage(`清除失败：${String(error)}`);
    }
  }

  async function selectLlmProfile(profileId: string) {
    const profile = llmProfiles.find((item) => item.id === profileId);
    if (!profile) return;
    setActiveLlmProfileId(profile.id);
    setLlmProfileName(profile.name);
    setLlmForm({
      provider: profile.provider,
      baseUrl: profile.baseUrl,
      model: profile.model,
      wireApi: profile.wireApi,
      reasoningEffort: profile.reasoningEffort,
      disableResponseStorage: profile.disableResponseStorage,
      promptVersion: profile.promptVersion,
      temperature: profile.temperature,
      updatedAt: profile.updatedAt,
    });
    try {
      setLlmApiKey(await readSecret(llmSecretKey(profile.id)));
    } catch (error) {
      setLlmMessage(`读取配置密钥失败：${String(error)}`);
    }
  }

  function startNewLlmProfile() {
    setActiveLlmProfileId("");
    setLlmProfileName("新的 LLM 配置");
    setLlmForm(defaultLlmSettings);
    setLlmApiKey("");
  }

  function openWordStudy() {
    if (!vocabularyQueue.length) {
      setWordBookMessage("今日暂无背词队列，可以先导入词书或调整学习计划。");
      return;
    }
    setWordCardIndex(0);
    setWordCardRevealed(false);
    setView("wordStudy");
  }

  async function saveLlmConfig(event?: FormEvent) {
    event?.preventDefault();
    try {
      const saved = await safeInvoke<LlmProfile>("save_llm_profile", {
        request: {
          id: activeLlmProfileId || null,
          name: llmProfileName,
          provider: llmForm.provider,
          baseUrl: llmForm.baseUrl,
          model: llmForm.model,
          wireApi: llmForm.wireApi,
          reasoningEffort: llmForm.reasoningEffort,
          disableResponseStorage: llmForm.disableResponseStorage,
          temperature: Number(llmForm.temperature),
          isDefault: true,
        },
      });
      if (llmApiKey.trim()) {
        await writeSecret(llmSecretKey(saved.id), llmApiKey.trim());
      }
      setActiveLlmProfileId(saved.id);
      setLlmProfileName(saved.name);
      setLlmSettings(saved);
      setLlmForm({
        provider: saved.provider,
        baseUrl: saved.baseUrl,
        model: saved.model,
        wireApi: saved.wireApi,
        reasoningEffort: saved.reasoningEffort,
        disableResponseStorage: saved.disableResponseStorage,
        promptVersion: saved.promptVersion,
        temperature: saved.temperature,
        updatedAt: saved.updatedAt,
      });
      setLlmProfiles(await safeInvoke<LlmProfile[]>("list_llm_profiles"));
      setLlmMessage("LLM 配置已保存，API Key 按配置保存在本机 Stronghold。");
      setDatabaseError("");
      return saved;
    } catch (error) {
      setLlmMessage(`保存失败：${String(error)}`);
      setDatabaseError(String(error));
      return null;
    }
  }

  async function testLlmConfig() {
    const saved = await saveLlmConfig();
    if (!saved) return;
    const apiKey = llmApiKey.trim() || (await readSecret(llmSecretKey(saved.id)));
    if (!apiKey) {
      setLlmMessage("请先填写 API Key。");
      return;
    }
    try {
      await safeInvoke<string>("test_llm_profile", {
        request: { profileId: saved.id, apiKey },
      });
      setLlmMessage("连接测试成功。");
    } catch (error) {
      setLlmMessage(`连接测试失败：${String(error)}`);
    }
  }

  async function deleteActiveLlmProfile() {
    if (!activeLlmProfileId) return;
    try {
      await safeInvoke("delete_llm_profile", { profileId: activeLlmProfileId });
      await removeSecret(llmSecretKey(activeLlmProfileId));
      setLlmMessage("已删除当前 LLM 配置。");
      await refreshCloudData();
    } catch (error) {
      setLlmMessage(`删除失败：${String(error)}`);
    }
  }

  async function enrichPlainWords() {
    const saved = await saveLlmConfig();
    if (!saved) return;
    const apiKey = llmApiKey.trim() || (await readSecret(llmSecretKey(saved.id)));
    if (!apiKey) {
      setVocabularyEnrichMessage("请先在同步设置页保存 LLM API Key。");
      setView("settings");
      return;
    }
    const words = parsePlainWordList(plainWordText);
    if (!words.length) {
      setVocabularyEnrichMessage("请先粘贴单词，每行一个或用逗号分隔。");
      return;
    }
    setVocabularyEnrichMessage("正在让 LLM 补全词条，生成后会先预览。");
    try {
      const result = await safeInvoke<EnrichVocabularyResponse>("enrich_vocabulary_words", {
        request: { profileId: saved.id, apiKey, words },
      });
      setEnrichedVocabulary(result.entries);
      setVocabularyEnrichMessage(`已补全 ${result.entries.length} 个词条，请检查后再确认入库。`);
      setWordBookText(vocabularyEntriesToMarkdown(wordBookName || "LLM 补全词书", result.entries));
    } catch (error) {
      setVocabularyEnrichMessage(`补全失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function importEnrichedVocabulary() {
    if (!enrichedVocabulary.length) return;
    const rawText = vocabularyEntriesToMarkdown(
      wordBookName || "LLM 补全词书",
      enrichedVocabulary,
    );
    try {
      const result = await safeInvoke<ImportVocabularyBookResult>("import_vocabulary_book", {
        request: {
          name: wordBookName || "LLM 补全词书",
          source: "llm_enriched_markdown",
          rawText,
          note: "由 LLM 补全后确认导入",
          importedAt: new Date().toISOString(),
        },
      });
      await refreshVocabularyData(result.book.id);
      setVocabularyEnrichMessage(
        `已入库 ${result.importedCount} 个新词，复用 ${result.reusedCount} 个旧词。`,
      );
    } catch (error) {
      setVocabularyEnrichMessage(`入库失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function saveLearningIntensity(intensity: LearningPlanSettings["intensity"]) {
    try {
      const settings = await safeInvoke<LearningPlanSettings>("save_learning_plan_settings", {
        request: { intensity },
      });
      const plan = await safeInvoke<DailyLearningPlan>("get_today_learning_plan", {
        planDate: todayKey(),
      });
      setLearningPlanSettings(settings);
      setDailyLearningPlan(plan);
    } catch (error) {
      setDatabaseError(String(error));
    }
  }

  async function openGeneratedScene(sceneId: string) {
    try {
      const detail = await safeInvoke<GeneratedSceneDetail>("get_generated_scene", {
        sceneId,
      });
      setActiveScene(detail);
    } catch (error) {
      setSceneMessage(`读取场景失败：${String(error)}`);
    }
  }

  async function generateScene() {
    const saved = await saveLlmConfig();
    if (!saved) return;
    const apiKey = llmApiKey.trim() || (await readSecret(llmSecretKey(saved.id)));
    if (!apiKey) {
      setSceneMessage("请先在同步设置页保存 LLM API Key。");
      setView("settings");
      return;
    }
    const targetIds = sceneTargetWords.map((item) => item.id);
    if (!targetIds.length) {
      setSceneMessage("还没有可用于生成场景的单词，请先导入单词书。");
      return;
    }

    setSceneMessage("正在生成场景，对话会按结构化 JSON 保存。");
    try {
      const detail = await safeInvoke<GeneratedSceneDetail>(
        "generate_scene_from_vocabulary",
        {
          request: {
            apiKey,
            title: sceneTitle,
            topic: sceneTopic,
            vocabularyItemIds: targetIds,
          },
        },
      );
      setActiveScene(detail);
      const scenes = await safeInvoke<GeneratedScene[]>("list_generated_scenes");
      setGeneratedScenes(scenes);
      setSceneMessage(`已生成：${detail.scene.title}`);
      setDatabaseError("");
    } catch (error) {
      setSceneMessage(`生成失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function planSceneBatch() {
    setSceneMessage("正在规划场景课，优先使用未学新词和弱词。");
    try {
      const plan = await safeInvoke<SceneBatchPlanResponse>("plan_scene_batch", {
        request: {
          title: sceneBatchTitle,
          coursePackId: sceneCoursePackId || activeCourse?.id || null,
          selectedTopics: selectedSceneTopics,
          coreWordIds: [],
        },
      });
      setSceneBatchPlan(plan);
      setSceneMessage(
        `已规划 ${plan.batch.plannedSceneCount} 节课，核心词 ${plan.batch.coreWordCount} 个。`,
      );
    } catch (error) {
      setSceneMessage(`规划失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function generatePlannedSceneBatch() {
    const saved = await saveLlmConfig();
    if (!saved) return;
    const apiKey = llmApiKey.trim() || (await readSecret(llmSecretKey(saved.id)));
    if (!apiKey) {
      setSceneMessage("请先在同步设置页保存 LLM API Key。");
      setView("settings");
      return;
    }
    const plan = sceneBatchPlan ?? (await safeInvoke<SceneBatchPlanResponse>("plan_scene_batch", {
      request: {
        title: sceneBatchTitle,
        coursePackId: sceneCoursePackId || activeCourse?.id || null,
        selectedTopics: selectedSceneTopics,
        coreWordIds: [],
      },
    }));
    setSceneBatchPlan(plan);
    setSceneMessage("正在批量生成场景课，会自动校验核心词覆盖率。");
    try {
      const updated = await safeInvoke<SceneBatchPlanResponse>("generate_scene_batch", {
        request: { batchId: plan.batch.id, profileId: saved.id, apiKey },
      });
      setSceneBatchPlan(updated);
      const scenes = await safeInvoke<GeneratedScene[]>("list_generated_scenes");
      setGeneratedScenes(scenes);
      setSceneMessage(`批量生成完成：${updated.batch.status}`);
    } catch (error) {
      setSceneMessage(`批量生成失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function addCurrentBatchToCourse() {
    const targetCourseId = sceneCoursePackId || activeCourse?.id;
    if (!targetCourseId) {
      setSceneMessage("请先选择要加入的课程。");
      return;
    }
    const batchId = sceneBatchPlan?.batch.id;
    const sceneIds = generatedScenes
      .filter(
        (scene) =>
          scene.status === "succeeded" &&
          !scene.isAddedToCourse &&
          (!batchId || scene.batchId === batchId),
      )
      .map((scene) => scene.id);
    if (!sceneIds.length) {
      setSceneMessage("当前没有可加入课程的成功场景课。");
      return;
    }
    try {
      const result = await safeInvoke<AddScenesToCourseResponse>("add_scenes_to_course", {
        request: { sceneIds, coursePackId: targetCourseId },
      });
      setSceneMessage(
        `已加入课程：${result.createdLessons} 个课时，${result.createdSentences} 句。`,
      );
      const [loadedState, scenes] = await Promise.all([
        safeInvoke<AppState>("load_app_state"),
        safeInvoke<GeneratedScene[]>("list_generated_scenes"),
      ]);
      applyLoadedState(loadedState);
      setGeneratedScenes(scenes);
    } catch (error) {
      setSceneMessage(`加入课程失败：${String(error)}`);
      setDatabaseError(String(error));
    }
  }

  async function seedDemo() {
    const coursePack: CoursePack = {
      id: uid("course"),
      name: "经济讨论句子课",
      language: "en",
      dailyNewTarget: 20,
      createdAt: new Date().toISOString(),
    };
    const rows: ImportRow[] = [
      {
        lessonTitle: "第1课：经济讨论",
        english: "The market is showing strong growth this quarter.",
        chinese: "市场这个季度展现出强劲的增长。",
        phonetic: "",
        note: "",
      },
      {
        lessonTitle: "第1课：经济讨论",
        english: "Yeah, the economy is developing faster than most people expect.",
        chinese: "是的，经济发展比大多数人预期的要快。",
        phonetic: "",
        note: "",
      },
      {
        lessonTitle: "第2课：经济观点",
        english: "What's your view on the current economic situation?",
        chinese: "你对当前的经济形势有什么看法？",
        phonetic: "",
        note: "",
      },
    ];
    const now = new Date().toISOString();
    const lessonTitles = [...new Set(rows.map((row) => row.lessonTitle))];
    const lessons = lessonTitles.map<NewLesson>((title, index) => ({
      id: uid("lesson"),
      coursePackId: coursePack.id,
      title,
      sortOrder: index,
      createdAt: now,
    }));
    const lessonIds = new Map(lessons.map((lesson) => [lesson.title, lesson.id]));
    const sentences = rows.map<SentenceItem>((row) => ({
      id: uid("sentence"),
      coursePackId: coursePack.id,
      lessonTitle: row.lessonTitle,
      english: row.english,
      chinese: row.chinese,
      phonetic: row.phonetic,
      note: row.note,
      status: "new",
      favorite: false,
      showCount: 0,
      reviewCount: 0,
      errorCount: 0,
      nextReviewAt: null,
      createdAt: now,
    }));

    try {
      await persistCoursePack(coursePack);
      for (const lesson of lessons) await persistLesson(lesson);
      for (const sentence of sentences) {
        await persistSentence(sentence, lessonIds.get(sentence.lessonTitle) ?? "");
      }
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
      return;
    }

    updateState((current) => ({
      ...current,
      activeCoursePackId: coursePack.id,
      coursePacks: [coursePack, ...current.coursePacks],
      sentences: [...current.sentences, ...sentences],
    }));
    setSelectedLessonTitle(null);
    setView("courseDetail");
  }

  function evaluateAnswer() {
    if (!currentSentence) return;

    const actual = buildAnswerFromWords(answerParts, answerWords);
    setLastSubmittedAnswer(actual);

    const wrong = getWrongWordIndexes(expectedWords, answerWords);

    if (!wrong.size) {
      setAnswerResult("correct");
      setWrongIndexes(new Set());
      setShowingAnswer(true);
      return;
    }

    setAnswerResult("wrong");
    setWrongIndexes(wrong);
    setShowingAnswer(false);
    window.setTimeout(() => focusNextEditableAnswerInput(Math.min(...wrong)), 0);
  }

  function submitAnswer(event: FormEvent) {
    event.preventDefault();
    if (answerResult === "correct") {
      applyRating("good", true);
      return;
    }
    evaluateAnswer();
  }

  function applyRating(rating: Rating, force = false) {
    if (!currentSentence) return;
    if (!force && answerResult !== "correct") return;

    const rule = ratingRules[rating];
    const reviewedAt = new Date().toISOString();
    const nextReviewAt = new Date(
      Date.now() + rule.minutes * 60 * 1000,
    ).toISOString();
    const isCorrect = force || answerResult === "correct";
    const review: ReviewSubmission = {
      id: uid("review"),
      sentenceId: currentSentence.id,
      userAnswer: lastSubmittedAnswer || buildAnswerFromWords(answerParts, answerWords),
      isCorrect,
      rating,
      reviewedAt,
      intervalMinutes: rule.minutes,
      wrongIndexes: [...wrongIndexes],
      nextReviewAt,
    };

    updateState((current) => {
      const stats = ensureTodayStats(current);
      const today = stats[todayKey()];
      const target = current.sentences.find(
        (sentence) => sentence.id === currentSentence.id,
      );
      const wasNew = target?.status === "new";

      return {
        ...current,
        stats: {
          ...stats,
          [todayKey()]: {
            ...today,
            newCount: today.newCount + (wasNew ? 1 : 0),
            reviewCount: today.reviewCount + 1,
          },
        },
        reviews: [
          ...current.reviews,
          {
            ...review,
            coursePackId: currentSentence.coursePackId,
          },
        ],
        sentences: current.sentences.map((sentence) =>
          sentence.id === currentSentence.id
            ? {
                ...sentence,
                status: rule.status,
                reviewCount: sentence.reviewCount + 1,
                showCount: sentence.showCount + 1,
                errorCount: sentence.errorCount + (isCorrect ? 0 : 1),
                nextReviewAt,
              }
            : sentence,
        ),
      };
    });

    if (databasePath && databaseHydrated) {
      safeInvoke("record_review", { review }).catch((error) => {
        setDatabaseError(String(error));
      });
    }

    setSessionIndex((index) => Math.max(0, Math.min(index, session.length - 2)));
    resetAnswerState();
  }

  function handleAnswerKeyDown(
    event: ReactKeyboardEvent<HTMLInputElement>,
    wordIndex: number,
  ) {
    if (event.key === " ") {
      event.preventDefault();
      focusNextEditableAnswerInput(wordIndex + 1);
      return;
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      focusNextEditableAnswerInput(wordIndex + 1);
      return;
    }

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      focusPreviousEditableAnswerInput(wordIndex - 1);
      return;
    }

    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      event.currentTarget.form?.requestSubmit();
    }
  }

  function focusNextEditableAnswerInput(startIndex: number) {
    const inputs = [
      ...document.querySelectorAll<HTMLInputElement>(".blank-input"),
    ].filter((input) => !input.disabled);
    const nextInput =
      inputs.find(
        (input) => Number(input.dataset.wordIndex ?? "-1") >= startIndex,
      ) ?? inputs[0];
    nextInput?.focus();
    nextInput?.select();
  }

  function focusPreviousEditableAnswerInput(startIndex: number) {
    const inputs = [
      ...document.querySelectorAll<HTMLInputElement>(".blank-input"),
    ].filter((input) => !input.disabled);
    const previousInput =
      inputs
        .slice()
        .reverse()
        .find(
          (input) => Number(input.dataset.wordIndex ?? "-1") <= startIndex,
        ) ?? inputs[inputs.length - 1];
    previousInput?.focus();
    previousInput?.select();
  }

  return (
    <div className={`app-shell ${view === "study" || view === "wordStudy" ? "study-mode" : ""}`}>
      {view !== "study" && view !== "wordStudy" && (
        <aside className="sidebar">
          <div className="brand">
            <div className="brand-mark">M</div>
            <div>
              <div className="brand-title">MomoLite</div>
              <div className="brand-subtitle">Sentence trainer</div>
            </div>
          </div>
          <nav className="nav">
            {navItems.map((item) => {
              const Icon = item.icon;
              return (
                <button
                  className={`nav-item ${view === item.view ? "active" : ""}`}
                  key={item.view}
                  onClick={() => openView(item.view)}
                  type="button"
                >
                  <Icon size={18} />
                  <span>{item.label}</span>
                </button>
              );
            })}
          </nav>
          <div className="side-note">
            <Sparkles size={16} />
            <span>本地 SQLite + 离线朗读</span>
          </div>
        </aside>
      )}

      <main className="workspace">
        {view !== "study" && view !== "wordStudy" && (
          <header className="topbar">
            <div>
              <h1>{viewCopy[view].title}</h1>
              <p>{viewCopy[view].subtitle}</p>
            </div>
            <div className="topbar-actions">
              {activeCourse && (
                <button
                  className="ghost-button"
                  onClick={() => openCourse(activeCourse.id)}
                  type="button"
                >
                  <BookOpen size={17} />
                  {activeCourse.name}
                </button>
              )}
              <button
                className="primary-button"
                disabled={!activeCourse || activeDueCount === 0}
                onClick={() => openStudy(activeCourse?.id)}
                type="button"
              >
                <Play size={17} />
                继续学习
              </button>
            </div>
          </header>
        )}

        {databaseError && (
          <div className="notice danger-notice">
            本地数据或外部服务异常：{databaseError}
          </div>
        )}

        {!hasTauriBridge() && (
          <div className="notice">
            <BookOpen size={16} />
            当前是网页预览，只能查看界面；SQLite 和 Piper 需要用 MomoLite 桌面软件打开。
          </div>
        )}

        {!databaseHydrated && (
          <div className="notice">
            <RefreshCw size={16} />
            正在载入本地学习数据
          </div>
        )}

        {view === "home" && (
          <section className="home-page">
            <section className="home-hero mission-hero">
              <div className="hero-copy">
                <div className="eyebrow">
                  <Sparkles size={16} />
                  今日任务中心
                </div>
                <h2>今天先拿下词，再把它变成能听能说的英文。</h2>
                <p>
                  {dueVocabularyCount
                    ? `今日有 ${dueVocabularyCount} 个单词在队列里，完成后就能继续生成场景课。`
                    : nextCourse
                      ? `下一步建议继续「${nextCourse.course.name}」，还差 ${remainingTodaySentences} 句到今日目标。`
                      : "先导入一本单词书，或创建一个课程包。"}
                </p>
                <div className="hero-actions">
                  <button
                    className="primary-button large"
                    disabled={!dueVocabularyCount}
                    onClick={openWordStudy}
                    type="button"
                  >
                    <Play size={18} />
                    今日背词
                  </button>
                  <button
                    className="ghost-button large"
                    onClick={() =>
                      nextCourse ? openCourse(nextCourse.course.id) : openView("courses")
                    }
                    type="button"
                  >
                    <LibraryBig size={18} />
                    {nextCourse ? "句子训练" : "创建课程"}
                  </button>
                </div>
              </div>
              <div className="goal-card mission-goal">
                <div
                  className="goal-ring"
                  style={{ "--progress": `${todayPercent}%` } as React.CSSProperties}
                >
                  <span>{todayPercent}%</span>
                </div>
                <div>
                  <strong>今日目标</strong>
                  <span>
                    {todayStats.reviewCount}/{todayTarget} 句
                  </span>
                  <em>还差 {remainingTodaySentences} 句</em>
                </div>
              </div>
            </section>

            <section className="mission-board">
              <button
                className="mission-card primary-mission"
                disabled={!dueVocabularyCount}
                onClick={openWordStudy}
                type="button"
              >
                <span className="mission-icon"><Target size={20} /></span>
                <span>
                  <strong>今日背词</strong>
                  <em>{dueVocabularyCount ? `${dueVocabularyCount} 个词待复习` : "队列已清空"}</em>
                </span>
                <ChevronRight size={18} />
              </button>
              <button
                className="mission-card"
                onClick={() => (nextCourse ? openStudy(nextCourse.course.id) : openView("courses"))}
                type="button"
              >
                <span className="mission-icon"><BookOpen size={20} /></span>
                <span>
                  <strong>句子训练</strong>
                  <em>{dueCount} 句待学，目标还差 {remainingTodaySentences} 句</em>
                </span>
                <ChevronRight size={18} />
              </button>
              <button
                className="mission-card"
                onClick={() => openView("scenes")}
                type="button"
              >
                <span className="mission-icon"><Sparkles size={20} /></span>
                <span>
                  <strong>生成场景课</strong>
                  <em>{generatedScenes.length} 节草稿，把词放进对话</em>
                </span>
                <ChevronRight size={18} />
              </button>
            </section>

            <section className="dashboard-grid home-dashboard">
              <Surface className="quest-panel" title="今日学习计划">
                <div className="quest-header">
                  <span className="soft-badge accent">
                    {learningPlanSettings.intensity === "light"
                      ? "浅学模式"
                      : learningPlanSettings.intensity === "intensive"
                        ? "高强度模式"
                        : "标准模式"}
                  </span>
                  <strong>
                    新词 {dailyLearningPlan.newWordTarget} · 复习 {dailyLearningPlan.dueCount} · 场景课{" "}
                    {dailyLearningPlan.sceneLessonTarget}
                  </strong>
                </div>
                <ProgressBar value={todayPercent} />
                <p>{dailyLearningPlan.explanation || "先背词，再进句子和场景课，今天只推进一小步。"}</p>
                <div className="reward-strip">
                  <div>
                    <Flame size={18} />
                    <span>连续 {streakDays} 天</span>
                  </div>
                  <div>
                    <ListChecks size={18} />
                    <span>{dueCount} 句待学</span>
                  </div>
                  <div>
                    <Trophy size={18} />
                    <span>{weakVocabularyCount} 个弱词</span>
                  </div>
                </div>
              </Surface>

              <Surface className="content-health-panel" title="内容库存">
                <div className="mini-stat-grid">
                  <div>
                    <span>全局词条</span>
                    <strong>{vocabularyItems.length}</strong>
                  </div>
                  <div>
                    <span>今日队列</span>
                    <strong>{dueVocabularyCount}</strong>
                  </div>
                  <div>
                    <span>单词书</span>
                    <strong>{vocabularyBooks.length}</strong>
                  </div>
                  <div>
                    <span>场景课</span>
                    <strong>{generatedScenes.length}</strong>
                  </div>
                </div>
              </Surface>
            </section>

            <section className="dashboard-grid">
              <Surface
                action={
                  <button
                    className="small-action"
                    onClick={() => openView("courses")}
                    type="button"
                  >
                    全部课程
                    <ChevronRight size={16} />
                  </button>
                }
                title="最近课程"
              >
                <div className="course-card-list">
                  {courseSummaries.length ? (
                    courseSummaries.slice(0, 4).map((summary) => (
                      <CourseCard
                        key={summary.course.id}
                        onDelete={deleteCourse}
                        onOpen={openCourse}
                        onStudy={openStudy}
                        summary={summary}
                      />
                    ))
                  ) : (
                    <EmptyState
                      actionLabel="创建第一门课程"
                      icon={LibraryBig}
                      onAction={() => openView("courses")}
                      title="还没有课程包"
                    />
                  )}
                </div>
              </Surface>

              <Surface title="继续学习">
                {nextCourse ? (
                  <div className="continue-card">
                    <div className="badge-row">
                      <span className="soft-badge">
                        {nextCourse.lessonCount} 课时
                      </span>
                      <span className="soft-badge accent">
                        {nextCourse.dueCount} 句待学
                      </span>
                    </div>
                    <h3>{nextCourse.course.name}</h3>
                    <ProgressBar value={nextCourse.progressPercent} />
                    <div className="continue-footer">
                      <span>上次：{formatShortDate(nextCourse.lastReviewedAt)}</span>
                      <button
                        className="primary-button"
                        disabled={nextCourse.dueCount === 0}
                        onClick={() => openStudy(nextCourse.course.id)}
                        type="button"
                      >
                        <Play size={16} />
                        继续
                      </button>
                    </div>
                  </div>
                ) : (
                  <EmptyState
                    actionLabel="去课程包"
                    icon={BookOpen}
                    onAction={() => openView("courses")}
                    title="还没有可学习内容"
                  />
                )}
              </Surface>
            </section>
          </section>
        )}

        {view === "vocabulary" && (
          <section className="vocabulary-page library-page">
            <div className="library-actions">
              <div className="segmented">
                <button
                  className={learningPlanSettings.intensity === "light" ? "active" : ""}
                  onClick={() => saveLearningIntensity("light")}
                  type="button"
                >
                  浅学
                </button>
                <button
                  className={learningPlanSettings.intensity === "standard" ? "active" : ""}
                  onClick={() => saveLearningIntensity("standard")}
                  type="button"
                >
                  标准
                </button>
                <button
                  className={learningPlanSettings.intensity === "intensive" ? "active" : ""}
                  onClick={() => saveLearningIntensity("intensive")}
                  type="button"
                >
                  高强度
                </button>
              </div>
              <button
                className="primary-button"
                disabled={!dueVocabularyCount}
                onClick={openWordStudy}
                type="button"
              >
                <Play size={17} />
                专注背词
              </button>
              <button
                className="ghost-button"
                onClick={() => setShowWordBookImport((value) => !value)}
                type="button"
              >
                <Import size={17} />
                {showWordBookImport ? "收起导入" : "新建/导入词书"}
              </button>
            </div>

            <section className="vocabulary-scoreboard">
              <div className="score-card word-score-card">
                <span>今日背词队列</span>
                <strong>{dueVocabularyCount}</strong>
                <em>{dueVocabularyCount ? "进入专注模式，一次只看一个词" : "今日队列已完成"}</em>
              </div>
              <div className="score-card">
                <span>全局词库</span>
                <strong>{vocabularyItems.length}</strong>
                <em>{weakVocabularyCount} 个弱词会优先进入场景课</em>
              </div>
              <div className="score-card">
                <span>当前词书进度</span>
                <strong>
                  {activeVocabularyBook
                    ? `${activeBookReviewedCount}/${activeVocabularyBook.items.length}`
                    : "0/0"}
                </strong>
                <em>{activeVocabularyBook?.book.name ?? "选择一本词书查看详情"}</em>
              </div>
            </section>

            <section className="library-shell">
              <Surface className="library-sidebar-panel" title="词书">
                <div className="wordbook-list library-list">
                  {vocabularyBooks.length ? (
                    vocabularyBooks.map((book) => (
                      <button
                        className={`wordbook-row ${
                          activeVocabularyBook?.book.id === book.id ? "active" : ""
                        }`}
                        key={book.id}
                        onClick={() => openVocabularyBook(book.id)}
                        type="button"
                      >
                        <div>
                          <div className="row-title">{book.name}</div>
                          <div className="row-subtitle">
                            {book.itemCount} 词 · {formatShortDate(book.importedAt)}
                          </div>
                        </div>
                        <ChevronRight size={16} />
                      </button>
                    ))
                  ) : (
                    <EmptyState
                      actionLabel="新建词书"
                      icon={Import}
                      onAction={() => setShowWordBookImport(true)}
                      title="还没有单词书"
                    />
                  )}
                </div>
              </Surface>

              <Surface
                className="library-detail-panel"
                title={activeVocabularyBook ? activeVocabularyBook.book.name : "词书详情"}
              >
                {showWordBookImport ? (
                  <div className="import-drawer">
                    <div className="segmented">
                      <button
                        className={wordImportMode === "markdown" ? "active" : ""}
                        onClick={() => setWordImportMode("markdown")}
                        type="button"
                      >
                        Markdown 词书
                      </button>
                      <button
                        className={wordImportMode === "plain" ? "active" : ""}
                        onClick={() => setWordImportMode("plain")}
                        type="button"
                      >
                        只给单词
                      </button>
                    </div>
                    <input
                      onChange={(event) => setWordBookName(event.target.value)}
                      placeholder={vocabularyPreview.title || "先输入词书名称"}
                      value={wordBookName}
                    />
                    {wordImportMode === "markdown" ? (
                      <>
                        <textarea
                          onChange={(event) => {
                            setWordBookText(event.target.value);
                            setWordBookMessage("粘贴 Markdown 单词书后，会在下方预览。");
                          }}
                          spellCheck={false}
                          value={wordBookText}
                        />
                        <div className="import-footer">
                          <span>{wordBookMessage}</span>
                          <span>
                            识别 {vocabularyPreview.items.length} · 跳过{" "}
                            {vocabularyPreview.skipped}
                          </span>
                        </div>
                        {vocabularyPreview.items.length > 0 && (
                          <VocabularyPreview items={vocabularyPreview.items.slice(0, 5)} />
                        )}
                        <button
                          className="primary-button"
                          disabled={!vocabularyPreview.items.length}
                          onClick={importVocabularyBook}
                          type="button"
                        >
                          <Import size={17} />
                          创建并导入
                        </button>
                      </>
                    ) : (
                      <>
                        <textarea
                          onChange={(event) => setPlainWordText(event.target.value)}
                          placeholder="每行一个单词，也可以用逗号分隔"
                          spellCheck={false}
                          value={plainWordText}
                        />
                        <div className="import-footer">
                          <span>{vocabularyEnrichMessage}</span>
                          <span>{parsePlainWordList(plainWordText).length} 个词</span>
                        </div>
                        <div className="button-row">
                          <button
                            className="ghost-button"
                            onClick={enrichPlainWords}
                            type="button"
                          >
                            <Sparkles size={17} />
                            LLM 补全预览
                          </button>
                          <button
                            className="primary-button"
                            disabled={!enrichedVocabulary.length}
                            onClick={importEnrichedVocabulary}
                            type="button"
                          >
                            <CheckCircle2 size={17} />
                            确认入库
                          </button>
                        </div>
                        {enrichedVocabulary.length > 0 && (
                          <VocabularyPreview items={enrichedVocabulary.slice(0, 6)} />
                        )}
                      </>
                    )}
                  </div>
                ) : activeVocabularyBook ? (
                  <div className="book-detail-view">
                    <section className="book-hero">
                      <div>
                        <span className="soft-badge">单词书</span>
                        <h2>{activeVocabularyBook.book.name}</h2>
                        <p>
                          {activeVocabularyBook.items.length} 个词 · 已复习 {activeBookReviewedCount} · 弱词{" "}
                          {activeBookWeakCount}
                        </p>
                      </div>
                      <button
                        className="primary-button large"
                        disabled={!dueVocabularyCount}
                        onClick={openWordStudy}
                        type="button"
                      >
                        <Play size={18} />
                        进入专注背词
                      </button>
                    </section>
                    <ProgressBar
                      value={Math.round(
                        (activeBookReviewedCount / Math.max(1, activeVocabularyBook.items.length)) *
                          100,
                      )}
                    />
                    <div className="book-summary-strip">
                      <div>
                        <span>词条</span>
                        <strong>{activeVocabularyBook.items.length}</strong>
                      </div>
                      <div>
                        <span>已复习</span>
                        <strong>{activeBookReviewedCount}</strong>
                      </div>
                      <div>
                        <span>弱项</span>
                        <strong>{activeBookWeakCount}</strong>
                      </div>
                      <div>
                        <span>今日队列</span>
                        <strong>{dueVocabularyCount}</strong>
                      </div>
                    </div>
                    <VocabularyList items={activeVocabularyBook.items} />
                  </div>
                ) : (
                  <EmptyState
                    actionLabel="新建词书"
                    icon={BookOpen}
                    onAction={() => setShowWordBookImport(true)}
                    title="选择一本词书，或新建导入"
                  />
                )}
              </Surface>
            </section>
          </section>
        )}

        {view === "wordStudy" && (
          <section className="word-focus-page">
            {currentVocabularyItem ? (
              <>
                <header className="word-focus-topbar">
                  <button
                    className="icon-button"
                    onClick={() => openView("vocabulary")}
                    type="button"
                    aria-label="退出背词"
                  >
                    <X size={24} />
                  </button>
                  <div className="word-focus-progress">
                    <strong>专注背词</strong>
                    <span>{Math.min(wordCardIndex + 1, vocabularyQueue.length)}/{vocabularyQueue.length}</span>
                    <ProgressBar
                      value={Math.round(
                        ((wordCardIndex + 1) / Math.max(1, vocabularyQueue.length)) * 100,
                      )}
                    />
                  </div>
                  <button
                    className="icon-button"
                    onClick={() => setWordCardRevealed((value) => !value)}
                    type="button"
                    aria-label="切换详情"
                  >
                    <Menu size={22} />
                  </button>
                </header>

                <main className="word-focus-card">
                  <section className="word-focus-hero">
                    <h2>{currentVocabularyItem.text}</h2>
                    <div className="word-focus-phonetic">
                      {currentVocabularyItem.partOfSpeech || "word"}
                      {currentVocabularyItem.phonetic ? ` · ${currentVocabularyItem.phonetic}` : ""}
                      <button
                        className="icon-button"
                        onClick={() => speakEnglish(currentVocabularyItem.text)}
                        type="button"
                        aria-label="朗读"
                      >
                        <Volume2 size={18} />
                      </button>
                    </div>
                  </section>

                  <section className="word-focus-meaning">
                    <strong>{currentVocabularyItem.primaryMeaning}</strong>
                  </section>

                  {wordCardRevealed ? (
                    <section className="word-focus-scroll">
                      {currentVocabularyItem.example && (
                        <div className="focus-section">
                          <h3>例句</h3>
                          <p>{currentVocabularyItem.example}</p>
                          {currentVocabularyItem.exampleCn && <em>{currentVocabularyItem.exampleCn}</em>}
                        </div>
                      )}
                      <WordDetailGrid item={currentVocabularyItem} />
                    </section>
                  ) : (
                    <button
                      className="focus-reveal-button"
                      onClick={() => setWordCardRevealed(true)}
                      type="button"
                    >
                      展开例句和助记
                    </button>
                  )}
                </main>

                <footer className="word-focus-actions">
                  <button
                    className="rating-button danger"
                    onClick={() => reviewVocabularyItem("again")}
                    type="button"
                  >
                    忘记
                    <span>稍后再来</span>
                  </button>
                  <button
                    className="rating-button warn"
                    onClick={() => reviewVocabularyItem("hard")}
                    type="button"
                  >
                    模糊
                    <span>需要提醒</span>
                  </button>
                  <button
                    className="rating-button primary"
                    onClick={() => reviewVocabularyItem("good")}
                    type="button"
                  >
                    认识
                    <span>进入复习</span>
                  </button>
                  <button
                    className="rating-button success"
                    onClick={() => reviewVocabularyItem("easy")}
                    type="button"
                  >
                    熟悉
                    <span>延后出现</span>
                  </button>
                </footer>
              </>
            ) : (
              <div className="study-complete">
                <div className="complete-mark">
                  <CheckCircle2 size={28} />
                </div>
                <h2>今日单词队列完成</h2>
                <p>可以去生成场景课，把今天的词放进真实对话里。</p>
                <div className="complete-actions">
                  <button className="ghost-button" onClick={() => openView("vocabulary")} type="button">
                    返回词书
                  </button>
                  <button className="primary-button" onClick={() => openView("scenes")} type="button">
                    生成场景课
                  </button>
                </div>
              </div>
            )}
          </section>
        )}

        {view === "scenes" && (
          <section className="scenes-page">
            <section className="scene-pipeline">
              <div className="pipeline-step active">
                <span>1</span>
                <strong>规划</strong>
                <em>选词与主题</em>
              </div>
              <div className={`pipeline-step ${sceneBatchPlan ? "active" : ""}`}>
                <span>2</span>
                <strong>生成</strong>
                <em>{sceneBatchPlan?.batch.plannedSceneCount ?? 0} 节候选课</em>
              </div>
              <div className={`pipeline-step ${activeScene ? "active" : ""}`}>
                <span>3</span>
                <strong>阅读</strong>
                <em>高亮目标词</em>
              </div>
              <div className="pipeline-step">
                <span>4</span>
                <strong>入课</strong>
                <em>进入训练</em>
              </div>
            </section>

            <section className="scene-workbench">
              <Surface
                className="scene-planner"
                action={
                  <button
                    className="small-button"
                    onClick={() => setShowSceneAdvanced((value) => !value)}
                    type="button"
                  >
                    <Menu size={16} />
                    设置
                  </button>
                }
                title="场景课生成"
              >
                <div className="scene-planner-main">
                  <div>
                    <div className="eyebrow">
                      <Sparkles size={16} />
                      AI 场景课
                    </div>
                    <h2>把今天的词编成一组能读、能听、能练的课。</h2>
                    <div className="scene-topic-summary">
                      {selectedSceneTopics.slice(0, 4).map((topic) => (
                        <span key={topic}>{topic}</span>
                      ))}
                      {selectedSceneTopics.length > 4 && (
                        <span>+{selectedSceneTopics.length - 4}</span>
                      )}
                    </div>
                  </div>
                  <div className="scene-command-stack">
                    <button className="ghost-button" onClick={planSceneBatch} type="button">
                      <ListChecks size={17} />
                      规划课表
                    </button>
                    <button
                      className="primary-button"
                      disabled={!vocabularyItems.length}
                      onClick={generatePlannedSceneBatch}
                      type="button"
                    >
                      <Sparkles size={17} />
                      生成草稿
                    </button>
                    <button
                      className="ghost-button"
                      onClick={addCurrentBatchToCourse}
                      type="button"
                    >
                      <Plus size={17} />
                      加入课程
                    </button>
                  </div>
                </div>

                <div className="scene-planner-strip">
                  <div>
                    <span>核心词池</span>
                    <strong>{sceneBatchPlan?.batch.coreWordCount ?? sceneTargetWords.length}</strong>
                  </div>
                  <div>
                    <span>预计课数</span>
                    <strong>{sceneBatchPlan?.batch.plannedSceneCount ?? 0}</strong>
                  </div>
                  <div>
                    <span>草稿</span>
                    <strong>{generatedScenes.filter((scene) => !scene.isAddedToCourse).length}</strong>
                  </div>
                </div>

                <div className="target-word-shelf">
                  {sceneTargetWords.length ? (
                    sceneTargetWords.slice(0, 12).map((word) => (
                      <span className="target-word-pill" key={word.id}>
                        {word.text}
                      </span>
                    ))
                  ) : (
                    <span className="muted">暂无可生成的目标词</span>
                  )}
                  {sceneTargetWords.length > 12 && (
                    <span className="target-word-pill">+{sceneTargetWords.length - 12}</span>
                  )}
                </div>

                {showSceneAdvanced && (
                  <div className="scene-advanced">
                    <div className="scene-field-grid">
                      <input
                        onChange={(event) => setSceneBatchTitle(event.target.value)}
                        placeholder="批次名，例如 CET4 Week 1 晚间复习"
                        value={sceneBatchTitle}
                      />
                      <select
                        onChange={(event) => setSceneCoursePackId(event.target.value)}
                        value={sceneCoursePackId}
                      >
                        <option value="">先生成草稿</option>
                        {state.coursePacks.map((course) => (
                          <option key={course.id} value={course.id}>
                            {course.name}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div className="topic-chip-grid">
                      {sceneTopicOptions.map((topic) => (
                        <button
                          className={`topic-chip ${
                            selectedSceneTopics.includes(topic) ? "active" : ""
                          }`}
                          aria-pressed={selectedSceneTopics.includes(topic)}
                          key={topic}
                          onClick={() =>
                            setSelectedSceneTopics((current) =>
                              current.includes(topic)
                                ? current.filter((item) => item !== topic)
                                : [...current, topic],
                            )
                          }
                          type="button"
                        >
                          {topic}
                        </button>
                      ))}
                    </div>
                    <div className="single-scene-row">
                      <input
                        onChange={(event) => setSceneTitle(event.target.value)}
                        placeholder="单课标题，可留空"
                        value={sceneTitle}
                      />
                      <input
                        onChange={(event) => setSceneTopic(event.target.value)}
                        placeholder="单课主题"
                        value={sceneTopic}
                      />
                      <button
                        className="small-button"
                        disabled={!sceneTargetWords.length}
                        onClick={generateScene}
                        type="button"
                      >
                        单课
                      </button>
                    </div>
                  </div>
                )}

                {sceneBatchPlan && (
                  <div className="plan-strip">
                    {sceneBatchPlan.plans.slice(0, 6).map((plan) => (
                      <div className="plan-chip" key={plan.id}>
                        <strong>第 {plan.sortOrder + 1} 课</strong>
                        <span>
                          {jsonArrayCount(plan.coreWordIds)} 词 · {plan.status}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
                <p className="status-text">{sceneMessage}</p>
              </Surface>

              <Surface className="scene-drafts" title="草稿箱">
                <div className="scene-card-list">
                  {generatedScenes.length ? (
                    generatedScenes.map((scene) => (
                      <button
                        className={`scene-card ${
                          activeScene?.scene.id === scene.id ? "active" : ""
                        }`}
                        key={scene.id}
                        onClick={() => openGeneratedScene(scene.id)}
                        type="button"
                      >
                        <div>
                          <strong>{scene.title}</strong>
                          <span>
                            {scene.status}
                            {scene.isAddedToCourse ? " · 已入课" : " · 草稿"} ·{" "}
                            {jsonArrayCount(scene.targetWordsSnapshot)} 词 ·{" "}
                            {formatShortDate(scene.updatedAt)}
                          </span>
                        </div>
                        <ChevronRight size={16} />
                      </button>
                    ))
                  ) : (
                    <EmptyState icon={Sparkles} title="还没有场景课草稿" />
                  )}
                </div>
              </Surface>
            </section>

            <Surface className="scene-reader-surface" title={activeScene ? "沉浸阅读" : "阅读区"}>
              {activeScene ? (
                <article className="scene-reader">
                  <header className="scene-reader-header">
                    <div>
                      <span className="soft-badge accent">AI 场景课</span>
                      <h2>{activeScene.scene.title}</h2>
                      <p>{activeScene.scene.scenario}</p>
                    </div>
                    <div className="scene-reader-meta">
                      <span>{activeScene.lines.length} 句</span>
                      <span>{activeSceneTargets.length} 词</span>
                      <span>{formatCoverageRate(activeSceneCoverage.coreCoverageRate)}</span>
                      <span>{activeScene.scene.isAddedToCourse ? "已入课程" : "草稿"}</span>
                    </div>
                  </header>

                  {activeSceneTargets.length > 0 && (
                    <section className="reader-word-index">
                      <div className="reader-section-title">
                        <ListChecks size={16} />
                        核心词索引
                      </div>
                      <div className="reader-word-grid">
                        {activeSceneTargets.map((word) => (
                          <div className="reader-word" key={word.id || word.text}>
                            <strong>{word.text}</strong>
                            <span>{word.meaning || word.difficulty || "目标词"}</span>
                          </div>
                        ))}
                      </div>
                    </section>
                  )}

                  <div className="immersive-lines">
                    {activeScene.lines.map((line, index) => (
                      <section className="immersive-line" key={line.id}>
                        <div className="speaker-chip">
                          {line.speaker || (index % 2 ? "B" : "A")}
                        </div>
                        <div className="line-body">
                          <p className="line-en">
                            {highlightTargetWords(line.english, activeSceneTargets)}
                          </p>
                          <p className="line-cn">{line.chinese}</p>
                        </div>
                      </section>
                    ))}
                  </div>
                </article>
              ) : (
                <EmptyState icon={Sparkles} title="点击一节场景课，进入阅读视图" />
              )}
            </Surface>
          </section>
        )}

        {view === "settings" && (
          <section className="settings-page">
            <section className="settings-intro">
              <div>
                <span className="soft-badge">系统能力</span>
                <h2>把同步、模型和语音放在这里，学习时不被打扰。</h2>
                <p>日常学习只看任务页；需要调整外部服务时再进入设置。</p>
              </div>
            </section>
            <div className="settings-grid">
              <Surface title="坚果云 WebDAV 同步">
                <form className="settings-form" onSubmit={saveSyncConfig}>
                  <label>
                    服务商
                    <input
                      readOnly
                      value={syncForm.provider === "jianguoyun_webdav" ? "坚果云 WebDAV" : syncForm.provider}
                    />
                  </label>
                  <label>
                    WebDAV 地址
                    <input
                      onChange={(event) =>
                        setSyncForm((current) => ({
                          ...current,
                          baseUrl: event.target.value,
                        }))
                      }
                      value={syncForm.baseUrl}
                    />
                  </label>
                  <label>
                    账号
                    <input
                      onChange={(event) =>
                        setSyncForm((current) => ({
                          ...current,
                          username: event.target.value,
                        }))
                      }
                      placeholder="坚果云账号邮箱"
                      value={syncForm.username}
                    />
                  </label>
                  <label>
                    第三方应用密码
                    <input
                      onChange={(event) => setWebdavPassword(event.target.value)}
                      placeholder={webdavPassword ? "已保存，输入新密码可覆盖" : "不是网页登录密码"}
                      type="password"
                      value={webdavPassword}
                    />
                  </label>
                  <label>
                    远程目录
                    <input
                      onChange={(event) =>
                        setSyncForm((current) => ({
                          ...current,
                          remotePath: event.target.value,
                        }))
                      }
                      value={syncForm.remotePath}
                    />
                  </label>
                  <label className="inline-check">
                    <input
                      checked={syncForm.autoSyncEnabled}
                      onChange={(event) =>
                        setSyncForm((current) => ({
                          ...current,
                          autoSyncEnabled: event.target.checked,
                        }))
                      }
                      type="checkbox"
                    />
                    自动同步
                  </label>
                  <div className="button-row">
                    <button className="primary-button" type="submit">
                      保存
                    </button>
                    <button className="ghost-button" onClick={testWebDavConnection} type="button">
                      测试连接
                    </button>
                    <button className="ghost-button" onClick={runWebDavSync} type="button">
                      立即同步
                    </button>
                    <button className="small-button" onClick={clearSyncCredential} type="button">
                      清除凭据
                    </button>
                  </div>
                  <p className="status-text">
                    {syncMessage} 状态：{syncSettings.syncStatus}
                  </p>
                </form>
              </Surface>

              <Surface title="LLM 场景生成">
                <form className="settings-form" onSubmit={saveLlmConfig}>
                  <label>
                    配置档案
                    <select
                      onChange={(event) => selectLlmProfile(event.target.value)}
                      value={activeLlmProfileId}
                    >
                      {llmProfiles.map((profile) => (
                        <option key={profile.id} value={profile.id}>
                          {profile.name}
                          {profile.isDefault ? " · 默认" : ""}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    配置名称
                    <input
                      onChange={(event) => setLlmProfileName(event.target.value)}
                      placeholder="例如：我的中转站 GPT-5.4"
                      value={llmProfileName}
                    />
                  </label>
                  <label>
                    Provider
                    <select
                      onChange={(event) => {
                        const provider = event.target.value;
                        const preset = llmProviderPresets[provider];
                        setLlmForm((current) => ({
                          ...current,
                          provider,
                          baseUrl: preset?.baseUrl ?? current.baseUrl,
                          model: preset?.model ?? current.model,
                          wireApi: preset?.wireApi ?? current.wireApi,
                          reasoningEffort:
                            preset?.reasoningEffort ?? current.reasoningEffort,
                          disableResponseStorage:
                            preset?.disableResponseStorage ??
                            current.disableResponseStorage,
                          temperature: preset?.temperature ?? current.temperature,
                        }));
                      }}
                      value={llmForm.provider}
                    >
                      {Object.entries(llmProviderPresets).map(([value, preset]) => (
                        <option key={value} value={value}>
                          {preset.label}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Wire API
                    <select
                      onChange={(event) =>
                        setLlmForm((current) => ({
                          ...current,
                          wireApi: event.target.value as LlmSettings["wireApi"],
                        }))
                      }
                      value={llmForm.wireApi}
                    >
                      <option value="responses">Responses</option>
                      <option value="chat_completions">Chat Completions</option>
                    </select>
                  </label>
                  <label>
                    API 地址
                    <input
                      onChange={(event) =>
                        setLlmForm((current) => ({
                          ...current,
                          baseUrl: event.target.value,
                        }))
                      }
                      value={llmForm.baseUrl}
                    />
                  </label>
                  <label>
                    Model
                    <input
                      onChange={(event) =>
                        setLlmForm((current) => ({
                          ...current,
                          model: event.target.value,
                        }))
                      }
                      placeholder={
                        llmForm.provider === "volcengine_ark"
                          ? "填写火山方舟推理接入点 ID，例如 ep-xxxxxxxx"
                          : llmForm.provider === "gpt2"
                            ? "gpt-5.4"
                          : "填写模型名，例如 gpt-4.1-mini"
                      }
                      value={llmForm.model}
                    />
                  </label>
                  {llmForm.wireApi === "responses" && (
                    <>
                      <label>
                        Reasoning Effort
                        <select
                          onChange={(event) =>
                            setLlmForm((current) => ({
                              ...current,
                              reasoningEffort:
                                event.target.value as LlmSettings["reasoningEffort"],
                            }))
                          }
                          value={llmForm.reasoningEffort}
                        >
                          <option value="minimal">minimal</option>
                          <option value="low">low</option>
                          <option value="medium">medium</option>
                          <option value="high">high</option>
                        </select>
                      </label>
                      <label className="inline-check">
                        <input
                          checked={llmForm.disableResponseStorage}
                          onChange={(event) =>
                            setLlmForm((current) => ({
                              ...current,
                              disableResponseStorage: event.target.checked,
                            }))
                          }
                          type="checkbox"
                        />
                        disable_response_storage / store=false
                      </label>
                    </>
                  )}
                  <label>
                    Temperature
                    <input
                      max={2}
                      min={0}
                      onChange={(event) =>
                        setLlmForm((current) => ({
                          ...current,
                          temperature: Number(event.target.value),
                        }))
                      }
                      step={0.1}
                      type="number"
                      value={llmForm.temperature}
                    />
                  </label>
                  <label>
                    API Key
                    <input
                      onChange={(event) => setLlmApiKey(event.target.value)}
                      placeholder={llmApiKey ? "已保存，输入新 Key 可覆盖" : "仅保存在本机 Stronghold"}
                      type="password"
                      value={llmApiKey}
                    />
                  </label>
                  <div className="button-row">
                    <button className="primary-button" type="submit">
                      保存配置
                    </button>
                    <button className="ghost-button" onClick={testLlmConfig} type="button">
                      测试连接
                    </button>
                    <button className="ghost-button" onClick={startNewLlmProfile} type="button">
                      新建配置
                    </button>
                    <button className="small-button" onClick={deleteActiveLlmProfile} type="button">
                      删除配置
                    </button>
                    <button
                      className="ghost-button"
                      onClick={() => openView("scenes")}
                      type="button"
                    >
                      去生成场景
                    </button>
                  </div>
                  <p className="status-text">
                    {llmMessage} Prompt：{llmSettings.promptVersion}。Responses 模式按 reasoning effort 控制，temperature 不随请求发送。
                  </p>
                </form>
              </Surface>
            </div>
          </section>
        )}

        {view === "courses" && (
          <section className="courses-page">
            <section className="course-library-hero">
              <div>
                <span className="soft-badge accent">课程库</span>
                <h2>把句子按课程收好，每次只推进一小节。</h2>
                <p>{state.coursePacks.length} 个课程包 · {state.sentences.length} 句 · {dueCount} 句待学</p>
              </div>
              <form className="create-course-form" onSubmit={createCourse}>
                <input
                  onChange={(event) => setCourseName(event.target.value)}
                  placeholder="例如：商务英语 30 天"
                  value={courseName}
                />
                <button className="primary-button" type="submit">
                  <Plus size={17} />
                  新建课程
                </button>
              </form>
            </section>
            <div className="course-grid">
              {courseSummaries.length ? (
                courseSummaries.map((summary) => (
                  <CourseCard
                    key={summary.course.id}
                    onDelete={deleteCourse}
                    onOpen={openCourse}
                    onStudy={openStudy}
                    summary={summary}
                  />
                ))
              ) : (
                <EmptyState
                  actionLabel="生成示例课程"
                  icon={Sparkles}
                  onAction={seedDemo}
                  title="从一门课程开始"
                />
              )}
            </div>
          </section>
        )}

        {view === "courseDetail" && (
          <section className="course-detail-page">
            {activeCourse && activeSummary ? (
              <>
                <section className="course-cover">
                  <button
                    className="icon-text-button"
                    onClick={() => openView("courses")}
                    type="button"
                  >
                    <ArrowLeft size={17} />
                    课程包
                  </button>
                  <div className="course-cover-main">
                    <div>
                      <div className="eyebrow">
                        <BookOpen size={16} />
                        {activeSummary.lessonCount} 课时 · {activeSummary.totalSentences} 句
                      </div>
                      <h2>{activeCourse.name}</h2>
                      <p>
                        {activeSummary.dueCount > 0
                          ? `今天还有 ${activeSummary.dueCount} 句可以训练。`
                          : "当前课程今日队列已完成。"}
                      </p>
                    </div>
                    <div className="course-cover-actions">
                      <button
                        className="ghost-button"
                        onClick={() => setShowImport((value) => !value)}
                        type="button"
                      >
                        <Import size={17} />
                        导入句子
                      </button>
                      <button
                        className="primary-button"
                        disabled={activeSummary.dueCount === 0}
                        onClick={() => openStudy(activeCourse.id)}
                        type="button"
                      >
                        <Play size={17} />
                        继续学习
                      </button>
                    </div>
                  </div>
                  <ProgressBar value={activeSummary.progressPercent} />
                  {activeSummary.duplicateCount > 1 && (
                    <div className="notice compact">
                      有 {activeSummary.duplicateCount} 个同名课程包，后续导入前请确认当前课程。
                    </div>
                  )}
                </section>

                <div className="detail-grid">
                  <section className="lesson-section">
                    <div className="section-heading">
                      <h2>课时</h2>
                      <span>{lessonSummaries.length} 个</span>
                    </div>
                    <LessonGrid
                      activeTitle={selectedLessonTitle}
                      lessons={lessonSummaries}
                      onSelect={setSelectedLessonTitle}
                    />
                  </section>

                  <section className="side-stack">
                    {showImport && (
                      <Surface
                        action={
                          <button
                            className="primary-button"
                            disabled={importPreview.newCount === 0}
                            onClick={importSentences}
                            type="button"
                          >
                            <Import size={17} />
                            导入
                          </button>
                        }
                        title="批量导入"
                      >
                        <div className="import-format">
                          <code>## 第一天</code>
                          <code>English sentence.=中文意思</code>
                        </div>
                        <textarea
                          onChange={(event) => {
                            setImportText(event.target.value);
                            setImportMessage(
                              "粘贴课程文本后，会在下方预览课时和句子。",
                            );
                          }}
                          placeholder={`## 第一天\nYeah, the economy is developing faster than most people expect.=是的，经济发展比大多数人预期的要快。\n\n## 第二天\nHey, did you receive my text yesterday?=嘿，你收到我昨天的短信了吗？`}
                          spellCheck={false}
                          value={importText}
                        />
                        <div className="import-footer">
                          <span>{importMessage}</span>
                          <span>
                            新增 {importPreview.newCount} · 重复 {importPreview.duplicateCount}
                          </span>
                        </div>
                        {importRows.length > 0 && (
                          <ImportPreview rows={importRows.slice(0, 8)} />
                        )}
                      </Surface>
                    )}

                    <Surface title="添加单句">
                      <form className="sentence-form" onSubmit={addSentence}>
                        <input
                          onChange={(event) =>
                            setSentenceDraft({
                              ...sentenceDraft,
                              english: event.target.value,
                            })
                          }
                          placeholder="英文句子"
                          value={sentenceDraft.english}
                        />
                        <input
                          onChange={(event) =>
                            setSentenceDraft({
                              ...sentenceDraft,
                              chinese: event.target.value,
                            })
                          }
                          placeholder="中文意思"
                          value={sentenceDraft.chinese}
                        />
                        <input
                          onChange={(event) =>
                            setSentenceDraft({
                              ...sentenceDraft,
                              phonetic: event.target.value,
                            })
                          }
                          placeholder="音标，可不填"
                          value={sentenceDraft.phonetic}
                        />
                        <input
                          onChange={(event) =>
                            setSentenceDraft({
                              ...sentenceDraft,
                              note: event.target.value,
                            })
                          }
                          placeholder="备注"
                          value={sentenceDraft.note}
                        />
                        <button className="primary-button" type="submit">
                          <Plus size={17} />
                          添加句子
                        </button>
                      </form>
                    </Surface>
                  </section>
                </div>

                <Surface
                  action={
                    selectedLessonTitle ? (
                      <div className="row-actions">
                        <button
                          className="ghost-button"
                          onClick={() => setSelectedLessonTitle(null)}
                          type="button"
                        >
                          查看全部
                        </button>
                        <button
                          className="primary-button"
                          disabled={!selectedLessonSummary || selectedLessonSummary.due === 0}
                          onClick={() =>
                            openStudy(activeCourse.id, null, selectedLessonTitle)
                          }
                          type="button"
                        >
                          <Play size={17} />
                          学习本课
                        </button>
                      </div>
                    ) : undefined
                  }
                  title={selectedLessonTitle ? `「${selectedLessonTitle}」句子` : "全部句子"}
                >
                  {selectedLessonTitle && selectedLessonSummary && (
                    <div className="lesson-detail-strip">
                      <span>{selectedLessonSummary.total} 句</span>
                      <span>{selectedLessonSummary.due} 待学</span>
                      <span>{selectedLessonSummary.mastered} 已掌握</span>
                    </div>
                  )}
                  <SentenceList
                    onDelete={deleteSentence}
                    sentences={selectedLessonSentences}
                  />
                </Surface>
              </>
            ) : (
              <EmptyState
                actionLabel="新建课程"
                icon={LibraryBig}
                onAction={() => openView("courses")}
                title="先选择一个课程包"
              />
            )}
          </section>
        )}

        {view === "study" && (
          <section className={`study-layout ${queueCollapsed ? "queue-collapsed" : ""}`}>
            <section className="study-card">
              {currentSentence && activeCourse ? (
                <>
                  <div className="study-topbar">
                    <button
                      className="icon-text-button"
                      onClick={() => openCourse(activeCourse.id)}
                      type="button"
                    >
                      <ArrowLeft size={17} />
                      {studyLessonTitle ?? activeCourse.name}
                    </button>
                    <div className="study-progress">
                      <span>
                        {Math.min(sessionIndex + 1, session.length)} / {session.length}
                      </span>
                      <ProgressBar
                        value={Math.round(
                          ((sessionIndex + 1) / Math.max(1, session.length)) * 100,
                        )}
                      />
                    </div>
                    <div className="study-tools">
                      <button
                        aria-label="朗读"
                        className="icon-button"
                        onClick={() => speakEnglish(currentSentence.english)}
                        type="button"
                      >
                        <Volume2 size={19} />
                      </button>
                      <button
                        aria-label="显示答案"
                        className="icon-button"
                        onClick={() => setShowingAnswer((value) => !value)}
                        type="button"
                      >
                        <CheckCircle2 size={19} />
                      </button>
                      <button
                        aria-label="学习队列"
                        className="icon-button"
                        onClick={() => setQueueCollapsed((value) => !value)}
                        type="button"
                      >
                        {queueCollapsed ? <Menu size={19} /> : <X size={19} />}
                      </button>
                    </div>
                  </div>

                  <div className="card-prompt">
                    <div className="lesson-pill">{currentSentence.lessonTitle}</div>
                    <div className="card-word">{currentSentence.chinese}</div>
                  </div>

                  <form className="answer-form" onSubmit={submitAnswer}>
                    <div className="answer-blanks">
                      {answerParts.map((part, index) =>
                        part.type === "word" ? (
                          <input
                            aria-label={`填写第 ${part.wordIndex + 1} 个单词`}
                            className={`blank-input ${
                              wrongIndexes.has(part.wordIndex) ? "wrong" : ""
                            } ${
                              answerResult === "wrong" && !wrongIndexes.has(part.wordIndex)
                                ? "locked"
                                : ""
                            }`}
                            data-word-index={part.wordIndex}
                            disabled={
                              answerResult === "correct" ||
                              (answerResult === "wrong" && !wrongIndexes.has(part.wordIndex))
                            }
                            key={`word-${part.wordIndex}`}
                            onChange={(event) =>
                              setAnswerWords((current) => ({
                                ...current,
                                [part.wordIndex]: event.target.value,
                              }))
                            }
                            onKeyDown={(event) =>
                              handleAnswerKeyDown(event, part.wordIndex)
                            }
                            placeholder=""
                            spellCheck={false}
                            style={{
                              width: `${Math.max(86, part.text.length * 22 + 28)}px`,
                            }}
                            value={answerWords[part.wordIndex] ?? ""}
                          />
                        ) : (
                          <span className="fixed-text" key={`fixed-${index}`}>
                            {part.text}
                          </span>
                        ),
                      )}
                    </div>
                    <button className="study-submit-button" type="submit">
                      {answerResult === "correct"
                        ? "下一句"
                        : answerResult === "wrong"
                          ? "重新提交错词"
                          : "提交答案"}
                    </button>
                  </form>

                  {answerResult === "wrong" && (
                    <div className="answer-result wrong">
                      红色空格需要重填，其他单词已锁定。
                    </div>
                  )}

                  {answerResult === "correct" && (
                    <div className="answer-result correct">
                      答对了。先看一遍完整英文，再进入下一句。
                    </div>
                  )}

                  {ttsStatus && <div className="tts-status">{ttsStatus}</div>}

                  <div className={`card-back ${showingAnswer ? "" : "hidden"}`}>
                    <div className="answer-label">参考英文</div>
                    <div className="translation">{currentSentence.english}</div>
                    {currentSentence.phonetic && (
                      <div className="phonetic">{currentSentence.phonetic}</div>
                    )}
                    {currentSentence.note && <div className="note">{currentSentence.note}</div>}
                  </div>
                </>
              ) : (
                <div className="study-complete">
                  <div className="complete-mark">
                    <Trophy size={28} />
                  </div>
                  <h2>今日队列完成</h2>
                  <p>这门课程暂时没有到期句子。</p>
                  <div className="complete-stats">
                    <Metric icon={Target} label="今日完成" value={todayStats.reviewCount} />
                    <Metric icon={Flame} label="连续天数" value={streakDays} />
                    <Metric icon={ListChecks} label="当前待学" value={activeDueCount} accent />
                  </div>
                  <div className="complete-actions">
                    <button
                      className="primary-button"
                      onClick={() => activeCourse && openCourse(activeCourse.id)}
                      type="button"
                    >
                      <BookOpen size={17} />
                      返回课程
                    </button>
                    <button
                      className="ghost-button"
                      onClick={() => openView("home")}
                      type="button"
                    >
                      <Home size={17} />
                      回首页
                    </button>
                  </div>
                </div>
              )}
            </section>
            {!queueCollapsed && (
              <Surface
                action={
                  <button
                    className="small-action"
                    onClick={() => {
                      setSessionIndex(0);
                      resetAnswerState();
                    }}
                    type="button"
                  >
                    <RefreshCw size={15} />
                    重置
                  </button>
                }
                className="queue-panel"
                title="学习队列"
              >
                <QueueList items={studyQueue} />
              </Surface>
            )}
          </section>
        )}

        {view === "stats" && (
          <section className="stats-page">
            <section className="stats-hero">
              <div>
                <span className="soft-badge accent">学习表现</span>
                <h2>看趋势，不看压力。</h2>
                <p>统计页只回答：你坚持了多久、完成了多少、哪里还薄弱。</p>
              </div>
              <div className="streak-badge">
                <Flame size={22} />
                <strong>{streakDays}</strong>
                <span>连续天数</span>
              </div>
            </section>
            <div className="metric-grid">
              <Metric icon={Target} label="今日完成" value={todayStats.reviewCount} />
              <Metric icon={BookOpen} label="课程包" value={state.coursePacks.length} />
              <Metric icon={ListChecks} label="句子总数" value={state.sentences.length} />
              <Metric icon={Flame} label="连续天数" value={streakDays} accent />
            </div>
            <Surface title="课程进度">
              <div className="stats-list">
                {courseSummaries.length ? (
                  courseSummaries.map((summary) => (
                    <div className="stat-row" key={summary.course.id}>
                      <div>
                        <div className="row-title">{summary.course.name}</div>
                        <div className="row-subtitle">
                          {summary.masteredCount}/{summary.totalSentences} 已掌握 ·{" "}
                          {summary.dueCount} 待学
                        </div>
                      </div>
                      <ProgressBar value={summary.progressPercent} />
                    </div>
                  ))
                ) : (
                  <EmptyState
                    actionLabel="去创建"
                    icon={LibraryBig}
                    onAction={() => openView("courses")}
                    title="暂无统计"
                  />
                )}
              </div>
            </Surface>
          </section>
        )}
      </main>
    </div>
  );
}

function Metric({
  accent,
  icon: Icon,
  label,
  value,
}: {
  accent?: boolean;
  icon: LucideIcon;
  label: string;
  value: number;
}) {
  return (
    <article className={`metric ${accent ? "accent" : ""}`}>
      <div className="metric-icon">
        <Icon size={18} />
      </div>
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  );
}

function Surface({
  action,
  children,
  className = "",
  title,
}: {
  action?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
  title: string;
}) {
  return (
    <section className={`surface ${className}`}>
      <div className="surface-header">
        <h2>{title}</h2>
        {action}
      </div>
      {children}
    </section>
  );
}

function ProgressBar({ value }: { value: number }) {
  const safeValue = Math.max(0, Math.min(100, value));
  return (
    <div className="progress-track" aria-label={`进度 ${safeValue}%`}>
      <div className="progress-bar" style={{ width: `${safeValue}%` }} />
    </div>
  );
}

function CourseCard({
  onDelete,
  onOpen,
  onStudy,
  summary,
}: {
  onDelete: (coursePackId: string) => void;
  onOpen: (coursePackId: string) => void;
  onStudy: (coursePackId: string) => void;
  summary: CourseSummary;
}) {
  return (
    <article className="course-card">
      <div className="course-card-top">
        <div>
          <div className="row-title">{summary.course.name}</div>
          <div className="row-subtitle">
            {summary.lessonCount} 课时 · {summary.totalSentences} 句
          </div>
        </div>
        {summary.duplicateCount > 1 && <span className="warn-pill">同名</span>}
      </div>
      <ProgressBar value={summary.progressPercent} />
      <div className="course-card-meta">
        <span>{summary.dueCount} 待学</span>
        <span>{summary.progressPercent}%</span>
      </div>
      <div className="row-actions">
        <button
          className="small-button"
          onClick={() => onOpen(summary.course.id)}
          type="button"
        >
          查看
        </button>
        <button
          className="small-button primary-small"
          disabled={summary.dueCount === 0}
          onClick={() => onStudy(summary.course.id)}
          type="button"
        >
          学习
        </button>
        <button
          aria-label={`删除 ${summary.course.name}`}
          className="icon-button danger"
          onClick={() => onDelete(summary.course.id)}
          type="button"
        >
          <Trash2 size={16} />
        </button>
      </div>
    </article>
  );
}

function LessonGrid({
  activeTitle,
  lessons,
  onSelect,
}: {
  activeTitle: string | null;
  lessons: LessonSummary[];
  onSelect: (title: string) => void;
}) {
  if (!lessons.length) {
    return <EmptyState icon={Import} title="还没有课时" />;
  }

  return (
    <div className="lesson-grid">
      {lessons.map((lesson) => (
        <button
          className={`lesson-card ${lesson.title === activeTitle ? "active" : ""}`}
          key={lesson.title}
          onClick={() => onSelect(lesson.title)}
          type="button"
        >
          <div className="lesson-card-index">#{lesson.index}</div>
          <h3>{lesson.title}</h3>
          <p>
            {lesson.mastered}/{lesson.total} 已掌握 · {lesson.due} 待学
          </p>
          <ProgressBar value={lesson.progressPercent} />
        </button>
      ))}
    </div>
  );
}

function ImportPreview({ rows }: { rows: ImportRow[] }) {
  return (
    <div className="import-preview">
      {rows.map((row, index) => (
        <div className="preview-row" key={`${row.english}-${index}`}>
          <span>{row.lessonTitle}</span>
          <strong>{row.english}</strong>
          <em>{row.chinese}</em>
        </div>
      ))}
    </div>
  );
}

function VocabularyPreview({ items }: { items: VocabularyImportPreviewItem[] }) {
  return (
    <div className="import-preview">
      {items.map((item, index) => (
        <div className="preview-row" key={`${item.text}-${index}`}>
          <span>{item.partOfSpeech || item.difficulty || "词条"}</span>
          <strong>{item.text}</strong>
          <em>{item.primaryMeaning}</em>
        </div>
      ))}
    </div>
  );
}

function WordDetailGrid({ item }: { item: VocabularyItem }) {
  const details = [
    ["词根词缀", item.roots],
    ["同根词", item.wordFamily],
    ["近义词", item.synonyms],
    ["反义词", item.antonyms],
    ["形象记忆", item.memoryHint],
    ["场景标签", item.tags],
  ].filter(([, value]) => value);

  if (!details.length) {
    return null;
  }

  return (
    <div className="word-detail-grid">
      {details.map(([label, value]) => (
        <div className="word-detail" key={label}>
          <span>{label}</span>
          <strong>{value}</strong>
        </div>
      ))}
    </div>
  );
}

function VocabularyList({ items }: { items: VocabularyItem[] }) {
  if (!items.length) {
    return <EmptyState icon={BookOpen} title="暂无词条" />;
  }

  return (
    <div className="vocabulary-list">
      {items.map((item) => (
        <div className="vocabulary-row" key={item.id}>
          <div>
            <div className="row-title">{item.text}</div>
            <div className="row-subtitle">
              {item.primaryMeaning}
              {item.phonetic ? ` · ${item.phonetic}` : ""}
            </div>
          </div>
          <div className="word-score">
            <span>{item.familiarity}</span>
            <small>熟悉度</small>
          </div>
        </div>
      ))}
    </div>
  );
}

function SentenceList({
  onDelete,
  sentences,
}: {
  onDelete: (sentenceId: string) => void;
  sentences: SentenceItem[];
}) {
  if (!sentences.length) {
    return <EmptyState icon={BookOpen} title="当前课程还没有句子" />;
  }

  return (
    <div className="sentence-list">
      {sentences.map((sentence) => (
        <div className="sentence-row" key={sentence.id}>
          <div>
            <div className="row-title">{sentence.english}</div>
            <div className="row-subtitle">
              {sentence.lessonTitle} · {sentence.chinese}
              {sentence.phonetic ? ` · ${sentence.phonetic}` : ""}
            </div>
          </div>
          <div className="row-actions">
            <span className={`status-pill ${sentence.status}`}>
              {statusLabel(sentence.status)}
            </span>
            <button
              aria-label="删除句子"
              className="icon-button danger"
              onClick={() => onDelete(sentence.id)}
              type="button"
            >
              <Trash2 size={16} />
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}

function QueueList({ items }: { items: StudyQueueItem[] }) {
  if (!items.length) {
    return <EmptyState icon={CheckCircle2} title="当前队列已完成" />;
  }

  return (
    <div className="queue-list">
      {items.map((item) => (
        <div className={`queue-row ${item.isCurrent ? "active" : ""}`} key={item.id}>
          <div className="row-title">{item.english}</div>
          <div className="row-subtitle">
            {item.lessonTitle} · {item.chinese} · {statusLabel(item.status)}
          </div>
        </div>
      ))}
    </div>
  );
}

function EmptyState({
  actionLabel,
  icon: Icon,
  onAction,
  title,
}: {
  actionLabel?: string;
  icon: LucideIcon;
  onAction?: () => void;
  title: string;
}) {
  return (
    <div className="empty-state">
      <Icon size={24} />
      <span>{title}</span>
      {actionLabel && onAction && (
        <button className="ghost-button" onClick={onAction} type="button">
          {actionLabel}
        </button>
      )}
    </div>
  );
}

export default App;
