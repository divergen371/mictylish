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
  - `let name = expr` の最小パース
  - 基本式: `int` / `string` / `ident` / `list`
  - `|>` は T04 で実装予定（現時点は明示エラー）

## プロジェクト構成
- `src/main.rs`: アプリ起動（非同期 REPL）
- `src/repl.rs`: REPL 本体
- `src/token.rs`: トークン定義
- `src/lexer.rs`: 手書き Lexer
- `src/parser.rs`: 手書き Parser（雛形）
- `src/resolver.rs`: シャドウイング禁止の名前解決
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
- T03: `let` 文の構文解析と AST 生成を拡張（span 精度と構文エラーケース強化）
- T04: `|>` の優先順位・結合規則を実装
- T05: Resolver と Parser を統合して検証を強化

## 設計ドキュメント
- `docs/要件定義書.md`
- `docs/詳細設計書.md`
- `docs/実装計画.md`
- `docs/タスク管理.md`
- `docs/ロードマップ.md`
- `docs/Walkthrough.md`
