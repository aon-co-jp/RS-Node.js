// process.exit stops pending timers
setTimeout(function () { console.log('never'); }, 30);
console.log('before');
process.exit(3);
console.log('after');
