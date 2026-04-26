import { invoke } from "@tauri-apps/api/core";
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

type View = "home" | "courses" | "courseDetail" | "study" | "stats";
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

const viewCopy: Record<View, { title: string; subtitle: string }> = {
  home: { title: "首页", subtitle: "今天继续一小步，英语句子更顺一点。" },
  courses: { title: "课程包", subtitle: "整理你的句子材料和训练路径。" },
  courseDetail: { title: "课程详情", subtitle: "课时、导入和继续学习都在这里。" },
  study: { title: "中译英训练", subtitle: "看中文，听英文，补全原句。" },
  stats: { title: "统计", subtitle: "查看课程进度和最近学习表现。" },
};

const navItems: Array<{ view: View; label: string; icon: LucideIcon }> = [
  { view: "home", label: "首页", icon: Home },
  { view: "courses", label: "课程包", icon: LibraryBig },
  { view: "stats", label: "统计", icon: BarChart3 },
];

const ratingRules: Record<Rating, { minutes: number; status: SentenceStatus }> = {
  again: { minutes: 10, status: "learning" },
  hard: { minutes: 1440, status: "learning" },
  good: { minutes: 4320, status: "mastered" },
  easy: { minutes: 10080, status: "mastered" },
};

const initialState: AppState = {
  activeCoursePackId: "",
  coursePacks: [],
  sentences: [],
  reviews: [],
  stats: {},
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
  const nextCourse =
    courseSummaries.find((summary) => summary.course.id === lastStudy?.coursePackId) ??
    courseSummaries.find((summary) => summary.dueCount > 0) ??
    courseSummaries[0];

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
    <div className={`app-shell ${view === "study" ? "study-mode" : ""}`}>
      {view !== "study" && (
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
        {view !== "study" && (
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
            SQLite 连接异常：{databaseError}
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
            <section className="home-hero">
              <div className="hero-copy">
                <div className="eyebrow">
                  <Sparkles size={16} />
                  今日训练
                </div>
                <h2>把中文提示变成自然英文。</h2>
                <p>
                  {nextCourse
                    ? `下一组：${nextCourse.course.name}`
                    : "先创建一个课程包，再导入你的句子材料。"}
                </p>
                <div className="hero-actions">
                  <button
                    className="primary-button large"
                    disabled={!nextCourse || nextCourse.dueCount === 0}
                    onClick={() => openStudy(nextCourse?.course.id)}
                    type="button"
                  >
                    <Play size={18} />
                    开始训练
                  </button>
                  <button
                    className="ghost-button large"
                    onClick={() =>
                      nextCourse ? openCourse(nextCourse.course.id) : openView("courses")
                    }
                    type="button"
                  >
                    <LibraryBig size={18} />
                    {nextCourse ? "查看课程" : "创建课程"}
                  </button>
                </div>
              </div>
              <div className="goal-card">
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
                </div>
              </div>
            </section>

            <div className="metric-grid">
              <Metric icon={Target} label="今日完成" value={todayStats.reviewCount} />
              <Metric icon={ListChecks} label="待学习" value={dueCount} />
              <Metric icon={Flame} label="连续天数" value={streakDays} accent />
              <Metric icon={Trophy} label="已掌握" value={activeSummary?.masteredCount ?? 0} />
            </div>

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

        {view === "courses" && (
          <section className="courses-page">
            <Surface title="新建课程包">
              <form className="create-course-form" onSubmit={createCourse}>
                <input
                  onChange={(event) => setCourseName(event.target.value)}
                  placeholder="例如：商务英语 30 天"
                  value={courseName}
                />
                <button className="primary-button" type="submit">
                  <Plus size={17} />
                  新建
                </button>
              </form>
            </Surface>

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
