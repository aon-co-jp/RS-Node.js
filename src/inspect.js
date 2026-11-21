// util.inspect / util.format / console.log の整形(Node.js の lib/internal/util/inspect.js の主な動きを小さく移したもの)。
// 対応: プリミティブ・配列・オブジェクト・入れ子(depth 2)・Map・Set・Date・RegExp・Error・関数・クラス・Symbol・BigInt・循環参照・
// 72桁の折り返し(breakLength 80、compact 3)・配列の桁そろえ(7要素以上)・100要素/1000文字の省略。
// 未対応(出力が本物の Node と違う): Proxy・TypedArray の詳細・getter の表示・showHidden・colors・WeakMap の中身・ArrayBuffer。
(function (g) {
  var kObject = 0, kArray = 1, kArrayExtras = 2;
  var identRe = /^[a-zA-Z_][a-zA-Z_0-9]*$/;
  var escMap = { '\b': '\\b', '\t': '\\t', '\n': '\\n', '\f': '\\f', '\r': '\\r', '\\': '\\\\' };

  function quote(s) {
    var q = "'";
    if (s.indexOf("'") !== -1) { if (s.indexOf('"') === -1) q = '"'; else if (s.indexOf('`') === -1 && s.indexOf('${') === -1) q = '`'; }
    var out = '';
    for (var i = 0; i < s.length; i++) {
      var c = s.charAt(i), code = s.charCodeAt(i);
      if (c === q) out += '\\' + c;
      else if (escMap[c] !== undefined) out += escMap[c];
      else if (code < 0x20 || code === 0x7f) out += '\\x' + (code < 16 ? '0' : '') + code.toString(16).toUpperCase();
      else out += c;
    }
    return q + out + q;
  }

  function newCtx(o) {
    o = o || {};
    return {
      depth: o.depth === undefined ? 2 : o.depth, breakLength: o.breakLength === undefined ? 80 : o.breakLength,
      compact: o.compact === undefined ? 3 : o.compact, maxArrayLength: o.maxArrayLength === undefined ? 100 : o.maxArrayLength,
      maxStringLength: o.maxStringLength === undefined ? 10000 : o.maxStringLength, sorted: !!o.sorted, showHidden: !!o.showHidden,
      seen: [], circular: null, indentationLvl: 0, currentDepth: 0
    };
  }

  function typeOf(v) { return Object.prototype.toString.call(v).slice(8, -1); }
  function ctorName(v) {
    var o = v;
    while (o !== null && o !== undefined) {
      var d = Object.getOwnPropertyDescriptor(o, 'constructor');
      if (d && typeof d.value === 'function' && d.value.name !== '') return d.value.name;
      o = Object.getPrototypeOf(o);
    }
    return null;
  }

  function formatPrimitive(v, ctx) {
    if (typeof v === 'string') {
      var t = '';
      if (v.length > ctx.maxStringLength) { t = '... ' + (v.length - ctx.maxStringLength) + ' more character' + (v.length - ctx.maxStringLength > 1 ? 's' : ''); v = v.slice(0, ctx.maxStringLength); }
      return quote(v) + t;
    }
    if (typeof v === 'number') return (v === 0 && 1 / v === -Infinity) ? '-0' : String(v);
    if (typeof v === 'bigint') return String(v) + 'n';
    if (typeof v === 'symbol') return String(v);
    return String(v); // undefined / null / boolean
  }

  function formatValue(ctx, v, recurseTimes, typedArray) {
    if (typeof v !== 'object' && typeof v !== 'function') return formatPrimitive(v, ctx);
    if (v === null) return 'null';
    if (ctx.seen.indexOf(v) !== -1) {
      var idx = 1;
      if (ctx.circular === null) { ctx.circular = [v]; } else { idx = ctx.circular.indexOf(v); if (idx === -1) { ctx.circular.push(v); idx = ctx.circular.length; } else idx++; }
      return '[Circular *' + idx + ']';
    }
    return formatRaw(ctx, v, recurseTimes);
  }

  function keysOf(v) {
    var ks = Object.keys(v);
    if (ctxSorted) ks.sort();
    return ks;
  }
  var ctxSorted = false;

  function getPrefix(constructor, tag, fallback, size) {
    if (constructor === null) {
      if (tag !== '' && fallback !== tag) return '[' + fallback + (size !== undefined ? '(' + size + ')' : '') + ': null prototype] [' + tag + '] ';
      return '[' + fallback + (size !== undefined ? '(' + size + ')' : '') + ': null prototype] ';
    }
    if (tag !== '' && constructor !== tag) return constructor + (size !== undefined ? '(' + size + ')' : '') + ' [' + tag + '] ';
    return constructor + (size !== undefined ? '(' + size + ')' : '') + ' ';
  }

  function formatKey(k) {
    if (typeof k === 'symbol') return '[' + String(k) + ']';
    if (identRe.test(k)) return k;
    return quote(k);
  }

  function formatProperty(ctx, v, recurseTimes, key, type) {
    var desc = Object.getOwnPropertyDescriptor(v, key) || { value: v[key], enumerable: true };
    var str;
    if (desc.value !== undefined || (!desc.get && !desc.set)) {
      ctx.indentationLvl += 2;
      str = formatValue(ctx, desc.value, recurseTimes);
      ctx.indentationLvl -= 2;
    } else if (desc.get) {
      str = desc.set ? '[Getter/Setter]' : '[Getter]';
    } else {
      str = '[Setter]';
    }
    if (type === kArrayExtras) return str;
    return formatKey(key) + ': ' + str;
  }

  function isBelowBreakLength(ctx, output, start, base) {
    var totalLength = output.length + start;
    if (totalLength + output.length > ctx.breakLength) return false;
    for (var i = 0; i < output.length; i++) {
      totalLength += output[i].length;
      if (totalLength > ctx.breakLength) return false;
    }
    return base === '' || base.indexOf('\n') === -1;
  }

  function groupArrayElements(ctx, output, value) {
    var totalLength = 0, maxLength = 0, i = 0, outputLength = output.length;
    if (ctx.maxArrayLength < output.length) outputLength--;
    var separatorSpace = 2, dataLen = new Array(outputLength);
    for (; i < outputLength; i++) {
      var len = output[i].length;
      dataLen[i] = len; totalLength += len + separatorSpace;
      if (maxLength < len) maxLength = len;
    }
    var actualMax = maxLength + separatorSpace;
    if (actualMax * 3 + ctx.indentationLvl < ctx.breakLength && (totalLength / actualMax > 5 || maxLength <= 6)) {
      var averageBias = Math.sqrt(actualMax - totalLength / output.length);
      var biasedMax = Math.max(actualMax - 3 - averageBias, 1);
      var columns = Math.min(Math.round(Math.sqrt(2.5 * biasedMax * outputLength) / biasedMax), Math.floor((ctx.breakLength - ctx.indentationLvl) / actualMax), ctx.compact * 4, 15);
      if (columns <= 1) return output;
      var tmp = [], maxLineLength = [];
      for (var c = 0; c < columns; c++) {
        var lineLength = 0;
        for (var j = c; j < output.length; j += columns) { if (dataLen[j] > lineLength) lineLength = dataLen[j]; }
        maxLineLength.push(lineLength + separatorSpace);
      }
      var padStart = true;
      if (value !== undefined) {
        for (var k = 0; k < output.length; k++) { if (typeof value[k] !== 'number' && typeof value[k] !== 'bigint') { padStart = false; break; } }
      }
      function pad(s, n) { while (s.length < n) s = padStart ? ' ' + s : s + ' '; return s; }
      for (var r = 0; r < outputLength; r += columns) {
        var max = Math.min(r + columns, outputLength), str = '', jj = r;
        for (; jj < max - 1; jj++) str += pad(output[jj] + ', ', maxLineLength[jj - r]);
        if (padStart) str += pad(output[jj], maxLineLength[jj - r] - separatorSpace); else str += output[jj];
        tmp.push(str);
      }
      if (ctx.maxArrayLength < output.length) tmp.push(output[outputLength]);
      output = tmp;
    }
    return output;
  }

  function reduceToSingleString(ctx, output, base, braces, extrasType, recurseTimes, value) {
    if (ctx.compact !== true) {
      if (typeof ctx.compact === 'number' && ctx.compact >= 1) {
        var entries = output.length;
        if (extrasType === kArrayExtras && entries > 6) output = groupArrayElements(ctx, output, value);
        if (ctx.currentDepth - recurseTimes < ctx.compact && entries === output.length) {
          var start = output.length + ctx.indentationLvl + braces[0].length + base.length + 10;
          if (isBelowBreakLength(ctx, output, start, base)) {
            var joined = output.join(', ');
            if (joined.indexOf('\n') === -1) return (base ? base + ' ' : '') + braces[0] + ' ' + joined + ' ' + braces[1];
          }
        }
      }
      var indentation = '\n' + ' '.repeat(ctx.indentationLvl);
      return (base ? base + ' ' : '') + braces[0] + indentation + '  ' + output.join(',' + indentation + '  ') + indentation + braces[1];
    }
    if (isBelowBreakLength(ctx, output, 0, base)) return braces[0] + (base ? ' ' + base : '') + ' ' + output.join(', ') + ' ' + braces[1];
    var ind = '\n' + ' '.repeat(ctx.indentationLvl);
    return (base ? base + ' ' : '') + braces[0] + ind + '  ' + output.join(',' + ind + '  ') + ind + braces[1];
  }

  function formatRaw(ctx, value, recurseTimes) {
    var keys, protoProps, constructor = ctorName(value), tag = typeOf(value);
    if (tag === value_tag_skip) tag = '';
    var base = '', formatter = null, braces, noIterator = true, extrasType = kObject, output;
    var isArr = Array.isArray(value), isMap = value instanceof Map, isSet = value instanceof Set;
    var symTag = value[Symbol.toStringTag];
    var tagStr = (typeof symTag === 'string' && symTag !== '' && symTag !== constructor) ? symTag : '';

    keys = Object.keys(value);
    if (ctx.sorted) keys.sort();
    var syms = Object.getOwnPropertySymbols ? Object.getOwnPropertySymbols(value).filter(function (s) { var d = Object.getOwnPropertyDescriptor(value, s); return d && d.enumerable; }) : [];
    keys = keys.concat(syms);

    if (isArr) {
      var prefix = (constructor !== 'Array' || tagStr !== '') ? getPrefix(constructor, tagStr, 'Array', value.length) : '';
      keys = keys.filter(function (k) { return typeof k === 'symbol' || !(/^(0|[1-9][0-9]*)$/.test(k) && +k < value.length); });
      braces = [prefix + '[', ']'];
      if (value.length === 0 && keys.length === 0) return braces[0] + ']';
      extrasType = kArrayExtras; formatter = 'array';
    } else if (isSet) {
      var size = value.size, prefix2 = getPrefix(constructor, tagStr, 'Set', size);
      if (size === 0 && keys.length === 0) return prefix2 + '{}';
      braces = [prefix2 + '{', '}']; formatter = 'set';
    } else if (isMap) {
      var size2 = value.size, prefix3 = getPrefix(constructor, tagStr, 'Map', size2);
      if (size2 === 0 && keys.length === 0) return prefix3 + '{}';
      braces = [prefix3 + '{', '}']; formatter = 'map';
    } else {
      braces = ['{', '}'];
      if (typeof value === 'function') {
        var fn = Function.prototype.toString.call(value), nm = value.name;
        var pd = Object.getOwnPropertyDescriptor(value, 'prototype');
        if (/^class[\s{]/.test(fn) || (pd && pd.writable === false)) {
          base = '[class ' + (nm || '(anonymous)');
          var sup = Object.getPrototypeOf(value);
          if (sup && sup.name && sup !== Function.prototype) base += ' extends ' + sup.name;
          base += ']';
        } else {
          var kind = (tag === 'AsyncFunction' || /^async\s/.test(fn)) ? 'AsyncFunction' : ((tag === 'GeneratorFunction' || /^function\s*\*/.test(fn)) ? 'GeneratorFunction' : 'Function');
          base = '[' + kind + (nm ? ': ' + nm : ' (anonymous)') + ']';
        }
        if (constructor === null) base += ' [null prototype]';
        if (keys.length === 0) return base;
      } else if (value instanceof RegExp) {
        base = RegExp.prototype.toString.call(value);
        if (keys.length === 0) return base;
      } else if (value instanceof Date) {
        base = isNaN(value.getTime()) ? 'Invalid Date' : Date.prototype.toISOString.call(value);
        if (keys.length === 0) return base;
      } else if (value instanceof Error) {
        var stack = value.stack;
        base = (typeof stack === 'string' && stack !== '') ? stack : (Error.prototype.toString.call(value));
        keys = keys.filter(function (k) { return k !== 'stack' && k !== 'message'; });
        if (ctx.indentationLvl !== 0) base = base.split('\n').join('\n' + ' '.repeat(ctx.indentationLvl));
        if (keys.length === 0) return base;
      } else if (tag === 'Number' || tag === 'String' || tag === 'Boolean' || tag === 'BigInt' || tag === 'Symbol') {
        var inner = value.valueOf();
        base = '[' + tag + ': ' + formatPrimitive(inner, ctx) + ']';
        if (keys.length === 0) return base;
      } else if (tag === 'Promise') {
        base = '';
        braces[0] = getPrefix(constructor, tagStr, 'Promise') + '{';
        formatter = 'promise';
      } else {
        if (keys.length === 0) return (constructor === 'Object' && tagStr === '' ? '' : getPrefix(constructor, tagStr, 'Object')) + '{}';
        braces[0] = (constructor === 'Object' && tagStr === '') ? '{' : getPrefix(constructor, tagStr, 'Object') + '{';
      }
    }

    if (recurseTimes > ctx.depth && ctx.depth !== null) {
      var nm2 = constructor === null ? '[Object: null prototype]' : (constructor || 'Object');
      return '[' + (isArr ? 'Array' : nm2) + ']';
    }
    recurseTimes += 1;
    ctx.seen.push(value);
    ctx.currentDepth = recurseTimes;
    output = [];
    if (formatter === 'array') {
      var outLen = Math.min(Math.max(0, ctx.maxArrayLength), value.length), remaining = value.length - outLen;
      for (var i = 0; i < outLen; i++) {
        if (!(i in value)) {
          var h = i, holes = 0; while (h < value.length && !(h in value)) { holes++; h++; }
          output.push('<' + holes + ' empty item' + (holes > 1 ? 's' : '') + '>'); i = h - 1; continue;
        }
        output.push(formatProperty(ctx, value, recurseTimes, String(i), kArrayExtras));
      }
      if (remaining > 0) output.push('... ' + remaining + ' more item' + (remaining > 1 ? 's' : ''));
    } else if (formatter === 'set') {
      ctx.indentationLvl += 2;
      value.forEach(function (v) { output.push(formatValue(ctx, v, recurseTimes)); });
      ctx.indentationLvl -= 2;
    } else if (formatter === 'map') {
      ctx.indentationLvl += 2;
      value.forEach(function (v, k) { output.push(formatValue(ctx, k, recurseTimes) + ' => ' + formatValue(ctx, v, recurseTimes)); });
      ctx.indentationLvl -= 2;
    } else if (formatter === 'promise') {
      output.push('<pending>');
    }
    if (formatter === 'array' && ctx.showHidden) output.push('[length]: ' + value.length);
    for (var j = 0; j < keys.length; j++) output.push(formatProperty(ctx, value, recurseTimes, keys[j], kObject));
    ctx.seen.pop();

    if (ctx.circular !== null) {
      var index = ctx.circular.indexOf(value);
      if (index !== -1) {
        var ref = '<ref *' + (index + 1) + '>';
        if (ctx.compact !== true) base = base === '' ? ref : ref + ' ' + base; else braces[0] = ref + ' ' + braces[0];
      }
    }
    var res = reduceToSingleString(ctx, output, base, braces, extrasType, recurseTimes, isArr ? value : undefined);
    return res;
  }
  var value_tag_skip = '\u0000';

  function inspect(v, opts) {
    var o = opts;
    if (typeof opts !== 'object' || opts === null) { o = {}; if (arguments.length > 1 && typeof arguments[1] === 'boolean') o.showHidden = arguments[1]; if (arguments.length > 2 && arguments[2] !== undefined) o.depth = arguments[2]; }
    var ctx = newCtx(o);
    if (ctx.depth === Infinity) ctx.depth = null;
    return formatValue(ctx, v, 0);
  }
  inspect.custom = Symbol.for('nodejs.util.inspect.custom');

  function format(f) {
    var args = Array.prototype.slice.call(arguments), i = 0, str = '';
    function ins(x) { return inspect(x, { depth: 0, colors: false, compact: 3 }); }
    if (typeof f === 'string') {
      if (args.length === 1) return f;
      i = 1;
      var out = '', last = 0;
      for (var p = 0; p < f.length - 1; p++) {
        if (f.charCodeAt(p) !== 37) continue;
        var c = f.charAt(p + 1);
        if (c === '%') { out += f.slice(last, p) + '%'; last = p + 2; p++; continue; }
        if (i >= args.length) continue;
        var rep;
        var a = args[i];
        if (c === 's') rep = typeof a === 'string' ? a : (typeof a === 'number' ? ((a === 0 && 1 / a === -Infinity) ? '-0' : String(a)) : (typeof a === 'bigint' ? a + 'n' : (typeof a === 'object' && a !== null ? inspect(a, { depth: 0 }) : String(a))));
        else if (c === 'd') rep = typeof a === 'bigint' ? a + 'n' : (typeof a === 'symbol' ? 'NaN' : (typeof a === 'object' && a !== null ? 'NaN' : String(Number(a))));
        else if (c === 'i') rep = typeof a === 'bigint' ? a + 'n' : (typeof a === 'symbol' ? 'NaN' : String(parseInt(a, 10)));
        else if (c === 'f') rep = typeof a === 'symbol' ? 'NaN' : String(parseFloat(a));
        else if (c === 'j') { try { rep = JSON.stringify(a); } catch (e) { rep = '[Circular]'; } }
        else if (c === 'o') rep = inspect(a, { depth: 4, showHidden: true });
        else if (c === 'O') rep = inspect(a);
        else if (c === 'c') rep = '';
        else continue;
        out += f.slice(last, p) + rep; last = p + 2; p++; i++;
      }
      str = out + f.slice(last);
    } else {
      i = 0;
    }
    for (; i < args.length; i++) {
      var x = args[i];
      if (i > 0 || typeof f === 'string') str += ' ';
      str += typeof x === 'string' ? x : inspect(x);
    }
    return str;
  }

  g.__inspect = inspect;
  g.__format = format;
})(this);
