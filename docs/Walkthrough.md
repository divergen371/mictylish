# Walkthrough
## 2026-02-21
- `cargo new mictylish` 後の初期雛形を作成
- `tokio` / `miette` / `rustyline` を導入
- 名前衝突・glob非自動・引数分割なしの MVP テストを追加
- `docs/` 配下に要件定義書・詳細設計書・実装計画・タスク管理・ロードマップを作成
- T02: 手書きパーサ雛形を追加（`token` / `lexer` / `parser` モジュール）
- `let` 単文パースと基本式（int/string/ident/list）の最小実装を追加
- `|>` は T04 予定として明示エラーにし、仕様段階を固定
- `tests/parser_scaffold.rs` を追加し、lexer/parser 雛形のテストを実装
- `README.md` を新規作成し、先頭に `logo/Mictylish_logo.png` を配置
- `logo/Mictylish_logo.png` の背景を透過化（バックアップ: `logo/Mictylish_logo.backup.png`）
- ロゴ境界のギザつきを軽減するため、バックアップから再生成してエッジを平滑化
- 境界品質をさらに改善するため、8xスーパーサンプリング + アルファ微調整で再処理
- 8x調整が強すぎたため、バックアップから「4x + 軽量平滑化」設定に戻して再生成
## 次アクション
- T03: `let` 文の構文解析と AST 生成を拡張（span 精度と構文エラーケースを強化）
