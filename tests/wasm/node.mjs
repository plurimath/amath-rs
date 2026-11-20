import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { StageParser } = require('../../wasm-node/amath.js');
const read = (path) => JSON.parse(readFileSync(new URL(path, import.meta.url), 'utf8'));

// The existing corpus smoke compares ordered hash pairs as maps; retain all
// array order, slice values and UTF-8/character offsets outside those hash pairs.
function normalize(value) {
  if (Array.isArray(value)) return value.map(normalize);
  if (value && typeof value === 'object') {
    const result = Object.fromEntries(Object.entries(value).map(([key, item]) => [key, normalize(item)]));
    if (Array.isArray(result.hash)) {
      result.hash.sort((a, b) => {
        const left = JSON.stringify(a[0]);
        const right = JSON.stringify(b[0]);
        return left < right ? -1 : left > right ? 1 : 0;
      });
    }
    return result;
  }
  return value;
}

const parser = new StageParser();
try {
  const corpus = read('../fixtures/raw-smoke.json')
    .filter((row) => !row.expected.refusal).slice(0, 3);
  assert.equal(corpus.length, 3);
  for (const row of corpus) {
    const actual = JSON.parse(parser.parseRaw(row.input.str, row.options?.decimal ?? '.'));
    assert.equal(actual.preprocessed, row.expected.preprocessed.str, row.id);
    assert.deepEqual(normalize(actual.tree), normalize(row.expected.tree), row.id);
  }

  const references = read('../../target/wasm-reference.json');
  assert.equal(references.length, 5);
  for (const row of references) {
    assert.equal(parser.parseRaw(row.input, row.decimal), row.raw, `${row.input} native raw bytes`);
    const model = parser.parse(row.input, row.decimal);
    assert.equal(model, row.model, `${row.input} native model bytes`);
    assert.equal(parser.serialize(model), row.serialized, `${row.input} native writer bytes`);
    assert.equal(JSON.parse(model).decimal, row.decimal);
  }

  for (const input of ['{', 'null', '[]']) {
    assert.throws(() => parser.serialize(input), () => true, `invalid model ${input}`);
  }
  assert.throws(() => parser.parse('x', 'bad'), (error) => String(error).includes('unsupported decimal'));
  assert.throws(() => parser.parseRaw('x', 'bad'), (error) => String(error).includes('unsupported decimal'));
  const refused = read('../fixtures/raw-smoke.json').find((row) => row.expected.refusal);
  if (refused) {
    assert.throws(() => parser.parseRaw(refused.input.str, refused.options?.decimal ?? '.'), () => true);
  }
  // Refusals must leave the reusable parser usable.
  assert.equal(parser.parse(references[0].input, references[0].decimal), references[0].model);
  console.log(`WASM smoke passed: ${corpus.length} corpus rows, ${references.length} native byte references, invalid inputs and reuse`);
} finally {
  parser.free();
}
