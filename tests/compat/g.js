var fs = require('fs'), os = require('os'), path = require('path');
var f = path.join(os.tmpdir(), 'rn_compat_g.txt');
fs.writeFile(f, 'hello', function (e) {
  console.log('written', e);
  fs.readFile(f, 'utf8', function (e2, d) {
    console.log('read', e2, d);
    var raw = fs.readFileSync(f);
    console.log(Buffer.isBuffer(raw), raw.length, raw.toString(), typeof fs.readFileSync(f, 'utf8'));
    fs.readFile('/no/such/file', 'utf8', function (e3) { console.log('missing', e3 ? 'error' : 'ok'); });
    require('fs/promises').readFile(f, 'utf8').then(function (v) { console.log('promise', v); });
  });
});
console.log('sync-first', os.EOL.length > 0);
