import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(new URL("../src/answerRules.ts", import.meta.url), "utf8");
const output = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;

const exportsObject = {};
const sandbox = {
  exports: exportsObject,
  module: { exports: exportsObject },
};
vm.runInNewContext(output, sandbox);
const rules = sandbox.module.exports;

const sentence = "Yeah, the economy is developing faster than most people expect.";
const parts = rules.parseAnswerParts(sentence);
const words = rules.getAnswerWords(sentence);

assert.deepEqual(JSON.parse(JSON.stringify(words)), [
  "Yeah",
  "the",
  "economy",
  "is",
  "developing",
  "faster",
  "than",
  "most",
  "people",
  "expect",
]);

const rebuilt = rules.buildAnswerFromWords(parts, {
  0: "yeah",
  1: "THE",
  2: "economy",
  3: "is",
  4: "developing",
  5: "faster",
  6: "than",
  7: "most",
  8: "people",
  9: "expect",
});
assert.equal(rebuilt, "yeah, THE economy is developing faster than most people expect.");

assert.deepEqual(JSON.parse(JSON.stringify([...rules.getWrongWordIndexes(words, {
  0: "yeah",
  1: "the",
  2: "economy",
  3: "is",
  4: "developing",
  5: "slowly",
  6: "than",
  7: "most",
  8: "people",
  9: "expect",
})])), [5]);

assert.deepEqual(JSON.parse(JSON.stringify([...rules.getWrongWordIndexes(words, {
  0: "YEAH",
  1: "THE",
  2: "ECONOMY",
  3: "IS",
  4: "DEVELOPING",
  5: "FASTER",
  6: "THAN",
  7: "MOST",
  8: "PEOPLE",
  9: "EXPECT",
})])), []);

console.log("answer rules ok");
