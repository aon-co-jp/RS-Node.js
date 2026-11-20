#!/bin/bash
# Node.js との互換の通し試験: 同じスクリプトを rnodejs と本物の node で実行し、出力が一致するか比べる(node が無ければ rnodejs の結果だけ表示)
# 使い方: cargo build --release --features boa,ts && bash tests/compat/run.sh
cd "$(dirname "$0")/../.."
B=${RNODEJS:-target/release/rnodejs}
NODE=${NODE:-node}
for f in a.js b.ts; do
  ref=$f; [ $f = b.ts ] && ref=b_node.js
  "$B" tests/compat/$f one two > /tmp/rn_out.txt 2>&1
  if command -v "$NODE" >/dev/null; then
    "$NODE" tests/compat/$ref one two 2>&1 | tr -d '\r' > /tmp/node_out.txt
    diff /tmp/rn_out.txt /tmp/node_out.txt >/dev/null && echo "$f: same as node" || { echo "$f: DIFFERENT"; diff /tmp/rn_out.txt /tmp/node_out.txt; }
  else cat /tmp/rn_out.txt; fi
done
