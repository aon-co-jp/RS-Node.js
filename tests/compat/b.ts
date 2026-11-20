interface P { name: string; n: number }
function greet(p: P): string { return "hi " + p.name + " " + p.n; }
const x: number = 41;
console.log(greet({ name: "ts", n: x + 1 }));
