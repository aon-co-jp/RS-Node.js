import fs, { existsSync } from 'node:fs';
import * as path from 'path';
import def, { add, PI as pi, Counter } from './esm_lib.mjs';
import cjs from './lib.js';
import './esm_side.mjs';
export const answer = add(40, 2);
export function hello(n) { return 'hello ' + n; }
console.log(def('x'), add(1, 2), pi, new Counter().inc().n, answer, hello('esm'));
console.log(typeof fs.readFileSync, existsSync('/nonexistent'), typeof path.join, cjs.add(2, 3));
export default answer;
