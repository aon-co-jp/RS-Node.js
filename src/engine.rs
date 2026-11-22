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

/// util.inspect / util.format の整形(src/inspect.js)。PRELUDE より先に評価する
pub const INSPECT_JS: &str = include_str!("inspect.js");

/// ES モジュール(import / export)を CommonJS へ書き換える変換(src/esm.js)。PRELUDE より先に評価する
pub const ESM_JS: &str = include_str!("esm.js");

/// Node.js 風の土台(JS)。ネイティブ関数 `__print` `__readFile` `__writeFile` `__exists` `__cwd` `__env` を使う。
pub const PRELUDE: &str = r#"
(function (g) {
  g.console = { log: function () { __print(g.__format.apply(null, arguments)); } };
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
    resolve: function () { var r = ''; for (var i = arguments.length - 1; i >= 0 && r.charAt(0) !== '/' && !/^[A-Za-z]:[\/]/.test(r); i--) r = arguments[i] + (r ? '/' + r : ''); if (r.charAt(0) !== '/' && !/^[A-Za-z]:[\/]/.test(r)) r = __cwd() + '/' + r; return path.normalize(r); },
    dirname: function (p) { var i = p.lastIndexOf('/'); return i < 0 ? '.' : (i === 0 ? '/' : p.slice(0, i)); },
    basename: function (p, ext) { var b = p.slice(p.lastIndexOf('/') + 1); return ext && b.slice(-ext.length) === ext ? b.slice(0, -ext.length) : b; },
    extname: function (p) { var b = path.basename(p), i = b.lastIndexOf('.'); return i <= 0 ? '' : b.slice(i); }
  };
  path.posix = path;
  var fs = {
    readFileSync: function (p, o) { var s = __readFile(String(p)); var e = typeof o === 'string' ? o : (o && o.encoding); return e ? s : Buffer.from(s); },
    writeFileSync: function (p, d) { __writeFile(String(p), String(d)); },
    existsSync: function (p) { return __exists(String(p)); },
    statSync: function (p) {
      p = String(p);
      if (!__exists(p)) { var e = new Error("ENOENT: no such file or directory, stat '" + p + "'"); e.code = 'ENOENT'; throw e; }
      var f = __isFile(p);
      return { isFile: function () { return f; }, isDirectory: function () { return !f; } };
    }
  };
  function cbify(f) {
    return function () {
      var a = Array.prototype.slice.call(arguments), cb = a.pop();
      setImmediate(function () { var r, e = null; try { r = f.apply(null, a); } catch (x) { e = x; } cb(e, r); });
    };
  }
  fs.readFile = cbify(function (p, o) { return fs.readFileSync(p, o); });
  fs.writeFile = cbify(function (p, d) { fs.writeFileSync(p, d); });
  fs.exists = function (p, cb) { setImmediate(function () { cb(fs.existsSync(p)); }); };
  fs.promises = {
    readFile: function (p, o) { return new Promise(function (res, rej) { fs.readFile(p, o, function (e, v) { if (e) rej(e); else res(v); }); }); },
    writeFile: function (p, d) { return new Promise(function (res, rej) { fs.writeFile(p, d, function (e) { if (e) rej(e); else res(); }); }); }
  };
  var os = { EOL: String.fromCharCode(10), platform: function () { return 'linux'; }, tmpdir: function () { return '/tmp'; }, homedir: function () { return __env('HOME') || '/'; } };
  var cache = {};
  function load(file) {
    if (cache[file]) return cache[file].exports;
    var m = { exports: {} }; cache[file] = m;
    var src = __readFile(file);
    if (/\.mjs$/.test(file) || (!/\.json$/.test(file) && g.__isEsm(src))) src = g.__esm(src);
    if (/\.json$/.test(file)) { m.exports = JSON.parse(src); return m.exports; }
    new Function('module', 'exports', 'require', '__dirname', '__filename', src)(m, m.exports, makeRequire(path.dirname(file)), path.dirname(file), file);
    return m.exports;
  }
  function utf8enc(str) {
    var o = [];
    for (var i = 0; i < str.length; i++) {
      var c = str.charCodeAt(i);
      if (c >= 0xd800 && c < 0xdc00 && i + 1 < str.length) { c = 0x10000 + ((c - 0xd800) << 10) + (str.charCodeAt(++i) - 0xdc00); }
      if (c < 0x80) o.push(c);
      else if (c < 0x800) o.push(0xc0 | (c >> 6), 0x80 | (c & 63));
      else if (c < 0x10000) o.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
      else o.push(0xf0 | (c >> 18), 0x80 | ((c >> 12) & 63), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
    }
    return o;
  }
  function utf8dec(b) {
    var s = '', i = 0;
    while (i < b.length) {
      var c = b[i++];
      if (c >= 0xf0) { c = ((c & 7) << 18) | ((b[i++] & 63) << 12) | ((b[i++] & 63) << 6) | (b[i++] & 63); c -= 0x10000; s += String.fromCharCode(0xd800 + (c >> 10), 0xdc00 + (c & 1023)); continue; }
      if (c >= 0xe0) c = ((c & 15) << 12) | ((b[i++] & 63) << 6) | (b[i++] & 63);
      else if (c >= 0xc0) c = ((c & 31) << 6) | (b[i++] & 63);
      s += String.fromCharCode(c);
    }
    return s;
  }
  var B64 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  function mkbuf(arr) { var u = new Uint8Array(arr); Object.setPrototypeOf(u, Buffer.prototype); return u; }
  function Buffer() { throw new TypeError('Use Buffer.from() or Buffer.alloc()'); }
  Buffer.prototype = Object.create(Uint8Array.prototype, { constructor: { value: Buffer } });
  Buffer.from = function (v, enc) {
    if (typeof v === 'string') {
      enc = enc || 'utf8';
      if (enc === 'hex') { var h = []; for (var i = 0; i + 1 < v.length; i += 2) h.push(parseInt(v.slice(i, i + 2), 16)); return mkbuf(h); }
      if (enc === 'base64') {
        var bits = 0, n = 0, o = [], t = v.replace(/=+$/, '');
        for (var j = 0; j < t.length; j++) { var k = B64.indexOf(t.charAt(j)); if (k < 0) continue; bits = (bits << 6) | k; n += 6; if (n >= 8) { n -= 8; o.push((bits >> n) & 255); } }
        return mkbuf(o);
      }
      return mkbuf(utf8enc(v));
    }
    return mkbuf(Array.prototype.slice.call(v));
  };
  Buffer.alloc = function (n, fill) { var b = mkbuf(n); if (fill !== undefined) b.fill(typeof fill === 'string' ? fill.charCodeAt(0) : fill); return b; };
  Buffer.isBuffer = function (x) { return x instanceof Buffer; };
  Buffer.byteLength = function (s) { return typeof s === 'string' ? utf8enc(s).length : s.length; };
  Buffer.concat = function (l) { var o = []; l.forEach(function (b) { for (var i = 0; i < b.length; i++) o.push(b[i]); }); return mkbuf(o); };
  Buffer.prototype.toString = function (enc) {
    enc = enc || 'utf8';
    if (enc === 'hex') { var h = ''; for (var i = 0; i < this.length; i++) h += (this[i] < 16 ? '0' : '') + this[i].toString(16); return h; }
    if (enc === 'base64') {
      var r = '', i2 = 0;
      for (; i2 + 2 < this.length; i2 += 3) { var x = (this[i2] << 16) | (this[i2 + 1] << 8) | this[i2 + 2]; r += B64.charAt(x >> 18) + B64.charAt((x >> 12) & 63) + B64.charAt((x >> 6) & 63) + B64.charAt(x & 63); }
      var rem = this.length - i2;
      if (rem === 1) { var y = this[i2] << 16; r += B64.charAt(y >> 18) + B64.charAt((y >> 12) & 63) + '=='; }
      else if (rem === 2) { var z = (this[i2] << 16) | (this[i2 + 1] << 8); r += B64.charAt(z >> 18) + B64.charAt((z >> 12) & 63) + B64.charAt((z >> 6) & 63) + '='; }
      return r;
    }
    return utf8dec(this);
  };
  Buffer.prototype.toJSON = function () { return { type: 'Buffer', data: Array.prototype.slice.call(this) }; };
  Buffer.prototype.equals = function (o) { if (this.length !== o.length) return false; for (var i = 0; i < this.length; i++) if (this[i] !== o[i]) return false; return true; };
  Buffer.prototype.slice = Buffer.prototype.subarray = function (a, b) { return mkbuf(Array.prototype.slice.call(this, a, b)); };
  Buffer.prototype.readUInt8 = function (i) { return this[i]; };
  Buffer.prototype.writeUInt8 = function (v, i) { this[i] = v; return i + 1; };
  g.Buffer = Buffer;
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
    format: g.__format,
    inspect: g.__inspect,
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
      if (name === 'os') return os;
      if (name === 'fs/promises') return fs.promises;
      if (name === 'buffer') return { Buffer: Buffer };
      if (name === 'util') return util;
      var rel = name.charAt(0) === '.' || name.charAt(0) === '/';
      var bases = [];
      if (rel) bases.push(path.resolve(dir, name));
      else {
        var d = path.resolve(dir);
        for (;;) {
          if (path.basename(d) !== 'node_modules') bases.push(d === '/' ? '/node_modules/' + name : d + '/node_modules/' + name);
          var up = path.dirname(d);
          if (up === d) break;
          d = up;
        }
      }
      for (var b = 0; b < bases.length; b++) {
        var hit = resolveBase(bases[b]);
        if (hit) return load(hit);
      }
      var err = new Error("Cannot find module '" + name + "'");
      err.code = 'MODULE_NOT_FOUND';
      throw err;
    };
  }
  function isFile(p) { try { return fs.statSync(p).isFile(); } catch (e) { return false; } }
  function resolveIndex(b) {
    var x = ['/index.js', '/index.mjs', '/index.json'];
    for (var i = 0; i < x.length; i++) if (isFile(b + x[i])) return b + x[i];
    return null;
  }
  function resolveBase(base) {
    var c = [base, base + '.js', base + '.mjs', base + '.json'];
    for (var i = 0; i < c.length; i++) if (isFile(c[i])) return c[i];
    if (isFile(base + '/package.json')) {
      var main = null;
      try { main = JSON.parse(fs.readFileSync(base + '/package.json', 'utf8')).main; } catch (e) {}
      if (typeof main === 'string') {
        var mb = path.resolve(base, main);
        var h = [mb, mb + '.js', mb + '.json'];
        for (var j = 0; j < h.length; j++) if (isFile(h[j])) return h[j];
        var ix = resolveIndex(mb); if (ix) return ix;
      }
    }
    return resolveIndex(base);
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
    var esrc = __entry(); if (/\.mjs$/.test(file) || g.__isEsm(esrc)) esrc = g.__esm(esrc);
    new Function('module', 'exports', 'require', '__dirname', '__filename', esrc)(m, m.exports, makeRequire(dir), dir, file);
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
    use super::{JsEngine, ESM_JS, INSPECT_JS, PRELUDE};
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
            reg(&mut ctx, js_string!("__isFile"), NativeFunction::from_fn_ptr(|_, a, c| Ok(JsValue::from(std::path::Path::new(&arg_str(a, 0, c)?).is_file()))))?;
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
            ctx.eval(Source::from_bytes(INSPECT_JS.as_bytes())).map_err(|e| e.to_string())?;
            ctx.eval(Source::from_bytes(ESM_JS.as_bytes())).map_err(|e| e.to_string())?;
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
