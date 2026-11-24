> Latest translations / 翻訳の最新版: [0-LATEST-TRANSLATIONS.md](0-LATEST-TRANSLATIONS.md)

# RS-Node.js

Node.js のコア概念を、既存の Node.js 実装コードを一切流用せず Rust で
一から再実装するプロジェクト([RFrontEnd](https://github.com/aon-co-jp)
エコシステム傘下)。

## リポジトリ改称(2026-09-13)

`RNode.js`→`RS-Node.js`へGitHub上でrename済み。`aruaru.pro`向けの
フロント基盤整備に合わせた`RFrontEnd`傘下のネーミング統一の一環
(`RS-HTML`・`RS-CSS`・`RS-GraphQL`も同時に改称)。crate名(`rnodejs`)は
依存元が無いため変更しても影響は無いが、他の3リポジトリと揃えて
今回は見送り。以下の記述内の`RNode.js`表記は改称前の履歴として残す。

## これは何か(v0.1.0 の正直なスコープ)

Node.js は巨大であり、JavaScript エンジン(V8 相当)を一から作るのは
非現実的なため、v0.1.0 は **「Node.js 全体」ではなく次の 2 つの概念モデルの
骨格だけ**を実装している:

- **モジュール解決**(`resolve`): CommonJS 的な `require(path)` の
  パス解決アルゴリズム——相対パス・拡張子補完(`.js` → `.json`)・
  ディレクトリの `index`・`package.json` の `main`・`node_modules` の
  祖先方向探索——を、実ファイルシステムアクセスから切り離した純粋関数
  として実装。存在判定は `FileSystem` トレイトで抽象化され、テストは
  インメモリの `MapFileSystem` で実 I/O 無しに解決規則そのものを検証する。
- **イベントループ順序モデル**(`event_loop`): next-tick キュー →
  マイクロタスクキュー → タイマーフェーズという実行順序規則を、実時間や
  tokio に載せる前の**論理時刻ベースの決定的な純粋キューロジック**として
  実装。`setTimeout(0)` でも Promise `then` より後、タイマー 1 件ごとに
  tick/microtask を完全排出、といった Node の代表的挙動をテストで固定。
- **ランタイムアダプタ層**(`runtime`): 純粋モデルと実ランタイム
  (tokio/RPoem)の境界となる `LoopDriver` トレイト。テスト用
  `ManualDriver` 同梱。RPoem 統合は feature `rpoem-adapter` の設計
  スタブのみ(実装は未着手)。
- **`fs` モジュール**(`fs`、v0.2.0 で追加): Node の `fs`(同期 API)
  相当。`read_file`/`read_to_string`/`write_file`/`exists`/`mkdir`
  (`_recursive`)/`read_dir`/`stat`/`remove_file`/`remove_dir`
  (`_all`) を `std::fs` の薄いラッパーとして実装。実ファイルシステムに
  対する実 I/O(一時ディレクトリを使った読み書き往復テストで検証済み)。

**未実装(スコープ外)**: JavaScript の実行そのもの、`http`/`path` 等
それ以外のコアモジュール、`setImmediate`/I/O ポーリング/close フェーズ、
`package.json` の `exports`、ESM、`fs` の非同期版(`fs.promises`)。
RTypeScript との将来連携構想は `CLAUDE.md` の設計メモを参照。

## 使用例

```rust
use rnodejs::{resolve, MapFileSystem, EventLoop};

// モジュール解決(実 I/O 無し)
let fs = MapFileSystem::new()
    .with_file("/app/node_modules/lodash/index.js")
    .with_file("/app/src/util.js");
assert_eq!(resolve("lodash", "/app/src", &fs).unwrap(),
           "/app/node_modules/lodash/index.js");
assert_eq!(resolve("./util", "/app/src", &fs).unwrap(),
           "/app/src/util.js");

// イベントループ順序モデル(論理時刻・決定的)
let mut el = EventLoop::new();
el.schedule_timer(3, 0);      // setTimeout(0) 相当
el.schedule_microtask(2);     // Promise.then 相当
el.schedule_next_tick(1);     // process.nextTick 相当
let trace = el.run_until_idle();
assert_eq!(trace.iter().map(|e| e.id).collect::<Vec<_>>(), vec![1, 2, 3]);
```

## ビルド・テスト

```bash
cargo test
cargo test --features rpoem-adapter,tokio-driver
```

既定 42 テスト、全 feature 有効時 46 テスト green(2026-07-19時点)。

依存クレートゼロ(`unsafe_code = "deny"`)。追加のセットアップは不要。

## 関連プロジェクト

- [RPoem](https://github.com/aon-co-jp/RPoem) — tokio/hyper ベースの
  サーバー側実行基盤。I/O・HTTP は RPoem が担い、RNode.js は
  「Node.js 的な JavaScript ランタイム/モジュールシステムの土台」に
  焦点を当てる(役割重複を避ける分担)。
- [RTypeScript](https://github.com/aon-co-jp/RTypeScript) — トークナイザ+
  型注釈除去トランスパイラ。将来の JS 実行層の連携候補(`CLAUDE.md` 参照)。
- [RJSON](https://github.com/aon-co-jp/RJSON) — 本リポジトリの構成・
  ドキュメント体裁の雛形。

## ライセンス

Apache-2.0 OR MIT

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**

## 2026-11-20 追記: JavaScript / TypeScript を実行できるようになった(Node.js 互換を目指す) / Now runs JavaScript and TypeScript (aiming at Node.js compatibility)
**日本語**: `rnodejs <file.js|file.ts> [args...]` で実行できます(`cargo build --release --features boa,ts`)。JS は Rust 製エンジン **Boa**(feature `boa`)、TypeScript は **RS-TypeScript**(feature `ts`、型注釈を取り除く Rust 製)を通します。エンジンは `JsEngine` トレイトで抽象化してあり、将来 **RS-JavaScript**(一から作る Rust 製 JS エンジン)に差し替えられます。Node.js 風の土台: `console`、`process`(`argv`・`env`・`cwd`)、`require`(`path`・`fs` の同期版、相対パスの `.js`/`.json`/`index.js`、`module`/`exports`/`__dirname`/`__filename`)。**互換の確認方法**: `tests/compat/run.sh` が同じスクリプトを rnodejs と**本物の Node.js(v26.5.0)**で実行して出力を比べます(いまは 2 本とも一致)。この比較で `path.normalize` の末尾スラッシュの違いを見つけて直しました。同時に、RS-TypeScript の不具合(関数呼び出しの中のオブジェクトリテラル `{ n: x }` を引数の型注釈と取り違えて削る)も見つけて直しました。
**未対応(Node.js との差)**: `setTimeout` などのタイマー・イベントループ(`event_loop` モジュールのモデルはあるが Boa には未接続)、`process.exit`、`Buffer`、`fs` の非同期版、`http`・`events`・`util`・`os` などのコアモジュール、ESM(`import`)、`node_modules` と `package.json` の解決(`resolve` モジュールのモデルはあるが未接続)。互換の範囲は、本物の Node.js と比べて一致を確かめながら広げます。
**English**: Run `rnodejs <file.js|file.ts> [args...]` (build with `cargo build --release --features boa,ts`). JavaScript goes through the Rust engine **Boa** (feature `boa`); TypeScript goes through **RS-TypeScript** (feature `ts`, a Rust type-annotation stripper). The engine is behind the `JsEngine` trait so **RS-JavaScript** (our from-scratch Rust JS engine) can replace Boa later. Node.js-style base: `console`, `process` (`argv`, `env`, `cwd`), `require` (synchronous `path` and `fs`, relative `.js`/`.json`/`index.js`, `module`/`exports`/`__dirname`/`__filename`). **How compatibility is checked**: `tests/compat/run.sh` runs the same script under rnodejs and under **real Node.js (v26.5.0)** and compares the output (both scripts currently match). That comparison exposed a trailing-slash difference in `path.normalize`, now fixed, and a bug in RS-TypeScript (object-literal properties inside call arguments were stripped as parameter annotations), also fixed.
**Not yet (differences from Node.js)**: timers and the event loop (`setTimeout` etc.; the `event_loop` model exists but is not wired to Boa), `process.exit`, `Buffer`, async `fs`, core modules such as `http`, `events`, `util`, `os`, ESM (`import`), and `node_modules` / `package.json` resolution (the `resolve` model exists but is not wired). Compatibility will widen only as it is verified against real Node.js.

## 現状と完成予定 (2026-11-20) / Status and plan
### 日本語
**現状(確認済み)**: JavaScript(Boa)と TypeScript(RS-TypeScript)の実行、console・process(exit 含む)・require(path・fs 同期/非同期・fs/promises・events・util・os・buffer・node: 接頭辞)、setTimeout/setInterval/setImmediate/queueMicrotask/nextTick、Buffer。比較試験 a〜g は本物の Node(v26.5.0)と出力一致。
**完成予定(未了・この順で確認なしに進める)**: ①readFileSync が Buffer を返す ②util.inspect と console.log の Node 互換整形 ③node_modules の解決・package.json の main ④ESM(import/export) ⑤stream・http(Rust のサーバー部品を利用) ⑥自前エンジン RS-JavaScript への差し替え。
**約束**: 実機で確かめてから「できた」と書く/未対応は黙って落とさず明示する/秘密は書かない/新しい REST は作らない(Connect のみ)。
### English
**Current state (verified)**: Runs JavaScript (Boa) and TypeScript (RS-TypeScript); console, process (including exit), require (path, sync/async fs, fs/promises, events, util, os, buffer, the node: prefix), setTimeout/setInterval/setImmediate/queueMicrotask/nextTick, and Buffer. Compatibility tests a-g give the same output as real Node (v26.5.0).
**Planned (open, continued automatically in this order)**: (1) readFileSync returning a Buffer, (2) util.inspect and Node-compatible console.log formatting, (3) node_modules resolution and package.json main, (4) ESM (import/export), (5) streams and http (using our Rust server components), (6) swapping in our own RS-JavaScript engine.
**Rules**: say "done" only after checking on a real system; never drop unsupported items silently, always flag them; keep secrets out; no new REST (Connect only).


### 追記 (2026-11-20) / Update
**日本語**: `fs.readFileSync` / `fs.readFile` / `fs.promises.readFile` が、文字コード指定なしのとき本物の Node と同じく Buffer を返すようになった(`tests/compat/g.js` で一致確認)。完成予定の①は完了。残り: util.inspect と console.log の整形、node_modules 解決、ESM、stream・http、RS-JavaScript への差し替え。
**English**: `fs.readFileSync`, `fs.readFile` and `fs.promises.readFile` now return a Buffer when no encoding is given, like real Node (verified with `tests/compat/g.js`). Planned item 1 is done. Remaining: util.inspect and console.log formatting, node_modules resolution, ESM, streams and http, swapping in RS-JavaScript.

### 追記 (2026-11-20) 紹介ページ / Intro pages
**日本語**: 紹介ページ https://easy-web.tokyo/RS-Node.js/ とデモ https://easy-web.tokyo/RS-Node.js/demo/(実際に変換・実行した入出力の記録)を公開した。VPS の `/root/easy-web-intros/RS-Node.js/`(静的、ポート 8141)と `domains.toml` の `path_prefix` で配信。生成は runo の `scripts/gen-converters-intro.py`。日英のみで、他言語は準備中。
**English**: The intro page https://easy-web.tokyo/RS-Node.js/ and the demo https://easy-web.tokyo/RS-Node.js/demo/ (a record of real conversion/run input and output) are live. They are served statically from `/root/easy-web-intros/RS-Node.js/` on the VPS (port 8141) through a `path_prefix` entry in `domains.toml`; generated by `scripts/gen-converters-intro.py` in runo. Japanese and English only; other languages are in preparation.

## 多言語: 現状と完成予定 / Status and plan in 10 languages (ja, en above)

### 简体中文 — 现状与计划 (2026-11-20)
*以下为 AI 翻译,并已由 AI 按母语水平审校;尚未经人类母语者校对。*

**现状(已验证)**: 面向 Node.js 兼容的 Rust 运行时:JavaScript 由 Rust 引擎 Boa 运行,TypeScript 通过 RS-TypeScript 运行。支持 console、process(含 exit)、require(path、同步/异步 fs、fs/promises、events、util、os、buffer、node: 前缀)、setTimeout / setInterval / setImmediate / queueMicrotask / nextTick 和 Buffer。对比测试的输出与真实 Node(v26.5.0)一致。

**计划(未完成,按此顺序自动推进)**: util.inspect 与 console.log 的格式、node_modules 解析、ESM、stream 与 http、换用自研引擎 RS-JavaScript。

原则:先在真实环境确认再说“完成”;不支持的内容绝不悄悄丢弃,而是明确标出;不写入机密;不新增 REST(仅 Connect)。

### 繁體中文(台灣) — 現況與計畫 (2026-11-20)
*以下為 AI 翻譯,並已由 AI 以母語水準審校;尚未經人類母語者校對。*

**現況(已驗證)**: 以 Node.js 相容為目標的 Rust 執行環境:JavaScript 由 Rust 引擎 Boa 執行,TypeScript 透過 RS-TypeScript 執行。支援 console、process(含 exit)、require(path、同步/非同步 fs、fs/promises、events、util、os、buffer、node: 前綴)、setTimeout / setInterval / setImmediate / queueMicrotask / nextTick 與 Buffer。比對測試的輸出與真實 Node(v26.5.0)一致。

**計畫(未完成,依此順序自動推進)**: util.inspect 與 console.log 的格式、node_modules 解析、ESM、stream 與 http、改用自研引擎 RS-JavaScript。

原則:先在真實環境確認才說「完成」;不支援的內容絕不悄悄丟棄,而是明確標出;不寫入機密;不新增 REST(僅 Connect)。

### 한국어 — 현황과 계획 (2026-11-20)
*아래는 AI가 번역하고 AI가 원어민 수준으로 검수한 것이며, 사람 원어민의 검수는 아직 받지 않았습니다.*

**현황(검증 완료)**: Node.js 호환을 목표로 하는 Rust 런타임입니다. JavaScript는 Rust 엔진 Boa로, TypeScript는 RS-TypeScript로 실행합니다. console, process(exit 포함), require(path, 동기/비동기 fs, fs/promises, events, util, os, buffer, node: 접두사), setTimeout / setInterval / setImmediate / queueMicrotask / nextTick, Buffer를 지원합니다. 비교 테스트의 출력이 실제 Node(v26.5.0)와 같습니다.

**계획(미완료, 이 순서대로 자동 진행)**: util.inspect와 console.log의 출력 형식, node_modules 해석, ESM, stream과 http, 자체 엔진 RS-JavaScript로의 교체.

원칙: 실제 환경에서 확인한 뒤에만 "완료"라고 씁니다. 지원하지 않는 항목은 조용히 버리지 않고 명확히 표시합니다. 비밀 정보는 쓰지 않으며 새 REST는 만들지 않습니다(Connect만).

### Deutsch — Stand und Plan (2026-11-20)
*Die folgenden Texte wurden von einer KI übersetzt und von einer KI auf muttersprachliche Natürlichkeit geprüft; eine Prüfung durch menschliche Muttersprachler steht noch aus.*

**Aktueller Stand (geprüft)**: Eine Rust-Laufzeit mit dem Ziel der Node.js-Kompatibilität: JavaScript läuft auf der Rust-Engine Boa, TypeScript über RS-TypeScript. Unterstützt console, process (inkl. exit), require (path, synchrones/asynchrones fs, fs/promises, events, util, os, buffer, Präfix node:), setTimeout / setInterval / setImmediate / queueMicrotask / nextTick sowie Buffer. Die Vergleichstests liefern dieselbe Ausgabe wie echtes Node (v26.5.0).

**Geplant (offen, wird in dieser Reihenfolge automatisch fortgesetzt)**: util.inspect und die Formatierung von console.log, Auflösung von node_modules, ESM, Streams und http sowie der Wechsel auf die eigene Engine RS-JavaScript.

Regeln: „Fertig“ erst nach Prüfung auf einem echten System; Nicht-Unterstütztes nie stillschweigend weglassen, sondern klar kennzeichnen; keine Geheimnisse ablegen; keine neue REST-API (nur Connect).

### Français — État et plan (2026-11-20)
*Les textes suivants ont été traduits par une IA et relus par une IA pour leur naturel ; ils n'ont pas encore été relus par des locuteurs natifs humains.*

**État actuel (vérifié)**: Un environnement d'exécution Rust visant la compatibilité Node.js : JavaScript tourne sur le moteur Rust Boa, TypeScript via RS-TypeScript. Prend en charge console, process (dont exit), require (path, fs synchrone/asynchrone, fs/promises, events, util, os, buffer, préfixe node:), setTimeout / setInterval / setImmediate / queueMicrotask / nextTick et Buffer. Les tests de comparaison donnent la même sortie que le vrai Node (v26.5.0).

**Prévu (ouvert, poursuivi automatiquement dans cet ordre)**: util.inspect et le formatage de console.log, la résolution de node_modules, ESM, streams et http, puis le passage à notre propre moteur RS-JavaScript.

Règles : n'écrire « terminé » qu'après vérification sur un système réel ; ne jamais abandonner en silence ce qui n'est pas pris en charge, mais le signaler clairement ; aucun secret ; pas de nouvelle API REST (Connect uniquement).

### Español — Estado y plan (2026-11-20)
*Los textos siguientes fueron traducidos por una IA y revisados por una IA en cuanto a naturalidad; aún no los ha revisado un hablante nativo humano.*

**Estado actual (verificado)**: Un entorno de ejecución en Rust que busca compatibilidad con Node.js: JavaScript se ejecuta en el motor Rust Boa y TypeScript mediante RS-TypeScript. Admite console, process (con exit), require (path, fs síncrono/asíncrono, fs/promises, events, util, os, buffer, prefijo node:), setTimeout / setInterval / setImmediate / queueMicrotask / nextTick y Buffer. Las pruebas de comparación dan la misma salida que Node real (v26.5.0).

**Previsto (pendiente, se continúa automáticamente en este orden)**: El formato de util.inspect y console.log, la resolución de node_modules, ESM, streams y http, y el cambio a nuestro propio motor RS-JavaScript.

Reglas: decir «terminado» solo tras comprobarlo en un sistema real; nunca descartar en silencio lo no admitido, sino señalarlo claramente; sin secretos; sin nuevas API REST (solo Connect).

### Português (BR) — Estado e plano (2026-11-20)
*Os textos a seguir foram traduzidos por uma IA e revisados por uma IA quanto à naturalidade; ainda não foram revisados por falantes nativos humanos.*

**Estado atual (verificado)**: Um ambiente de execução em Rust que busca compatibilidade com o Node.js: o JavaScript roda no motor Rust Boa e o TypeScript via RS-TypeScript. Suporta console, process (com exit), require (path, fs síncrono/assíncrono, fs/promises, events, util, os, buffer, prefixo node:), setTimeout / setInterval / setImmediate / queueMicrotask / nextTick e Buffer. Os testes de comparação dão a mesma saída que o Node real (v26.5.0).

**Planejado (em aberto, seguido automaticamente nesta ordem)**: A formatação de util.inspect e console.log, a resolução de node_modules, ESM, streams e http, e a troca para o nosso próprio motor RS-JavaScript.

Regras: só dizer «concluído» depois de verificar em um sistema real; nunca descartar em silêncio o que não é suportado, mas sinalizar claramente; sem segredos; sem novas APIs REST (somente Connect).

### Русский — Состояние и план (2026-11-20)
*Следующие тексты переведены ИИ и проверены ИИ на естественность звучания; носители языка — люди — их пока не проверяли.*

**Текущее состояние (проверено)**: Среда выполнения на Rust, нацеленная на совместимость с Node.js: JavaScript работает на Rust-движке Boa, TypeScript — через RS-TypeScript. Поддерживает console, process (включая exit), require (path, синхронный/асинхронный fs, fs/promises, events, util, os, buffer, префикс node:), setTimeout / setInterval / setImmediate / queueMicrotask / nextTick и Buffer. Сравнительные тесты дают тот же вывод, что и настоящий Node (v26.5.0).

**Планируется (не завершено, продолжается автоматически в таком порядке)**: Форматирование util.inspect и console.log, разрешение node_modules, ESM, потоки и http, переход на собственный движок RS-JavaScript.

Правила: писать «готово» только после проверки на реальной системе; неподдерживаемое никогда не отбрасывать молча, а явно помечать; не хранить секреты; не создавать новый REST (только Connect).

## アラビア語・ペルシャ語(イラン) / Arabic and Persian (Iran) — AI翻訳+AIネイティブ確認、人間のネイティブ確認はまだ

### العربية — الحالة والخطة (2026-11-20)

<div dir="rtl" lang="ar">

*النص التالي ترجمه الذكاء الاصطناعي وراجعه الذكاء الاصطناعي من حيث سلامة اللغة، ولم يراجعه بعدُ متحدث أصلي من البشر.*

**الحالة الحالية (تم التحقق منها)**: بيئة تشغيل بلغة Rust تستهدف التوافق مع Node.js: يعمل JavaScript على محرك Rust المسمى Boa، ويعمل TypeScript عبر RS-TypeScript. تدعم console وprocess (مع exit) وrequire (path وfs المتزامن وغير المتزامن وfs/promises وevents وutil وos وbuffer والبادئة node:) وsetTimeout / setInterval / setImmediate / queueMicrotask / nextTick وBuffer. تعطي اختبارات المقارنة المخرجات نفسها التي يعطيها Node الحقيقي (v26.5.0).

**المخطَّط له (غير مكتمل، يُتابَع تلقائيًا بهذا الترتيب)**: تنسيق util.inspect وconsole.log، وتحليل مسارات node_modules، وESM، وstream وhttp، والتحوّل إلى محركنا الخاص RS-JavaScript.

المبادئ: لا نكتب «تم» إلا بعد التحقق على نظام حقيقي؛ لا نُسقِط ما لا يُدعَم بصمت بل نضع عليه علامة واضحة؛ لا نكتب أسرارًا؛ لا نضيف واجهات REST جديدة (Connect فقط).

</div>

### فارسی (ایران) — وضعیت و برنامه (2026-11-20)

<div dir="rtl" lang="fa">

*متن زیر را هوش مصنوعی ترجمه و از نظر روانی زبان بازبینی کرده است؛ هنوز هیچ فارسی‌زبانِ بومیِ انسانی آن را بررسی نکرده است.*

**وضعیت کنونی (بررسی‌شده)**: محیط اجرای Rust با هدف سازگاری با Node.js: JavaScript روی موتور Rust به نام Boa و TypeScript از راه RS-TypeScript اجرا می‌شود. از console، process (همراه exit)، require (path، fs همگام و ناهمگام، fs/promises، events، util، os، buffer، پیشوند node:)، setTimeout / setInterval / setImmediate / queueMicrotask / nextTick و Buffer پشتیبانی می‌کند. آزمون‌های مقایسه‌ای همان خروجی Node واقعی (v26.5.0) را می‌دهند.

**برنامه‌ریزی‌شده (ناتمام؛ به‌صورت خودکار و با همین ترتیب ادامه می‌یابد)**: قالب‌بندی util.inspect و console.log، یافتن ماژول‌ها در node_modules، ESM، stream و http و جایگزینی با موتور خودمان RS-JavaScript.

اصول: فقط پس از بررسی روی یک سامانهٔ واقعی می‌نویسیم «انجام شد»؛ آنچه پشتیبانی نمی‌شود هرگز بی‌صدا حذف نمی‌شود و آشکارا علامت می‌خورد؛ رازی (مانند گذرواژه و کلید) نمی‌نویسیم؛ REST جدید نمی‌سازیم (فقط Connect).

</div>

## 2026-11-21 更新(util.inspect と ESM) / 2026-11-21 update: util.inspect and ES modules (10 more languages; AI翻訳+AI確認、人間のネイティブ確認はまだ)

**日本語**: `console.log`・`util.inspect`・`util.format` の出力を本物の Node.js と一致させ、ES モジュール(`import` / `export`、`.mjs`)を CommonJS への書き換えで対応した。Boa を 0.22 へ更新。未対応: トップレベル `await`・`import.meta`・動的 `import()`・ライブバインディング。

**English**: `console.log`, `util.inspect` and `util.format` now match real Node.js output, and ES modules (`import` / `export`, `.mjs`) work through a CommonJS rewrite. Boa was updated to 0.22. Not supported yet: top-level `await`, `import.meta`, dynamic `import()`, live bindings.

### 简体中文 — 更新 (2026-11-21)

*以下为 AI 翻译,并已由 AI 按母语水平审校;尚未经人类母语者校对。*

`console.log` 与 `util.inspect` / `util.format` 现在的输出与真正的 Node.js 完全一致(嵌套对象、数组折行、`Map`/`Set`、类、循环引用、`%s %d %o` 等,已逐字节比较)。同时支持 ES 模块:`import`(默认、命名、`* as`、仅副作用)和 `export`(`export const`、`export default`、`export { a as b }`、`export * from`)会被改写成 CommonJS,也可 `require` `.mjs` 文件。JS 引擎 Boa 升级到 0.22(修复了类声明导致的崩溃)。尚未支持:顶层 `await`、`import.meta`、动态 `import()`、实时绑定。

### 繁體中文(台灣) — 更新 (2026-11-21)

*以下為 AI 翻譯,並已由 AI 以母語水準審校;尚未經人類母語者校對。*

`console.log` 與 `util.inspect` / `util.format` 現在的輸出與真正的 Node.js 完全一致(巢狀物件、陣列換行、`Map`/`Set`、類別、循環參照、`%s %d %o` 等,已逐位元組比較)。同時支援 ES 模組:`import`(預設、具名、`* as`、僅副作用)與 `export`(`export const`、`export default`、`export { a as b }`、`export * from`)會被改寫成 CommonJS,也可 `require` `.mjs` 檔案。JS 引擎 Boa 升級到 0.22(修正了類別宣告造成的當機)。尚未支援:頂層 `await`、`import.meta`、動態 `import()`、即時繫結。

### 한국어 — 업데이트 (2026-11-21)

*아래는 AI가 번역하고 AI가 원어민 수준으로 검수한 것이며, 사람 원어민의 검수는 아직 받지 않았습니다.*

`console.log`와 `util.inspect` / `util.format`의 출력이 이제 진짜 Node.js와 완전히 같습니다(중첩 객체, 배열 줄바꿈, `Map`/`Set`, 클래스, 순환 참조, `%s %d %o` 등을 바이트 단위로 비교했습니다). ES 모듈도 지원합니다: `import`(기본, 이름 있는, `* as`, 부수 효과만)와 `export`(`export const`, `export default`, `export { a as b }`, `export * from`)를 CommonJS로 바꿔 실행하며, `.mjs` 파일도 `require`할 수 있습니다. JS 엔진 Boa를 0.22로 올려 클래스 선언 때 죽던 문제를 고쳤습니다. 아직 안 되는 것: 최상위 `await`, `import.meta`, 동적 `import()`, 라이브 바인딩.

### Deutsch — Aktualisierung (2026-11-21)

*Die folgenden Texte wurden von einer KI übersetzt und von einer KI auf muttersprachliche Natürlichkeit geprüft; eine Prüfung durch menschliche Muttersprachler steht noch aus.*

Die Ausgabe von `console.log` sowie `util.inspect` / `util.format` ist jetzt identisch mit dem echten Node.js (verschachtelte Objekte, Zeilenumbruch bei Arrays, `Map`/`Set`, Klassen, Zirkelbezüge, `%s %d %o` und mehr, Byte für Byte verglichen). Außerdem werden ES-Module unterstützt: `import` (Standard, benannt, `* as`, nur Nebenwirkung) und `export` (`export const`, `export default`, `export { a as b }`, `export * from`) werden in CommonJS umgeschrieben; auch `.mjs`-Dateien lassen sich per `require` laden. Die JS-Engine Boa wurde auf 0.22 aktualisiert (behebt einen Absturz bei Klassendeklarationen). Noch nicht unterstützt: `await` auf oberster Ebene, `import.meta`, dynamisches `import()`, Live-Bindungen.

### Français — Mise à jour (2026-11-21)

*Les textes suivants ont été traduits par une IA et relus par une IA pour leur naturel ; ils n'ont pas encore été relus par des locuteurs natifs humains.*

La sortie de `console.log` et de `util.inspect` / `util.format` est désormais identique à celle du vrai Node.js (objets imbriqués, retours à la ligne des tableaux, `Map`/`Set`, classes, références circulaires, `%s %d %o`, etc., comparée octet par octet). Les modules ES sont aussi pris en charge : `import` (par défaut, nommé, `* as`, effet de bord seul) et `export` (`export const`, `export default`, `export { a as b }`, `export * from`) sont réécrits en CommonJS, et les fichiers `.mjs` peuvent être chargés avec `require`. Le moteur JS Boa passe à la version 0.22 (corrige un plantage sur les déclarations de classe). Pas encore pris en charge : `await` au niveau supérieur, `import.meta`, `import()` dynamique, liaisons dynamiques.

### Español — Actualización (2026-11-21)

*Los textos siguientes fueron traducidos por una IA y revisados por una IA en cuanto a naturalidad; aún no los ha revisado un hablante nativo humano.*

La salida de `console.log` y de `util.inspect` / `util.format` ahora es idéntica a la del Node.js real (objetos anidados, saltos de línea en arreglos, `Map`/`Set`, clases, referencias circulares, `%s %d %o`, etc., comparada byte a byte). También se admiten módulos ES: `import` (por defecto, con nombre, `* as`, solo efecto secundario) y `export` (`export const`, `export default`, `export { a as b }`, `export * from`) se reescriben a CommonJS, y los archivos `.mjs` se pueden cargar con `require`. El motor JS Boa sube a la versión 0.22 (corrige un fallo con las declaraciones de clase). Todavía no se admiten: `await` de nivel superior, `import.meta`, `import()` dinámico y enlaces vivos.

### Português (BR) — Atualização (2026-11-21)

*Os textos a seguir foram traduzidos por uma IA e revisados por uma IA quanto à naturalidade; ainda não foram revisados por falantes nativos humanos.*

A saída de `console.log` e de `util.inspect` / `util.format` agora é idêntica à do Node.js real (objetos aninhados, quebra de linha em arrays, `Map`/`Set`, classes, referências circulares, `%s %d %o` etc., comparada byte a byte). Módulos ES também são aceitos: `import` (padrão, nomeado, `* as`, só efeito colateral) e `export` (`export const`, `export default`, `export { a as b }`, `export * from`) são reescritos para CommonJS, e arquivos `.mjs` podem ser carregados com `require`. O motor JS Boa foi atualizado para a versão 0.22 (corrige uma falha em declarações de classe). Ainda não há suporte para: `await` no nível superior, `import.meta`, `import()` dinâmico e vínculos dinâmicos.

### Русский — Обновление (2026-11-21)

*Следующие тексты переведены ИИ и проверены ИИ на естественность звучания; носители языка — люди — их пока не проверяли.*

Вывод `console.log` и `util.inspect` / `util.format` теперь совпадает с настоящим Node.js (вложенные объекты, переносы в массивах, `Map`/`Set`, классы, циклические ссылки, `%s %d %o` и др.; сравнение побайтовое). Также поддерживаются ES-модули: `import` (по умолчанию, именованный, `* as`, только побочный эффект) и `export` (`export const`, `export default`, `export { a as b }`, `export * from`) переписываются в CommonJS, а файлы `.mjs` можно загружать через `require`. Движок JS Boa обновлён до версии 0.22 (исправлен сбой при объявлении классов). Пока не поддерживаются: `await` на верхнем уровне, `import.meta`, динамический `import()` и «живые» привязки.

### العربية — تحديث (2026-11-21)

<div dir="rtl" lang="ar">

*النص التالي ترجمه الذكاء الاصطناعي وراجعه الذكاء الاصطناعي من حيث سلامة اللغة، ولم يراجعه بعدُ متحدث أصلي من البشر.*

أصبح ناتج `console.log` و`util.inspect` / `util.format` مطابقًا لناتج Node.js الحقيقي (الكائنات المتداخلة، التفاف المصفوفات، `Map`/`Set`، الأصناف، المراجع الدائرية، `%s %d %o` وغيرها، وقد قورن بايتًا ببايت). كما تُدعم وحدات ES: يُعاد كتابة `import` (الافتراضي، المسمّى، `* as`، الأثر الجانبي فقط) و`export` (`export const` و`export default` و`export { a as b }` و`export * from`) إلى CommonJS، ويمكن تحميل ملفات `.mjs` عبر `require`. وتمت ترقية محرك JS المسمى Boa إلى الإصدار 0.22 (إصلاح انهيار عند تعريف الأصناف). غير المدعوم بعد: `await` في المستوى الأعلى، و`import.meta`، و`import()` الديناميكي، والارتباطات الحيّة.

</div>

### فارسی (ایران) — به‌روزرسانی (2026-11-21)

<div dir="rtl" lang="fa">

*متن زیر را هوش مصنوعی ترجمه و از نظر روانی زبان بازبینی کرده است؛ هنوز هیچ فارسی‌زبانِ بومیِ انسانی آن را بررسی نکرده است.*

خروجی `console.log` و `util.inspect` / `util.format` اکنون با Node.js واقعی یکسان است (شیءهای تودرتو، شکستن خط آرایه‌ها، `Map`/`Set`، کلاس‌ها، ارجاع‌های دوری، `%s %d %o` و غیره؛ بایت‌به‌بایت مقایسه شده است). ماژول‌های ES هم پشتیبانی می‌شوند: `import` (پیش‌فرض، نام‌دار، `* as`، فقط اثر جانبی) و `export` (`export const`، `export default`، `export { a as b }`، `export * from`) به CommonJS بازنویسی می‌شوند و فایل‌های `.mjs` را می‌توان با `require` بارگذاری کرد. موتور JS با نام Boa به نسخهٔ 0.22 ارتقا یافت (خرابی هنگام تعریف کلاس رفع شد). هنوز پشتیبانی نمی‌شود: `await` سطح بالا، `import.meta`، `import()` پویا و اتصال‌های زنده.

</div>
