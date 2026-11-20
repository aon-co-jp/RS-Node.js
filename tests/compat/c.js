// timers / microtasks ordering (compared with real Node)
console.log('start');
setTimeout(function () { console.log('timeout 120'); }, 120);
setTimeout(function (a, b) { console.log('timeout 50', a, b); }, 50, 'x', 'y');
var n = 0;
var h = setInterval(function () { n++; console.log('tick', n); if (n === 3) clearInterval(h); }, 10);
var c = setTimeout(function () { console.log('never'); }, 1);
clearTimeout(c);
process.nextTick(function () { console.log('nextTick'); });
Promise.resolve().then(function () { console.log('promise'); });
queueMicrotask(function () { console.log('microtask'); });
console.log('end');
