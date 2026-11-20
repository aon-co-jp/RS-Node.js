# CLAUDE.md (RS-Node.js / RNode.js)

- 姉妹リポジトリ: `RS-JavaScript`(Rust 製 JS エンジン、未実装)・`RS-TypeScript`(型注釈除去)・`RS-HTML`・`RS-CSS`・`RS-BootStrap`・`RS-React`、および `Reaxt`・`RPoRea`・`RPoem`・`RMojo`・`open-cosmo`・`pl2pl`。**常に同時に一緒に開発**する(1つ変えたら他の確認と README/CLAUDE/PORTING/POLICY の更新を同日に)。
- **Node.js 互換**が目標。機能を足すときは、本物の Node.js で同じスクリプトを動かして出力を比べる(`tests/compat/run.sh`)。互換を調べた結果(差・未対応)は README に正直に書く。
- JavaScript の実行は `JsEngine` トレイト経由。いまは Boa、将来 RS-JavaScript。TypeScript は RS-TypeScript で型注釈を除去してから実行。
- 新しい REST は作らない。秘密は書かない。開発・試験はローカル(WSL)、VPS は配備と確認だけ。
