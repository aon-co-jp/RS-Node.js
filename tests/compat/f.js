var b = Buffer.from('héllo 日本', 'utf8');
console.log(b.length, b.toString('hex'), b.toString('base64'), b.toString());
console.log(Buffer.from('aGk=', 'base64').toString(), Buffer.from('6869', 'hex').toString());
var c = Buffer.concat([Buffer.from('ab'), Buffer.alloc(2, 'x'), Buffer.from([99, 100])]);
console.log(c.toString(), Buffer.isBuffer(c), Buffer.isBuffer([]), Buffer.byteLength('日本'));
console.log(JSON.stringify(Buffer.from('hi')), c.slice(1, 4).toString(), c.equals(Buffer.from('abxxcd')), c[0]);
