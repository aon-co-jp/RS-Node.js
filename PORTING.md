
## 2026-11-20 追記: タイマーとマイクロタスク / Timers and microtasks
**日本語**: `setTimeout`/`setInterval`/`setImmediate`/`clear*`/`queueMicrotask`/`process.nextTick` を実装(`src/engine.rs` の PRELUDE と、評価後の満期順ループ)。`tests/compat/c.js` の出力が本物の Node(v26.5.0)と一致することを確認。落とし穴: interval の再登録で id を変えると `clearInterval` が効かず無限ループになる(id は据え置き、並び順は別の通し番号 `seq`)。次: Buffer、非同期 fs、events/util/os/http、ESM、`process.exit`。
**English**: `setTimeout`/`setInterval`/`setImmediate`/`clear*`/`queueMicrotask`/`process.nextTick` are implemented (the PRELUDE in `src/engine.rs` plus a due-order loop after evaluation). The output of `tests/compat/c.js` matches real Node (v26.5.0). Pitfall: changing a timer's id when an interval re-arms breaks `clearInterval` and loops forever (keep the id; order by a separate `seq`). Next: Buffer, async fs, events/util/os/http, ESM, `process.exit`.

## 2026-11-20 追記2: process.exit / process.exit
**日本語**: `process.exit(code)` を実装(ネイティブ `__exit`、標準出力を流してから終了、残りのタイマーは実行しない)。`tests/compat/d.js` が本物の Node と同じ出力・終了コード 3。次: Buffer、非同期 fs、events/util/os/http、ESM。
**English**: `process.exit(code)` is implemented (native `__exit`; flushes stdout, then exits without running pending timers). `tests/compat/d.js` matches real Node (same output, exit code 3). Next: Buffer, async fs, events/util/os/http, ESM.

## 2026-11-20 追記3: events と util / events and util
**日本語**: `require('events')`(EventEmitter: on/once/off/emit/prependListener/listenerCount/removeAllListeners)と `require('util')`(format・promisify・inherits・isDeepStrictEqual)、`node:` 接頭辞に対応。`tests/compat/e.js` が本物の Node と同じ出力。`util.inspect` は未実装。次: Buffer、非同期 fs、os/http、ESM。
**English**: `require('events')` (EventEmitter), `require('util')` (format, promisify, inherits, isDeepStrictEqual) and the `node:` prefix are supported; `tests/compat/e.js` matches real Node. `util.inspect` is not implemented. Next: Buffer, async fs, os/http, ESM.

## 2026-11-20 追記4: Buffer・非同期 fs・os / Buffer, async fs, os
**日本語**: `Buffer`(from: utf8/hex/base64・alloc・concat・isBuffer・byteLength、toString・toJSON・equals・slice)、非同期 `fs`(readFile/writeFile/exists、`fs.promises`、`require('fs/promises')`)、`os`(EOL・platform・tmpdir・homedir)に対応。`tests/compat/f.js`・`g.js` が本物の Node と同じ出力。落とし穴: Boa には `String.prototype.substr` が無い(`slice` を使う)、Rust の raw 文字列に JS の `'\n'` を書くときは実際の改行にならないよう注意。未実装: `readFileSync` が Buffer を返さない(常に文字列)、http、ESM、`util.inspect`。
**English**: `Buffer` (from: utf8/hex/base64, alloc, concat, isBuffer, byteLength, toString, toJSON, equals, slice), async `fs` (readFile/writeFile/exists, `fs.promises`, `require('fs/promises')`) and `os` (EOL, platform, tmpdir, homedir) are supported; `tests/compat/f.js` and `g.js` match real Node. Pitfalls: Boa has no `String.prototype.substr` (use `slice`), and mind newline escapes when writing JS inside a Rust raw string. Not yet: `readFileSync` returning a Buffer (it always returns a string), http, ESM, `util.inspect`.
