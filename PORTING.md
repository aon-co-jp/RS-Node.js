
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

## 2026-11-22 追記: node_modules の解決 / node_modules resolution
**日本語**: `require("foo")` のようなベア指定で、親方向へ `node_modules` を探し、`package.json` の `main`、ディレクトリの `index.js`/`index.json` に対応しました。見つからないときは `code: MODULE_NOT_FOUND` のエラーです。あわせて `fs.statSync`(`isFile`/`isDirectory`)を足し、Windows のドライブ文字つきパスで `path.resolve` が二重になる問題を直しました。試験素材は `tests/compat/nm/`(`foo-main` と `MODULE_NOT_FOUND` を確認、単体試験42件成功、`36cf78a`)。**まだ**: `package.json` の `exports`、シンボリックリンク、`stream` と `http`、RS-JavaScript への差し替え。
**English**: Bare requires such as `require("foo")` now search `node_modules` upward and honour `package.json` `main` and directory `index.js`/`index.json`; a miss throws an error with `code: MODULE_NOT_FOUND`. We also added `fs.statSync` (`isFile`/`isDirectory`) and fixed the doubled `path.resolve` result for Windows drive-letter paths. Fixture: `tests/compat/nm/` (prints `foo-main` and `MODULE_NOT_FOUND`; 42 unit tests pass; `36cf78a`). **Not yet**: `package.json` `exports`, symlinks, `stream` and `http`, swapping in RS-JavaScript.

## Versionless API の方針 / Versionless API policy (2026-11-25)
**日本語**: このリポジトリのAPIは **Versionless**(URL・パッケージ名に `v1` 等を付けず、互換性を壊さず足すだけで進化させる)。欄は足すだけ・足す欄は省略可能・知らない欄は無視・名前変更は新名を足して旧を非推奨に(期限は既定で6か月以上、使用0を確認してから削除)・enumは未知の値を許容・安全上の緊急修正は例外。既存の `.v1` は呼び出し側を壊さないよう当面残し、版なしの別名を追加して段階的に畳む。詳細・出典は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md) 末尾。適用はこれから順に行う。
**English**: This repository's APIs are **Versionless**: no `v1` in URLs or package names; evolve by adding only, without breaking compatibility. Only add fields, new fields are optional, ignore unknown fields, rename by adding the new name and deprecating the old (default deadline 6 months or more, remove only after confirming zero use), tolerate unknown enum values, emergency security fixes are the exception. Existing `.v1` names stay for now so callers do not break; a versionless alias is added and `.v1` is retired in stages. Details and sources: the end of [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md). Rollout is step by step.

## API 方針: REST 全廃・gRPC / Connect へ
**日本語**: このリポジトリでは **REST API を新規に作らない**。ブラウザ・`curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`)、サービス間は **gRPC / gRPC-web**。GraphQL は読み取り専用。既存の REST は触るときに移し、移したら旧経路は消す。JSON は RS-JSON(`from_str_secure` / `parse_secure`)で読む。例外は相手の規格が形を決めるもの(ACME、OAuth/OIDC のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)のみ。迷ったら Connect。正本は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md)。
**English**: This repository **does not add new REST APIs**. APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`); service-to-service calls use **gRPC / gRPC-web**. GraphQL is read-only. Existing REST is migrated when touched and the old route is removed. Read JSON with RS-JSON (`from_str_secure` / `parse_secure`). Exceptions are only shapes dictated by someone else's standard (ACME, OAuth/OIDC callbacks, third-party webhooks, health checks, static files, OTLP). When in doubt, use Connect. Source of truth: the link above.

## 完成まで作り切る方針 / Finish everything policy (2026-11-27)
**日本語**: 全リポジトリで、**未確認は自動で確認し、未実施は実施し、未了は完成させる**(確認を求めず進める)。使用頻度が低いと判断される機能や、**1から作り直す大規模開発になる内容でも、やめずに最後まで開発・TEST・DEBUGする**(別の方式で済ませない、後回しにしない)。進め方: 調査(英語・日本語の検索とGitHub) → 設計 → 実装 → 実機/実データで確認 → 失敗は原因を直して再試験 → README・CLAUDE・PORTING・POLICY を日英で更新 → push。「できた」と書くのは実測した範囲だけ、確認できなかった所は「未確認」と正直に書く。例外(確認を取る): 元に戻せない削除、秘密の公開、所有者の認証・決済・実機操作が要るもの、利用者のシステム設定の変更。
**English**: In every repository, **verify what is unverified, do what is not yet done, and finish what is unfinished** automatically, without asking. Even features that look rarely used, or work that means **a large rewrite from scratch, are developed, tested and debugged to the end** (no substituting another approach, no postponing). Method: research (English and Japanese search plus GitHub) -> design -> implement -> verify on real devices or data -> fix causes and retest on failure -> update README, CLAUDE, PORTING and POLICY in Japanese and English -> push. Write "done" only for what was measured, and say "unverified" honestly for the rest. Exceptions that still need confirmation: irreversible deletions, publishing secrets, anything needing the owner's authentication, payment or real-device actions, and changes to the user's system settings.