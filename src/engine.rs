//! JavaScript エンジンとの接続(2026-11-20 追加)。
//!
//! - [`JsEngine`]: エンジンの抽象。いまは feature `boa`(Rust 製の JS エンジン Boa)の [`BoaEngine`] を実装。
//!   将来、`RS-JavaScript`(一から作る Rust 製 JS エンジン)が同じトレイトを実装すれば、差し替えられる。
//! - Node.js 風の土台: `console` / `process`(argv・env・cwd・exit は未対応)/ `require`('path' と 'fs' の同期版、相対パスの `.js`/`.json`)。
//!   ネイティブ側(Rust)の関数は `__print` `__readFile` `__writeFile` `__exists` `__cwd` `__env` の 6 つだけで、残りは JS の前置き(PRELUDE)。
//! - TypeScript: feature `ts` で `RS-TypeScript`(型注釈を取り除く Rust 製トランスパイラ)を通してから実行([`run_source`] の `is_ts`)。

/// エンジンの抽象(Boa でも、将来の RS-JavaScript でも差し替えられる)
pub trait JsEngine {
    /// ソースを評価し、最後の式の値を文字列で返す
    fn eval(&mut self, source: &str) -> Result<String, String>;
}

/// Node.js 風の土台(JS)。ネイティブ関数 `__print` `__readFile` `__writeFile` `__exists` `__cwd` `__env` を使う。
pub const PRELUDE: &str = r#"
(function (g) {
  var fmt = function (a) { return typeof a === 'string' ? a : (a && typeof a === 'object' ? JSON.stringify(a) : String(a)); };
  g.console = { log: function () { __print(Array.prototype.map.call(arguments, fmt).join(' ')); } };
  g.console.error = g.console.warn = g.console.info = g.console.log;
  g.process = { argv: __argv(), env: new Proxy({}, { get: function (_, k) { return __env(String(k)); } }), cwd: function () { return __cwd(); }, exit: function (c) { __exit(c === undefined ? (g.process.exitCode | 0) : (c | 0)); }, platform: 'rnodejs' };
  var path = {
    sep: '/',
    normalize: function (p) {
      var abs = p.charAt(0) === '/', out = [];
      p.split('/').forEach(function (s) { if (s === '' || s === '.') return; if (s === '..') { if (out.length && out[out.length - 1] !== '..') out.pop(); else if (!abs) out.push('..'); } else out.push(s); });
      var res = (abs ? '/' : '') + out.join('/');
      if (!res) res = abs ? '/' : '.';
      if (p.length > 1 && p.charAt(p.length - 1) === '/' && res.charAt(res.length - 1) !== '/') res += '/';
      return res;
    },
    join: function () { return path.normalize(Array.prototype.filter.call(arguments, Boolean).join('/')); },
    resolve: function () { var r = ''; for (var i = arguments.length - 1; i >= 0 && r.charAt(0) !== '/'; i--) r = arguments[i] + (r ? '/' + r : ''); if (r.charAt(0) !== '/') r = __cwd() + '/' + r; return path.normalize(r); },
    dirname: function (p) { var i = p.lastIndexOf('/'); return i < 0 ? '.' : (i === 0 ? '/' : p.slice(0, i)); },
    basename: function (p, ext) { var b = p.slice(p.lastIndexOf('/') + 1); return ext && b.slice(-ext.length) === ext ? b.slice(0, -ext.length) : b; },
    extname: function (p) { var b = path.basename(p), i = b.lastIndexOf('.'); return i <= 0 ? '' : b.slice(i); }
  };
  path.posix = path;
  var fs = {
    readFileSync: function (p) { return __readFile(String(p)); },
    writeFileSync: function (p, d) { __writeFile(String(p), String(d)); },
    existsSync: function (p) { return __exists(String(p)); }
  };
  var cache = {};
  function load(file) {
    if (cache[file]) return cache[file].exports;
    var m = { exports: {} }; cache[file] = m;
    var src = fs.readFileSync(file);
    if (/\.json$/.test(file)) { m.exports = JSON.parse(src); return m.exports; }
    new Function('module', 'exports', 'require', '__dirname', '__filename', src)(m, m.exports, makeRequire(path.dirname(file)), path.dirname(file), file);
    return m.exports;
  }
  function EventEmitter() { this._events = {}; }
  EventEmitter.prototype._l = function (n) { if (!this._events) this._events = {}; return this._events[n] || (this._events[n] = []); };
  EventEmitter.prototype.on = EventEmitter.prototype.addListener = function (n, f) { this._l(n).push({ f: f, once: false }); return this; };
  EventEmitter.prototype.prependListener = function (n, f) { this._l(n).unshift({ f: f, once: false }); return this; };
  EventEmitter.prototype.once = function (n, f) { this._l(n).push({ f: f, once: true }); return this; };
  EventEmitter.prototype.off = EventEmitter.prototype.removeListener = function (n, f) {
    var a = this._l(n); for (var i = 0; i < a.length; i++) if (a[i].f === f) { a.splice(i, 1); break; } return this;
  };
  EventEmitter.prototype.removeAllListeners = function (n) { if (n === undefined) this._events = {}; else delete this._events[n]; return this; };
  EventEmitter.prototype.listenerCount = function (n) { return this._l(n).length; };
  EventEmitter.prototype.emit = function (n) {
    var a = this._l(n).slice(), args = Array.prototype.slice.call(arguments, 1);
    if (!a.length) { if (n === 'error') throw (args[0] instanceof Error ? args[0] : new Error('Unhandled error.')); return false; }
    for (var i = 0; i < a.length; i++) { if (a[i].once) this.removeListener(n, a[i].f); a[i].f.apply(this, args); }
    return true;
  };
  EventEmitter.EventEmitter = EventEmitter;
  var events = EventEmitter;
  var util = {
    format: function (f) {
      var args = Array.prototype.slice.call(arguments, 1), i = 0;
      if (typeof f !== 'string') return [f].concat(args).map(function (x) { return typeof x === 'string' ? x : JSON.stringify(x); }).join(' ');
      var r = f.replace(/%[sdifjo%]/g, function (m) {
        if (m === '%%') return '%'; if (i >= args.length) return m; var v = args[i++];
        if (m === '%s') return String(v); if (m === '%d') return String(Number(v)); if (m === '%i') return String(parseInt(v, 10));
        if (m === '%f') return String(parseFloat(v)); return JSON.stringify(v);
      });
      for (; i < args.length; i++) r += ' ' + (typeof args[i] === 'string' ? args[i] : JSON.stringify(args[i]));
      return r;
    },
    promisify: function (fn) { return function () { var a = Array.prototype.slice.call(arguments), self = this; return new Promise(function (res, rej) { a.push(function (e, v) { if (e) rej(e); else res(v); }); fn.apply(self, a); }); }; },
    inherits: function (c, s) { c.super_ = s; Object.setPrototypeOf(c.prototype, s.prototype); },
    isDeepStrictEqual: function (a, b) { return JSON.stringify(a) === JSON.stringify(b); }
  };
  function makeRequire(dir) {
    return function (name) {
      if (name.slice(0, 5) === 'node:') name = name.slice(5);
      if (name === 'path') return path;
      if (name === 'fs') return fs;
      if (name === 'events') return events;
      if (name === 'util') return util;
      var base = path.resolve(dir, name), c = [base, base + '.js', base + '.json', base + '/index.js'];
      for (var i = 0; i < c.length; i++) if (fs.existsSync(c[i])) return load(c[i]);
      throw new Error("Cannot find module '" + name + "'");
    };
  }
  var timers = [], tseq = 0, tnow = 0;
  function addTimer(fn, ms, args, rep) {
    var t = { id: ++tseq, seq: tseq, due: tnow + ms, fn: fn, args: args, rep: rep };
    timers.push(t);
    return { _id: t.id, ref: function () { return this; }, unref: function () { return this; }, hasRef: function () { return true; } };
  }
  g.setTimeout = function (fn, ms) { return addTimer(fn, Math.max(1, +ms || 0), Array.prototype.slice.call(arguments, 2), null); };
  g.setInterval = function (fn, ms) { ms = Math.max(1, +ms || 0); return addTimer(fn, ms, Array.prototype.slice.call(arguments, 2), ms); };
  g.setImmediate = function (fn) { return addTimer(fn, 0, Array.prototype.slice.call(arguments, 1), null); };
  g.clearTimeout = g.clearInterval = g.clearImmediate = function (h) {
    var id = h && h._id; timers = timers.filter(function (t) { return t.id !== id; });
  };
  g.queueMicrotask = function (fn) { Promise.resolve().then(fn); };
  g.process.nextTick = function (fn) { var a = Array.prototype.slice.call(arguments, 1); Promise.resolve().then(function () { fn.apply(null, a); }); };
  g.__runTimers = function () {
    if (!timers.length) return false;
    timers.sort(function (a, b) { return a.due - b.due || a.seq - b.seq; });
    var t = timers.shift();
    if (t.due > tnow) { __sleep(t.due - tnow); tnow = t.due; }
    if (t.rep !== null) { t.due = tnow + t.rep; t.seq = ++tseq; timers.push(t); }
    t.fn.apply(null, t.args);
    return true;
  };
  g.require = makeRequire(__cwd());
  g.__run = function () {
    var file = path.resolve(__entry_file()), dir = path.dirname(file), m = { exports: {} };
    cache[file] = m;
    new Function('module', 'exports', 'require', '__dirname', '__filename', __entry())(m, m.exports, makeRequire(dir), dir, file);
  };
  g.__rnodejs_load = load;
})(this);
"#;

/// 入口のファイル名の拡張子で TypeScript かどうかを決める
pub fn is_ts_file(name: &str) -> bool {
    name.ends_with(".ts") || name.ends_with(".mts") || name.ends_with(".cts")
}

/// TypeScript なら型注釈を取り除く(feature `ts`)。無ければエラー
pub fn to_js(source: &str, is_ts: bool) -> Result<String, String> {
    if !is_ts {
        return Ok(source.to_string());
    }
    #[cfg(feature = "ts")]
    {
        Ok(rs_typescript::transpile(source))
    }
    #[cfg(not(feature = "ts"))]
    {
        Err("TypeScript を実行するには feature `ts` が必要です(RS-TypeScript で型注釈を取り除きます)".into())
    }
}

#[cfg(feature = "boa")]
pub use boa_backend::BoaEngine;

#[cfg(feature = "boa")]
mod boa_backend {
    use super::{JsEngine, PRELUDE};
    use boa_engine::{js_string, Context, JsError, JsResult, JsValue, NativeFunction, Source};

    fn arg_str(args: &[JsValue], i: usize, ctx: &mut Context) -> JsResult<String> {
        Ok(args.get(i).cloned().unwrap_or(JsValue::undefined()).to_string(ctx)?.to_std_string_escaped())
    }
    fn err(m: String) -> JsError {
        boa_engine::JsNativeError::error().with_message(m).into()
    }

    /// Rust 製の JS エンジン Boa を使う実装
    pub struct BoaEngine {
        ctx: Context,
    }

    impl BoaEngine {
        /// `argv` は `process.argv` になる(先頭に実行ファイル名・スクリプト名を入れる)
        pub fn new(argv: Vec<String>) -> Result<Self, String> {
            let mut ctx = Context::default();
            let reg = |ctx: &mut Context, name: boa_engine::JsString, f: NativeFunction| ctx.register_global_callable(name, 1, f).map_err(|e| e.to_string());
            reg(&mut ctx, js_string!("__print"), NativeFunction::from_fn_ptr(|_, a, c| {
                println!("{}", arg_str(a, 0, c)?);
                Ok(JsValue::undefined())
            }))?;
            reg(&mut ctx, js_string!("__readFile"), NativeFunction::from_fn_ptr(|_, a, c| {
                let p = arg_str(a, 0, c)?;
                match std::fs::read_to_string(&p) {
                    Ok(s) => Ok(JsValue::from(boa_engine::JsString::from(s.as_str()))),
                    Err(e) => Err(err(format!("ENOENT/read error: {p}: {e}"))),
                }
            }))?;
            reg(&mut ctx, js_string!("__writeFile"), NativeFunction::from_fn_ptr(|_, a, c| {
                let (p, d) = (arg_str(a, 0, c)?, arg_str(a, 1, c)?);
                std::fs::write(&p, d).map_err(|e| err(format!("write error: {p}: {e}")))?;
                Ok(JsValue::undefined())
            }))?;
            reg(&mut ctx, js_string!("__entry"), NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::from(boa_engine::JsString::from(super::ENTRY.with(|e| e.borrow().1.clone()).as_str())))))?;
            reg(&mut ctx, js_string!("__entry_file"), NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::from(boa_engine::JsString::from(super::ENTRY.with(|e| e.borrow().0.clone()).as_str())))))?;
            reg(&mut ctx, js_string!("__exit"), NativeFunction::from_fn_ptr(|_, a, c| {
                use std::io::Write;
                let code = a.first().cloned().unwrap_or(JsValue::undefined()).to_number(c)?;
                let _ = std::io::stdout().flush();
                std::process::exit(code as i32)
            }))?;
            reg(&mut ctx, js_string!("__sleep"), NativeFunction::from_fn_ptr(|_, a, c| {
                let ms = a.first().cloned().unwrap_or(JsValue::undefined()).to_number(c)?;
                std::thread::sleep(std::time::Duration::from_millis(ms.max(0.0) as u64));
                Ok(JsValue::undefined())
            }))?;
            reg(&mut ctx, js_string!("__exists"), NativeFunction::from_fn_ptr(|_, a, c| Ok(JsValue::from(std::path::Path::new(&arg_str(a, 0, c)?).exists()))))?;
            reg(&mut ctx, js_string!("__cwd"), NativeFunction::from_fn_ptr(|_, _, _| {
                let d = std::env::current_dir().map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| "/".into());
                Ok(JsValue::from(boa_engine::JsString::from(d.as_str())))
            }))?;
            reg(&mut ctx, js_string!("__env"), NativeFunction::from_fn_ptr(|_, a, c| {
                Ok(match std::env::var(arg_str(a, 0, c)?) {
                    Ok(v) => JsValue::from(boa_engine::JsString::from(v.as_str())),
                    Err(_) => JsValue::undefined(),
                })
            }))?;
            let argv_json = format!("[{}]", argv.iter().map(|s| format!("{s:?}")).collect::<Vec<_>>().join(","));
            let argv_src = format!("globalThis.__argv = function () {{ return {argv_json}; }};");
            ctx.eval(Source::from_bytes(argv_src.as_bytes())).map_err(|e| e.to_string())?;
            ctx.eval(Source::from_bytes(PRELUDE.as_bytes())).map_err(|e| e.to_string())?;
            Ok(BoaEngine { ctx })
        }
    }

    impl JsEngine for BoaEngine {
        fn eval(&mut self, source: &str) -> Result<String, String> {
            let v = self.ctx.eval(Source::from_bytes(source.as_bytes())).map_err(|e| e.to_string())?;
            let _ = self.ctx.run_jobs();
            // タイマー: 満期順に 1 つずつ実行し、そのたびに Promise のジョブを流す(Node のイベントループ相当)
            loop {
                let more = self.ctx.eval(Source::from_bytes(b"__runTimers()")).map_err(|e| e.to_string())?;
                let _ = self.ctx.run_jobs();
                if !more.to_boolean() {
                    break;
                }
            }
            v.to_string(&mut self.ctx).map(|s| s.to_std_string_escaped()).map_err(|e| e.to_string())
        }
    }
}

thread_local! { pub(crate) static ENTRY: std::cell::RefCell<(String, String)> = const { std::cell::RefCell::new((String::new(), String::new())) }; }

/// ソース(JS または TS)を、エンジンで実行する(式として評価)
pub fn run_source(engine: &mut dyn JsEngine, source: &str, is_ts: bool) -> Result<String, String> {
    engine.eval(&to_js(source, is_ts)?)
}

/// 入口のファイルを Node.js と同じように実行する(`require` は、そのファイルのあるフォルダー基準。`module` / `exports` / `__dirname` / `__filename` あり)
pub fn run_file(engine: &mut dyn JsEngine, file: &str, source: &str) -> Result<(), String> {
    let js = to_js(source, is_ts_file(file))?;
    if std::env::var_os("RNODEJS_EMIT").is_some() {
        eprintln!("{js}");
    }
    ENTRY.with(|e| *e.borrow_mut() = (file.to_string(), js));
    engine.eval("__run()").map(|_| ())
}
