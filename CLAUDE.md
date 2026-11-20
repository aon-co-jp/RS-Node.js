# CLAUDE.md (RS-Node.js / RNode.js)

- 姉妹リポジトリ: `RS-JavaScript`(Rust 製 JS エンジン、未実装)・`RS-TypeScript`(型注釈除去)・`RS-HTML`・`RS-CSS`・`RS-BootStrap`・`RS-React`、および `Reaxt`・`RPoRea`・`RPoem`・`RMojo`・`open-cosmo`・`pl2pl`。**常に同時に一緒に開発**する(1つ変えたら他の確認と README/CLAUDE/PORTING/POLICY の更新を同日に)。
- **Node.js 互換**が目標。機能を足すときは、本物の Node.js で同じスクリプトを動かして出力を比べる(`tests/compat/run.sh`)。互換を調べた結果(差・未対応)は README に正直に書く。
- JavaScript の実行は `JsEngine` トレイト経由。いまは Boa、将来 RS-JavaScript。TypeScript は RS-TypeScript で型注釈を除去してから実行。
- 新しい REST は作らない。秘密は書かない。開発・試験はローカル(WSL)、VPS は配備と確認だけ。

## 現状と完成予定 (2026-11-20) / Status and plan
### 日本語
**現状(確認済み)**: JavaScript(Boa)と TypeScript(RS-TypeScript)の実行、console・process(exit 含む)・require(path・fs 同期/非同期・fs/promises・events・util・os・buffer・node: 接頭辞)、setTimeout/setInterval/setImmediate/queueMicrotask/nextTick、Buffer。比較試験 a〜g は本物の Node(v26.5.0)と出力一致。
**完成予定(未了・この順で確認なしに進める)**: ①readFileSync が Buffer を返す ②util.inspect と console.log の Node 互換整形 ③node_modules の解決・package.json の main ④ESM(import/export) ⑤stream・http(Rust のサーバー部品を利用) ⑥自前エンジン RS-JavaScript への差し替え。
**約束**: 実機で確かめてから「できた」と書く/未対応は黙って落とさず明示する/秘密は書かない/新しい REST は作らない(Connect のみ)。
### English
**Current state (verified)**: Runs JavaScript (Boa) and TypeScript (RS-TypeScript); console, process (including exit), require (path, sync/async fs, fs/promises, events, util, os, buffer, the node: prefix), setTimeout/setInterval/setImmediate/queueMicrotask/nextTick, and Buffer. Compatibility tests a-g give the same output as real Node (v26.5.0).
**Planned (open, continued automatically in this order)**: (1) readFileSync returning a Buffer, (2) util.inspect and Node-compatible console.log formatting, (3) node_modules resolution and package.json main, (4) ESM (import/export), (5) streams and http (using our Rust server components), (6) swapping in our own RS-JavaScript engine.
**Rules**: say "done" only after checking on a real system; never drop unsupported items silently, always flag them; keep secrets out; no new REST (Connect only).

