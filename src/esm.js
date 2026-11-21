// ES モジュール(import / export)を CommonJS へ書き換える小さな変換。本物のモジュールローダーではなく、よく使う書き方だけを扱う。
// 対応: import 既定/名前付き/名前空間/副作用のみ、export const|let|var|function|class|async function、export default、export { a, b as c }、export { x } from 'm'、export * from 'm'。
// 未対応(動かない): トップレベル await、import.meta、動的 import()、ライブバインディング(書き出した値が後で変わっても import 側には反映されない)、
//                    文字列やコメントの中にある "import ..." のような行の誤検出。
(function (g) {
  var IMPORT = /(^|[\n;])[ \t]*import\s+([^'"();]*?)\s*from\s*(['"])([^'"\n]+)\3[ \t]*;?/g;
  var IMPORT_BARE = /(^|[\n;])[ \t]*import\s*(['"])([^'"\n]+)\2[ \t]*;?/g;
  var EXPORT_FROM = /(^|[\n;])[ \t]*export\s*\{([^}]*)\}\s*from\s*(['"])([^'"\n]+)\3[ \t]*;?/g;
  var EXPORT_STAR = /(^|[\n;])[ \t]*export\s*\*\s*from\s*(['"])([^'"\n]+)\2[ \t]*;?/g;
  var EXPORT_LIST = /(^|[\n;])[ \t]*export\s*\{([^}]*)\}[ \t]*;?/g;
  var EXPORT_DECL = /(^|[\n;])[ \t]*export\s+(async\s+function\s*\*?|function\s*\*?|class|const|let|var)\s+([A-Za-z_$][\w$]*)/g;
  var EXPORT_DEFAULT_NAMED = /(^|[\n;])[ \t]*export\s+default\s+(async\s+function\s*\*?|function\s*\*?|class)\s+([A-Za-z_$][\w$]*)/g;
  var EXPORT_DEFAULT = /(^|[\n;])[ \t]*export\s+default\s+/g;

  var n = 0;
  function names(list) {
    return list.split(',').map(function (s) { return s.trim(); }).filter(Boolean).map(function (s) {
      var m = /^([\w$]+)\s+as\s+([\w$]+)$/.exec(s);
      return m ? [m[1], m[2]] : [s, s];
    });
  }

  g.__isEsm = function (src) { return /(^|[\n;])[ \t]*(import\s*[\w$*{'"]|export\s+(default|const|let|var|function|class|async|\{|\*))/.test(src); };

  g.__esm = function (src) {
    var tail = [];
    var out = src;
    out = out.replace(IMPORT, function (_, pre, clause, q, spec) {
      var id = '__m' + (++n), lines = [], c = clause.trim();
      lines.push('var ' + id + ' = require(' + q + spec + q + ');');
      var rest = c, m;
      if ((m = /^\*\s*as\s+([\w$]+)$/.exec(rest))) { lines.push('var ' + m[1] + ' = ' + id + ';'); return pre + lines.join(' '); }
      if ((m = /^([\w$]+)\s*,\s*(.*)$/.exec(rest))) { lines.push('var ' + m[1] + ' = ' + id + ' && ' + id + '.__esModule ? ' + id + '.default : ' + id + ';'); rest = m[2].trim(); }
      else if ((m = /^([\w$]+)$/.exec(rest))) { lines.push('var ' + m[1] + ' = ' + id + ' && ' + id + '.__esModule ? ' + id + '.default : ' + id + ';'); return pre + lines.join(' '); }
      if ((m = /^\*\s*as\s+([\w$]+)$/.exec(rest))) { lines.push('var ' + m[1] + ' = ' + id + ';'); }
      else if ((m = /^\{([\s\S]*)\}$/.exec(rest))) { names(m[1]).forEach(function (p) { lines.push('var ' + p[1] + ' = ' + id + '.' + p[0] + ';'); }); }
      return pre + lines.join(' ');
    });
    out = out.replace(IMPORT_BARE, function (_, pre, q, spec) { return pre + 'require(' + q + spec + q + ');'; });
    out = out.replace(EXPORT_FROM, function (_, pre, list, q, spec) {
      var id = '__m' + (++n), s = 'var ' + id + ' = require(' + q + spec + q + ');';
      names(list).forEach(function (p) { s += ' exports.' + p[1] + ' = ' + id + '.' + p[0] + ';'; });
      return pre + s;
    });
    out = out.replace(EXPORT_STAR, function (_, pre, q, spec) {
      var id = '__m' + (++n);
      return pre + 'var ' + id + ' = require(' + q + spec + q + '); Object.keys(' + id + ').forEach(function (k) { if (k !== "default" && k !== "__esModule") exports[k] = ' + id + '[k]; });';
    });
    out = out.replace(EXPORT_LIST, function (_, pre, list) {
      names(list).forEach(function (p) { tail.push('exports.' + p[1] + ' = ' + p[0] + ';'); });
      return pre;
    });
    out = out.replace(EXPORT_DEFAULT_NAMED, function (_, pre, kw, name) { tail.push('exports.default = ' + name + ';'); return pre + kw + ' ' + name; });
    out = out.replace(EXPORT_DEFAULT, function (_, pre) { return pre + 'exports.default = '; });
    out = out.replace(EXPORT_DECL, function (_, pre, kw, name) {
      tail.push('exports.' + name + ' = ' + name + ';');
      return pre + kw + ' ' + name;
    });
    return 'Object.defineProperty(exports, "__esModule", { value: true });\n' + out + '\n' + tail.join('\n');
  };
})(this);
