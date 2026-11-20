const path = require('path').posix; const fs = require('fs'); const lib = require('./lib');
console.log('sum', lib.add(2, 3), lib.name, path.join('/a', '..', 'b', 'c.txt'), path.extname('x/y.tar.gz'));
console.log(path.basename('/x/y.js', '.js'), path.dirname('/x/y/z'), path.normalize('a//b/../c/'), path.resolve('/a', 'b', '../c'));
fs.writeFileSync(__dirname + '/out.txt', 'hello');
console.log(fs.readFileSync(__dirname + '/out.txt', 'utf8'), fs.existsSync('/nonexistent'), process.argv.slice(2).join(','));
console.log([1, 2, 3].map(function (x) { return x * 2; }).join('-'), JSON.stringify({ a: 1, b: [true, null] }));
Promise.resolve(7).then(function (v) { console.log('promise', v); });
console.log('end');
