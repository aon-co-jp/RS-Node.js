var EventEmitter = require('events');
var util = require('node:util');
var e = new EventEmitter();
e.on('x', function (a, b) { console.log('x1', a, b); });
e.once('x', function (a) { console.log('once', a); });
console.log(e.emit('x', 1, 2), e.emit('x', 3), e.emit('none'), e.listenerCount('x'));
function F() { EventEmitter.call(this); }
util.inherits(F, EventEmitter);
var f = new F(); f.on('y', function () { console.log('y', this === f); }); f.emit('y');
console.log(util.format('%s=%d %j %%', 'a', 42, { k: [1] }, 'extra'));
util.promisify(function (v, cb) { cb(null, v * 2); })(21).then(function (r) { console.log('p', r); });
try { e.emit('error', new Error('boom')); } catch (x) { console.log('caught', x.message); }
