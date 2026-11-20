function greet(p) { return "hi " + p.name + " " + p.n; }
const x = 41;
console.log(greet({ name: "ts", n: x + 1 }));
