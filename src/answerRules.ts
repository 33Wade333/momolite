export type AnswerPart =
  | { type: "word"; text: string; wordIndex: number }
  | { type: "fixed"; text: string };

export function parseAnswerParts(value: string): AnswerPart[] {
  const matches = value.match(/[A-Za-z0-9]+(?:'[A-Za-z0-9]+)?|[^A-Za-z0-9]+/g) ?? [];
  let wordIndex = 0;
  return matches.map((text) => {
    if (/^[A-Za-z0-9]/.test(text)) {
      const part: AnswerPart = { type: "word", text, wordIndex };
      wordIndex += 1;
      return part;
    }
    return { type: "fixed", text };
  });
}

export function getAnswerWords(value: string) {
  return parseAnswerParts(value)
    .filter((part): part is Extract<AnswerPart, { type: "word" }> => part.type === "word")
    .map((part) => part.text);
}

export function normalizeAnswerWord(value: string) {
  return value.trim().toLocaleLowerCase();
}

export function buildAnswerFromWords(parts: AnswerPart[], words: Record<number, string>) {
  return parts
    .map((part) => (part.type === "word" ? words[part.wordIndex] ?? "" : part.text))
    .join("")
    .trim();
}

export function getWrongWordIndexes(expectedWords: string[], words: Record<number, string>) {
  const wrong = new Set<number>();
  expectedWords.forEach((expected, index) => {
    const actualWord = words[index] ?? "";
    if (normalizeAnswerWord(expected) !== normalizeAnswerWord(actualWord)) {
      wrong.add(index);
    }
  });
  return wrong;
}
