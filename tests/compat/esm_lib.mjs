export const PI = 3.14;
export function add(a, b) { return a + b; }
export class Counter { constructor() { this.n = 0; } inc() { this.n++; return this; } }
const secret = (s) => '<' + s + '>';
export { secret as shout };
export default function wrap(s) { return secret(s); }
