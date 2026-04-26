import { invoke } from "@tauri-apps/api/core";
import { FormEvent, KeyboardEvent, useEffect, useMemo, useState } from "react";
import {
  buildAnswerFromWords,
  getAnswerWords,
  getWrongWordIndexes,
  parseAnswerParts,
} from "./answerRules";
import "./App.css";

type View = "dashboard" | "courses" | "import" | "study" | "stats";
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

const DEFAULT_LESSON = "默认课时";

const views: Record<View, { title: string; subtitle: string }> = {
  dashboard: { title: "首页", subtitle: "今日任务和学习进度" },
  courses: { title: "课程包", subtitle: "管理课程、课时和句子" },
  import: { title: "导入", subtitle: "一次导入一个课程包里的多节课" },
  study: { title: "中译英", subtitle: "看中文，听英文，重打错误词" },
  stats: { title: "统计", subtitle: "课程进度和掌握情况" },
};

const ratingRules: Record<
  Rating,
  { label: string; minutes: number; status: SentenceStatus }
> = {
  again: { label: "不认识", minutes: 10, status: "learning" },
  hard: { label: "模糊", minutes: 1440, status: "learning" },
  good: { label: "认识", minutes: 4320, status: "learning" },
  easy: { label: "熟悉", minutes: 10080, status: "mastered" },
};

const initialState: AppState = {
  activeCoursePackId: "",
  coursePacks: [],
  sentences: [],
  reviews: [],
  stats: {},
};

const mainNavViews: View[] = ["dashboard", "courses", "stats"];

function uid(prefix: string) {
  return `${prefix}_${Date.now()}_${Math.random().toString(16).slice(2)}`;
}

function todayKey() {
  return new Date().toISOString().slice(0, 10);
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
    /^第\s*\d+\s*[课天]/.test(normalized) ||
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
    const expected = cursor.toISOString().slice(0, 10);
    if (day !== expected) break;
    streak += 1;
    cursor.setDate(cursor.getDate() - 1);
  }
  return streak;
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

function App() {
  const [state, setState] = useState<AppState>(initialState);
  const [view, setView] = useState<View>("dashboard");
  const [courseName, setCourseName] = useState("");
  const [sentenceDraft, setSentenceDraft] = useState({
    english: "",
    chinese: "",
    phonetic: "",
    note: "",
  });
  const [importText, setImportText] = useState("");
  const [sessionIndex, setSessionIndex] = useState(0);
  const [showingAnswer, setShowingAnswer] = useState(false);
  const [answerWords, setAnswerWords] = useState<Record<number, string>>({});
  const [answerResult, setAnswerResult] = useState<AnswerResult>("idle");
  const [wrongIndexes, setWrongIndexes] = useState<Set<number>>(new Set());
  const [lastSubmittedAnswer, setLastSubmittedAnswer] = useState("");
  const [importMessage, setImportMessage] = useState(
    "推荐格式：## 第1课 标题，然后每行写 英文=中文。也兼容 英文=中文=音标。",
  );
  const [databasePath, setDatabasePath] = useState("");
  const [databaseError, setDatabaseError] = useState("");
  const [databaseHydrated, setDatabaseHydrated] = useState(false);
  const [queueCollapsed, setQueueCollapsed] = useState(true);
  const [ttsStatus, setTtsStatus] = useState("");

  useEffect(() => {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) {
      return;
    }

    window.speechSynthesis.getVoices();
  }, []);

  async function speakEnglish(text: string) {
    if (!text) return;

    try {
      setTtsStatus("Piper 离线发音");
      const response = await invoke<TtsResponse>("synthesize_piper_tts", {
        request: { input: text },
      });
      const audio = new Audio(`data:audio/wav;base64,${response.audioBase64}`);
      await audio.play();
      setTtsStatus(response.cached ? "Piper 离线发音（缓存）" : response.engine);
    } catch (error) {
      setTtsStatus("Piper 发音失败，已回退本机语音");
      speakWithSystemVoice(text);
      console.warn(error);
    }
  }

  function applyLoadedState(loadedState: AppState) {
    if (loadedState.coursePacks.length > 0) {
      setState(loadedState);
    }
    setDatabaseHydrated(true);
  }

  useEffect(() => {
    invoke<string>("init_database")
      .then((path) => {
        setDatabasePath(path);
        setDatabaseError("");
        return invoke<AppState>("load_app_state");
      })
      .then((loadedState) => {
        applyLoadedState(loadedState);
      })
      .catch((error) => {
        setDatabaseError(String(error));
        setDatabaseHydrated(true);
      });
  }, []);

  const activeCourse = useMemo(
    () =>
      state.coursePacks.find((course) => course.id === state.activeCoursePackId) ??
      state.coursePacks[0],
    [state.activeCoursePackId, state.coursePacks],
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

  const session = useMemo(
    () => getDueSentences(state.sentences, activeCourse?.id).slice(0, 20),
    [activeCourse?.id, state.sentences],
  );

  const currentSentence = session[sessionIndex];
  const answerParts = currentSentence ? parseAnswerParts(currentSentence.english) : [];
  const expectedWords = currentSentence ? getAnswerWords(currentSentence.english) : [];
  const todayStats = state.stats[todayKey()] ?? {
    newCount: 0,
    reviewCount: 0,
    studyMinutes: 0,
  };
  const activeDueCount = activeCourse
    ? getDueSentences(state.sentences, activeCourse.id).length
    : 0;
  const importRows = useMemo(() => parseImport(importText), [importText]);

  useEffect(() => {
    if (view === "study" && currentSentence) {
      speakEnglish(currentSentence.english);
    }
  }, [currentSentence?.id, view]);

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
    if (next === "study") {
      setSessionIndex(0);
      resetAnswerState();
    }
  }

  async function persistCoursePack(coursePack: CoursePack) {
    if (!databasePath || !databaseHydrated) return;

    await invoke("create_course_pack", {
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

    await invoke("create_lesson", { lesson });
  }

  async function persistSentence(sentence: SentenceItem, lessonId: string) {
    if (!databasePath || !databaseHydrated) return;

    await invoke("create_sentence", {
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

  async function createCourse(event: FormEvent) {
    event.preventDefault();
    const name = courseName.trim();
    if (!name) return;

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
  }

  async function deleteCourse(coursePackId: string) {
    if (databasePath && databaseHydrated) {
      try {
        await invoke("delete_course_pack", { coursePackId });
        setDatabaseError("");
      } catch (error) {
        setDatabaseError(String(error));
        return;
      }
    }

    updateState((current) => {
      const coursePacks = current.coursePacks.filter(
        (course) => course.id !== coursePackId,
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
  }

  async function deleteSentence(sentenceId: string) {
    if (databasePath && databaseHydrated) {
      try {
        await invoke("delete_sentence", { sentenceId });
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

    const createdAt = new Date().toISOString();
    const lesson: NewLesson = {
      id: `lesson_${activeCourse.id}_default`,
      coursePackId: activeCourse.id,
      title: DEFAULT_LESSON,
      sortOrder: 0,
      createdAt,
    };
    const sentence: SentenceItem = {
      id: uid("sentence"),
      coursePackId: activeCourse.id,
      lessonTitle: DEFAULT_LESSON,
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
      await persistCoursePack(activeCourse);
      await persistLesson(lesson);
      await persistSentence(sentence, lesson.id);
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

    const now = new Date().toISOString();
    const lessonTitles = [...new Set(importRows.map((row) => row.lessonTitle))];
    const lessons = lessonTitles.map<NewLesson>((title, index) => ({
      id: uid("lesson"),
      coursePackId: activeCourse.id,
      title,
      sortOrder: index,
      createdAt: now,
    }));
    const lessonIds = new Map(lessons.map((lesson) => [lesson.title, lesson.id]));
    const sentences = importRows.map<SentenceItem>((row) => ({
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

    try {
      await persistCoursePack(activeCourse);
      for (const lesson of lessons) {
        await persistLesson(lesson);
      }
      for (const sentence of sentences) {
        await persistSentence(
          sentence,
          lessonIds.get(sentence.lessonTitle) ?? lessons[0]?.id ?? "",
        );
      }
      setDatabaseError("");
    } catch (error) {
      setDatabaseError(String(error));
      setImportMessage(`导入失败：${String(error)}`);
      return;
    }

    updateState((current) => ({
      ...current,
      sentences: [...current.sentences, ...sentences],
    }));
    setImportText("");
    setImportMessage(`已导入 ${sentences.length} 句，识别到 ${new Set(sentences.map((item) => item.lessonTitle)).size} 个课时`);
  }

  async function seedDemo() {
    const coursePack: CoursePack = {
      id: uid("course"),
      name: "经济讨论句子课",
      language: "en",
      dailyNewTarget: 20,
      createdAt: new Date().toISOString(),
    };
    const rows = [
      [
        "第1课：经济讨论",
        "The market is showing strong growth this quarter.",
        "市场这个季度展现出强劲的增长。",
        "[ˈmɑːrkɪt]",
      ],
      [
        "第1课：经济讨论",
        "Yeah, the economy is developing faster than most people expect.",
        "是的，经济发展比大多数人预期的要快。",
        "[ˌiːˈkɑːnəmi]",
      ],
      [
        "第2课：经济观点",
        "What's your view on the current economic situation?",
        "你对当前的经济形势有什么看法？",
        "[vjuː]",
      ],
      [
        "第2课：经济观点",
        "The interest rate is the major factor affecting business growth.",
        "利率是影响业务增长的主要因素。",
        "[ˈɪntrəst]",
      ],
      [
        "第3课：金融行业",
        "Financial industry professionals say profit margins are improving.",
        "金融行业专业人士表示利润率正在改善。",
        "[faɪˈnænʃl]",
      ],
      [
        "第3课：金融行业",
        "But the cost of operation is still very high.",
        "但运营成本仍然很高。",
        "[kɔːst]",
      ],
    ];

    const createdAt = new Date().toISOString();
    const lessonTitles = [...new Set(rows.map(([lessonTitle]) => lessonTitle))];
    const lessons = lessonTitles.map<NewLesson>((title, index) => ({
      id: uid("lesson"),
      coursePackId: coursePack.id,
      title,
      sortOrder: index,
      createdAt,
    }));
    const lessonIds = new Map(lessons.map((lesson) => [lesson.title, lesson.id]));
    const sentences = rows.map<SentenceItem>(
      ([lessonTitle, english, chinese, phonetic]) => ({
        id: uid("sentence"),
        coursePackId: coursePack.id,
        lessonTitle,
        english,
        chinese,
        phonetic,
        note: "",
        status: "new",
        favorite: false,
        showCount: 0,
        reviewCount: 0,
        errorCount: 0,
        nextReviewAt: null,
        createdAt,
      }),
    );

    try {
      await persistCoursePack(coursePack);
      for (const lesson of lessons) {
        await persistLesson(lesson);
      }
      for (const sentence of sentences) {
        await persistSentence(
          sentence,
          lessonIds.get(sentence.lessonTitle) ?? lessons[0]?.id ?? "",
        );
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
    setSessionIndex(0);
    resetAnswerState();
  }

  function evaluateAnswer() {
    if (!currentSentence) return;

    const actual = buildAnswerFromWords(answerParts, answerWords);
    setLastSubmittedAnswer(actual);

    const wrong = getWrongWordIndexes(expectedWords, answerWords);

    if (!wrong.size) {
      setAnswerResult("correct");
      setWrongIndexes(new Set());
      setShowingAnswer(false);
      window.setTimeout(() => applyRating("good", true), 260);
      return;
    }

    setAnswerResult("wrong");
    setWrongIndexes(wrong);
    setShowingAnswer(false);
    window.setTimeout(() => focusNextEditableAnswerInput(Math.min(...wrong)), 0);
  }

  function submitAnswer(event: FormEvent) {
    event.preventDefault();
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
            reviewCount: today.reviewCount + (wasNew ? 0 : 1),
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
      invoke("record_review", { review }).catch((error) => {
        setDatabaseError(String(error));
      });
    }

    setSessionIndex((index) => Math.min(index + 1, session.length - 1));
    resetAnswerState();
  }

  function handleAnswerKeyDown(
    event: KeyboardEvent<HTMLInputElement>,
    wordIndex: number,
  ) {
    if (event.key === " ") {
      event.preventDefault();
      focusNextEditableAnswerInput(wordIndex + 1);
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
  }

  const dueCount = getDueSentences(state.sentences).length;
  const streakDays = getStreakDays(state.stats);
  const lessons = new Set(activeSentences.map((sentence) => sentence.lessonTitle));
  const lessonSummaries = [...lessons].map((title, index) => {
    const lessonSentences = activeSentences.filter(
      (sentence) => sentence.lessonTitle === title,
    );
    const mastered = lessonSentences.filter(
      (sentence) => sentence.status === "mastered",
    ).length;
    return {
      title,
      index: index + 1,
      total: lessonSentences.length,
      mastered,
      due: lessonSentences.filter((sentence) =>
        getDueSentences([sentence]).length > 0,
      ).length,
    };
  });

  return (
    <div className={`app-shell ${view === "study" ? "study-mode" : ""}`}>
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">M</div>
          <div>
            <div className="brand-title">MomoLite</div>
            <div className="brand-subtitle">Local Study Lab</div>
          </div>
        </div>
        <nav className="nav">
          {mainNavViews.map((item) => (
            <button
              className={`nav-item ${view === item ? "active" : ""}`}
              key={item}
              onClick={() => openView(item)}
              type="button"
            >
              {views[item].title}
            </button>
          ))}
        </nav>
      </aside>

      <main className="workspace">
        <header className="topbar">
          <div>
            <h1>{views[view].title}</h1>
            <p>{views[view].subtitle}</p>
            <p className="database-status">
              {databasePath
                ? `SQLite 已就绪：${databasePath}`
                : databaseError
                  ? `SQLite 初始化失败：${databaseError}`
                  : "SQLite 正在初始化..."}
            </p>
          </div>
          <div className="topbar-actions">
            <select
              aria-label="选择课程包"
              onChange={(event) => {
                updateState((current) => ({
                  ...current,
                  activeCoursePackId: event.target.value,
                }));
                setSessionIndex(0);
                resetAnswerState();
              }}
              value={activeCourse?.id ?? ""}
            >
              {state.coursePacks.map((course) => (
                <option key={course.id} value={course.id}>
                  {course.name}
                </option>
              ))}
            </select>
            <button
              className="ghost-button"
              onClick={() => openView("import")}
              type="button"
            >
              ????
            </button>
            {!state.coursePacks.length && (
              <button className="ghost-button" onClick={seedDemo} type="button">
                ????
              </button>
            )}
          </div>
        </header>

        {view === "dashboard" && (
          <section>
            <div className="metric-grid">
              <Metric label="今日新学" value={todayStats.newCount} />
              <Metric label="今日复习" value={todayStats.reviewCount} />
              <Metric label="待复习" value={dueCount} />
              <Metric label="连续天数" value={streakDays} accent />
            </div>
            <div className="dashboard-layout">
              <Panel
                action={
                  <button
                    className="primary-button"
                    onClick={() => openView("study")}
                    type="button"
                  >
                    开始学习
                  </button>
                }
                title="今日队列"
              >
                <QueueList
                  sentences={getDueSentences(state.sentences, activeCourse?.id).slice(0, 8)}
                />
              </Panel>
              <Panel
                action={
                  <button
                    className="ghost-button"
                    onClick={() => openView("courses")}
                    type="button"
                  >
                    管理
                  </button>
                }
                title="最近课程包"
              >
                <CourseList
                  activeCoursePackId={activeCourse?.id}
                  coursePacks={state.coursePacks.slice(0, 5)}
                  onDelete={deleteCourse}
                  onSelect={(coursePackId) =>
                    updateState((current) => ({
                      ...current,
                      activeCoursePackId: coursePackId,
                    }))
                  }
                  sentences={state.sentences}
                />
              </Panel>
            </div>
          </section>
        )}

        {view === "courses" && (
          <section className="split-layout">
            <Panel title="课程包">
              <form className="inline-form" onSubmit={createCourse}>
                <input
                  onChange={(event) => setCourseName(event.target.value)}
                  placeholder="新课程包名称"
                  value={courseName}
                />
                <button className="primary-button" type="submit">
                  新建
                </button>
              </form>
              <CourseList
                activeCoursePackId={activeCourse?.id}
                coursePacks={state.coursePacks}
                onDelete={deleteCourse}
                onSelect={(coursePackId) =>
                  updateState((current) => ({
                    ...current,
                    activeCoursePackId: coursePackId,
                  }))
                }
                sentences={state.sentences}
              />
            </Panel>
            <Panel
              action={
                <span className="muted">
                  {activeSentences.length} 句 · {lessons.size} 课
                </span>
              }
              title="句子"
            >
              <div className="course-hero">
                <div>
                  <div className="row-title">
                    {activeCourse?.name ?? "请先创建课程"}
                  </div>
                  <div className="row-subtitle">
                    {lessonSummaries.length} 课 · {activeSentences.length} 句 · 自动保存
                  </div>
                </div>
                <button
                  className="primary-button"
                  onClick={() => openView("import")}
                  type="button"
                >
                  批量导入
                </button>
              </div>
              <LessonGrid lessons={lessonSummaries} />
              <form className="word-form" onSubmit={addSentence}>
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
                  placeholder="重点音标，可不填"
                  value={sentenceDraft.phonetic}
                />
                <input
                  onChange={(event) =>
                    setSentenceDraft({ ...sentenceDraft, note: event.target.value })
                  }
                  placeholder="备注"
                  value={sentenceDraft.note}
                />
                <button className="primary-button" type="submit">
                  添加句子
                </button>
              </form>
              <SentenceList
                onDelete={deleteSentence}
                sentences={activeSentences}
              />
            </Panel>
          </section>
        )}

        {view === "import" && (
          <Panel
            action={
              <button
                className="primary-button"
                onClick={importSentences}
                type="button"
              >
                导入当前课程包
              </button>
            }
            className="import-panel"
            title="批量导入"
          >
            <div className="format-note">
              <strong>推荐课时分隔符：</strong>
              <code>## 第1课 经济讨论</code>
              <span>每个分隔符后面的句子会归入这一课。句子格式用：</span>
              <code>英文=中文</code>
            </div>
            <textarea
              onChange={(event) => {
                setImportText(event.target.value);
                setImportMessage(
                  "推荐格式：## 第1课 标题，然后每行写 英文=中文。也兼容 英文=中文=音标。",
                );
              }}
              placeholder={`## 第1课 经济讨论\nThe market is showing strong growth this quarter.=市场这个季度展现出强劲的增长。\nYeah, the economy is developing faster than most people expect.=是的，经济发展比大多数人预期的要快。\n\n## 第2课 商务成本\nThe interest rate is the major factor affecting business growth.=利率是影响业务增长的主要因素。\nBut the cost of operation is still very high.=但运营成本仍然很高。`}
              spellCheck={false}
              value={importText}
            />
            <div className="import-footer">
              <span className="muted">{importMessage}</span>
              <span className="muted">预览 {importRows.length} 句</span>
            </div>
            {importRows.length > 0 && (
              <div className="preview-table">
                <table>
                  <thead>
                    <tr>
                      <th>课时</th>
                      <th>英文</th>
                      <th>中文</th>
                      <th>音标</th>
                    </tr>
                  </thead>
                  <tbody>
                    {importRows.slice(0, 30).map((row, index) => (
                      <tr key={`${row.english}-${index}`}>
                        <td>{row.lessonTitle}</td>
                        <td>{row.english}</td>
                        <td>{row.chinese}</td>
                        <td>{row.phonetic}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </Panel>
        )}

        {view === "study" && (
          <section className={`study-layout ${queueCollapsed ? "queue-collapsed" : ""}`}>
            <section className="study-card">
              {currentSentence ? (
                <>
                  <div className="card-meta">
                    <button
                      aria-label="toggle queue"
                      className="icon-button"
                      onClick={() => setQueueCollapsed((value) => !value)}
                      type="button"
                    >
                      {queueCollapsed ? "=" : "x"}
                    </button>
                    <button
                      className="icon-text-button"
                      onClick={() => openView("dashboard")}
                      type="button"
                    >
                      主页
                    </button>
                    <span className="study-progress">
                      {Math.min(sessionIndex + 1, session.length)} / {session.length}
                    </span>
                    <div className="study-tools">
                      <button
                        className="icon-text-button"
                        onClick={() => speakEnglish(currentSentence.english)}
                        type="button"
                      >
                        朗读
                      </button>
                      <button
                        className="icon-text-button"
                        onClick={() => setShowingAnswer((value) => !value)}
                        type="button"
                      >
                        答案
                      </button>
                    </div>
                  </div>

                  <div className="card-prompt">
                    <span className="prompt-label">
                      看中文，同时听英文朗读。输入英文后提交。
                    </span>
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
                            placeholder={"_".repeat(Math.max(3, part.text.length))}
                            spellCheck={false}
                            style={{
                              width: `${Math.max(96, part.text.length * 28 + 30)}px`,
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
                      {answerResult === "wrong" ? "重新提交错误词" : "提交答案"}
                    </button>
                  </form>

                  {answerResult === "wrong" && (
                    <div className="correction-panel">
                      <div className="answer-result wrong">
                        答案不一致。红色空格需要重新填写，其他单词已锁定。
                      </div>
                    </div>
                  )}

                  {ttsStatus && <div className="tts-status">{ttsStatus}</div>}

                  <div className={`card-back ${showingAnswer ? "" : "hidden"}`}>
                    <div className="answer-label">参考英文</div>
                    <div className="translation">{currentSentence.english}</div>
                    <div className="phonetic">{currentSentence.phonetic}</div>
                    <div className="note">{currentSentence.note}</div>
                  </div>

                  {answerResult === "correct" && (
                    <div className="answer-result correct">回答正确，进入下一句...</div>
                  )}
                </>
              ) : (
                <div className="study-complete">
                  <div className="complete-mark">✓</div>
                  <h2>今日队列已完成</h2>
                  <p>当前课程包没有到期句子，可以继续导入新课或查看学习进度。</p>
                  <div className="complete-stats">
                    <Metric label="今日新学" value={todayStats.newCount} />
                    <Metric label="今日复习" value={todayStats.reviewCount} />
                    <Metric label="当前待学" value={activeDueCount} accent />
                  </div>
                  <div className="complete-actions">
                    <button
                      className="primary-button"
                      onClick={() => openView("import")}
                      type="button"
                    >
                      导入更多
                    </button>
                    <button
                      className="ghost-button"
                      onClick={() => openView("stats")}
                      type="button"
                    >
                      查看统计
                    </button>
                  </div>
                </div>
              )}
            </section>
            {!queueCollapsed && (
            <Panel
              action={
                <button
                  className="ghost-button"
                  onClick={() => {
                    setSessionIndex(0);
                    resetAnswerState();
                  }}
                  type="button"
                >
                  刷新
                </button>
              }
              title="学习队列"
            >
              <QueueList sentences={session} />
            </Panel>
            )}
          </section>
        )}

        {view === "stats" && (
          <Panel title="课程包进度">
            <div className="stats-list">
              {state.coursePacks.length ? (
                state.coursePacks.map((course) => {
                  const sentences = state.sentences.filter(
                    (sentence) => sentence.coursePackId === course.id,
                  );
                  const mastered = sentences.filter(
                    (sentence) => sentence.status === "mastered",
                  ).length;
                  const percent = sentences.length
                    ? Math.round((mastered / sentences.length) * 100)
                    : 0;
                  return (
                    <div className="stat-row" key={course.id}>
                      <div className="row-title">{course.name}</div>
                      <div className="row-subtitle">
                        {mastered} / {sentences.length} 已掌握
                      </div>
                      <div className="progress-track">
                        <div
                          className="progress-bar"
                          style={{ width: `${percent}%` }}
                        />
                      </div>
                    </div>
                  );
                })
              ) : (
                <Empty text="暂无统计" />
              )}
            </div>
          </Panel>
        )}
      </main>
    </div>
  );
}

function Metric({
  accent,
  label,
  value,
}: {
  accent?: boolean;
  label: string;
  value: number;
}) {
  return (
    <article className={`metric ${accent ? "accent" : ""}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  );
}

function Panel({
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
    <section className={`panel ${className}`}>
      <div className="panel-header">
        <h2>{title}</h2>
        {action}
      </div>
      {children}
    </section>
  );
}

function CourseList({
  activeCoursePackId,
  coursePacks,
  onDelete,
  onSelect,
  sentences,
}: {
  activeCoursePackId?: string;
  coursePacks: CoursePack[];
  onDelete: (coursePackId: string) => void;
  onSelect: (coursePackId: string) => void;
  sentences: SentenceItem[];
}) {
  if (!coursePacks.length) return <Empty text="创建第一个课程包" />;

  return (
    <div className="deck-list">
      {coursePacks.map((course) => {
        const courseSentences = sentences.filter(
          (sentence) => sentence.coursePackId === course.id,
        );
        const due = getDueSentences(sentences, course.id).length;
        const lessonCount = new Set(
          courseSentences.map((sentence) => sentence.lessonTitle),
        ).size;

        return (
          <div
            className={`deck-row ${course.id === activeCoursePackId ? "active" : ""}`}
            key={course.id}
          >
            <div>
              <div className="row-title">{course.name}</div>
              <div className="row-subtitle">
                {lessonCount} 课 · {courseSentences.length} 句 · {due} 句待学
              </div>
            </div>
            <div className="row-actions">
              <button
                className="small-button"
                onClick={() => onSelect(course.id)}
                type="button"
              >
                选择
              </button>
              <button
                className="small-button danger"
                onClick={() => onDelete(course.id)}
                type="button"
              >
                删除
              </button>
            </div>
          </div>
        );
      })}
    </div>
  );
}

function LessonGrid({
  lessons,
}: {
  lessons: Array<{
    title: string;
    index: number;
    total: number;
    mastered: number;
    due: number;
  }>;
}) {
  if (!lessons.length) {
    return (
      <div className="lesson-empty">
        <strong>还没有课时</strong>
        <span>先点击“导入课程”，按 `## 第一天` 这样的分隔符批量导入。</span>
      </div>
    );
  }

  return (
    <div className="lesson-grid">
      {lessons.map((lesson) => (
        <article className="lesson-card" key={lesson.title}>
          <div className="lesson-card-index">#{lesson.index}</div>
          <h3>{lesson.title}</h3>
          <p>
            {lesson.mastered}/{lesson.total} 已掌握 · {lesson.due} 待学
          </p>
          <span>最近学习</span>
        </article>
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
  if (!sentences.length) return <Empty text="当前课程包还没有句子" />;

  return (
    <div className="word-list">
      {sentences.map((sentence) => (
        <div className="word-row" key={sentence.id}>
          <div>
            <div className="row-title">{sentence.english}</div>
            <div className="row-subtitle">
              {sentence.lessonTitle} · {sentence.chinese}
              {sentence.phonetic ? ` · ${sentence.phonetic}` : ""}
            </div>
          </div>
          <div className="row-actions">
            <span className="row-subtitle">{sentence.status}</span>
            <button
              className="small-button danger"
              onClick={() => onDelete(sentence.id)}
              type="button"
            >
              删除
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}

function QueueList({ sentences }: { sentences: SentenceItem[] }) {
  if (!sentences.length) return <Empty text="当前课程包暂无待学习句子" />;

  return (
    <div className="queue-list">
      {sentences.map((sentence) => (
        <div className="queue-row" key={sentence.id}>
          <div className="row-title">{sentence.english}</div>
          <div className="row-subtitle">
            {sentence.lessonTitle} · {sentence.chinese} · {sentence.status}
          </div>
        </div>
      ))}
    </div>
  );
}

function Empty({ text }: { text: string }) {
  return (
    <div className="queue-row">
      <span className="muted">{text}</span>
    </div>
  );
}

export default App;
