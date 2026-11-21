// console.log / util.inspect の整形(本物の Node.js と出力を比べる)
var util = require('util');
console.log({ a: 1, b: 'x', c: [1, 2, 3], d: { e: null, f: undefined } });
console.log([1, 2, 3], ['a', 'b'], [], {}, [[]], [{}]);
console.log('plain', 42, -0, 1.5, true, null, undefined, 10n);
console.log({ s: "it's", t: 'say "hi"', u: 'a\nb', 'key-x': 1, 2: 'n' });
console.log({ a: { b: { c: { d: 1 } } } });
console.log([[1, [2, [3, [4]]]]]);
console.log(new Map([['a', 1], ['b', { x: 1 }]]), new Set([1, 'two']), new Map(), new Set());
console.log([function foo() {}, () => 1, class A {}, class B extends Array {}]);
class P { constructor() { this.x = 1; this.y = [1, 2]; } }
console.log(new P(), [new P()]);
console.log(Array.from({ length: 30 }, function (_, i) { return i * 3; }));
console.log(Array.from({ length: 8 }, function (_, i) { return 'item' + i; }));
console.log({ name: 'a long name for wrapping', description: 'another long string value here', items: [1, 2, 3], more: { k: 'v' } });
var o = { n: 1 }; o.self = o; console.log(o);
console.log(new Date(0), /re/gi, Symbol('s'), [Symbol.iterator]);
console.log(Array.from({ length: 120 }, function (_, i) { return i; }));
console.log(util.inspect({ a: { b: { c: { d: 1 } } } }, { depth: 0 }), util.inspect('str'), util.inspect([1, [2]], false, null));
console.log(util.format('%s %d %i %f %j %o %O %%', 'a', 1.5, 2.9, '3.5', { k: 1 }, [1], { z: 2 }), util.format('x', 1, 'y', { a: 1 }));
console.log('%s is %d years', 'Bob', 42, 'extra');
console.log([undefined, null, NaN, Infinity, -Infinity], { x: [undefined] });
console.log([1.5, 22.25, 3, 400, 5, 6, 7, 8, 9, 10]);
console.log({ f: function () {}, g: function named() {} });
console.log([ 'x'.repeat(30), 'y'.repeat(30), 'z'.repeat(30) ]);
