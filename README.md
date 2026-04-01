![Mictylish Logo](logo/Mictylish_logo.png)

# mictylish
安全性を最優先にした、データフロー指向モダンシェルの実験実装です。
従来シェルの事故要因（暗黙展開、単語分割、曖昧な失敗処理）を、言語仕様と実行モデルで構造的に減らすことを目的にしています。

## 目標
- パイプ中心の操作性を維持する
- 値（Value/Row）中心のストリーム処理を行う
- 失敗を `Result` として扱い、診断可能にする
- 副作用を `io do ... end` 境界に閉じ込める
- 再束縛・シャドウイングを禁止する

## 現在の実装状況（MVP途中）
- `tokio` + `rustyline` で最小 REPL を実装
- `miette` による診断エラー基盤を実装
- 安全な外部コマンド実行モデル（program + args）を実装
- `glob(...)` を明示 API として実装
- 手書きパーサ雛形（`token` / `lexer` / `parser`）を追加
  - `let` / `let mut` / `set` のパースと評価（不変変数への `set` は静的拒否）
  - 基本式: `int` / `string` / `ident` / `list` / `fn x -> expr end` / `match expr do pat when guard -> expr ... end` / `with pat <- expr, ... do body else fallback end`
  - 比較演算子: `==` / `!=`（`Bool` を返す）
  - `|>` は左結合でパースし、`Expr::Pipe` として AST 化
  - `io do expr end` による副作用境界制御
  - `run_text(prog, args...)` — `io` 内でのみ外部コマンド実行（失敗は構造化エラー）
  - 言語内 Result: `ok(v)` / `err(v)` / `is_ok()` / `is_err()` 組み込み。`Ok(pat)` / `Err(pat)` でパターンマッチ
  - 評価器（`eval`）: リテラル・`let` 束縛・リスト・`fn`・`match`・`with`・`io`・`|>`・Result
- REPL でパース → `Resolver` → `eval` の順（成功時は `name = value` を表示）

## プロジェクト構成
- `src/main.rs`: アプリ起動（非同期 REPL）
- `src/repl.rs`: REPL 本体
- `src/token.rs`: トークン定義
- `src/lexer.rs`: 手書き Lexer
- `src/parser.rs`: 手書き Parser（雛形）
- `src/resolver.rs`: シャドウイング禁止の名前解決
- `src/eval.rs`: 式・`let` の評価（最小）
- `src/command.rs`: 外部コマンド仕様
- `src/runtime.rs`: 実行ブリッジ
- `src/builtin.rs`: 組み込み関数（例: glob）
- `docs/`: 要件・設計・計画・ロードマップ・進捗記録

## 開発用コマンド
```bash
cargo test
```

```bash
cargo run
```

## 直近タスク
- Stream ベースの `map` / `where` / `each`
- Record リテラルとアクセス
- JSON 変換ブリッジ

## 設計ドキュメント
- `docs/要件定義書.md`
- `docs/詳細設計書.md`
- `docs/実装計画.md`
- `docs/タスク管理.md`
- `docs/ロードマップ.md`
- `docs/Walkthrough.md`
