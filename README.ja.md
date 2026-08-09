# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep ロゴ" width="280"/>
</p>

**Aternos サーバーマネージャー & 24/7 ダッシュボード。** 単一の Rust バイナリ（約1.7MB）で無料の Aternos Minecraft サーバーを 24 時間オンラインに保ち、モダンな Web パネルで操作できます — ブラウザ自動化なし、純粋な HTTP。

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.ar.md">العربية</a> · <a href="README.zh.md">中文</a>
</p>

## 機能

- **Keep-alive ループ** — 90 秒ごとに確認し、オフラインなら自動再起動（切替可）
- **Web パネル** — ライブ状態、起動/停止/再起動、自動起動スイッチ
- **サーバーコンソール** — ブラウザでサーバーログをリアルタイム表示
- **設定エディタ** — `server.properties` をパネルから読み書き
- **プレイヤーリスト** — 誰がオンラインか
- **リクエストインスペクタ** — 各 HTTP リクエストと JSON 応答（学習用）
- **14言語** — ヘッダーで UI 切替
- **暗号化セッション** — Cookie を AES-256-GCM で保存、鍵は PC から出ない

## 要件

- Windows 10/11（内蔵 `curl.exe` を使用）
- Rust ツールチェーン（ビルド時のみ）

## インストール

```powershell
cd rust
cargo build --release
# バイナリ: target/release/aterkeep.exe
```

## セッションのエクスポート（一度だけ）

1. **https://aternos.org** を開いてログイン。
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → `:` の後 → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: `ATERNOS_SESSION` と `ATERNOS_SERVER` をコピー
4. `http/session.json` を作成（形式: [English README](README.md#setup--export-your-session-once)）:

```json
{
  "token": "PASTE_AJAX_TOKEN",
  "sec": "PASTE_GENERATE_AJAX_TOKEN_VALUE",
  "cookies": [
    { "name": "ATERNOS_SESSION", "value": "PASTE_SESSION_VALUE" },
    { "name": "ATERNOS_SERVER", "value": "PASTE_SERVER_ID" }
  ]
}
```

5. インポート:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

`session.enc` + `aterkeep.key` が生成されます — **鍵ファイルを失わないでください**。セッション復号の唯一の手段です。

## 起動

```powershell
.\target\release\aterkeep.exe
```

**http://127.0.0.1:4041** をブラウザで開く。

## パネルのタブ

| タブ | 機能 |
|---|---|
| **ステータス** | 状態バッジ、操作、自動起動、ライブログ、インスペクタ |
| **コンソール** | サーバーログのストリーム（10秒更新） |
| **設定** | `server.properties` を編集して保存 |
| **プレイヤー** | オンラインプレイヤー一覧 |

**自動起動スイッチは重要:** オフにするとサーバーは二度と再起動されません。**停止**ボタンは自動的にオフにします。

## セッションの有効期間

Aternos のセッション Cookie は **約30日** 有効です。パネルに `OTURUM BİTTİ`/`LOGGED OUT` と表示されたら、エクスポート手順を繰り返して再インポートしてください。

## セキュリティ

- セッションは暗号化して保存（`session.enc`、AES-256-GCM）
- `aterkeep.key` はコミットされません
- API 文字列はバイナリ内で暗号化され、実行時にあなたの鍵で復号されます
- パネルは `127.0.0.1` のみにバインド

## ライセンス

**aterkeep は商用ソフトウェアです — オープンソースではありません。**

ソースコードは透明性と評価のためにのみ公開されています。個人的・非商用の利用は
許可されます。再配布・再販売・派生物の作成・商用利用は**禁止**です。全条項は
[LICENSE](LICENSE) を参照してください。

## ライセンスの購入

商用利用、再配布、ホワイトラベル、および keep-alive エンジン（`aterkeep-core`）
のソースアクセスは有償の商用ライセンスで提供されます。

**連絡先:** berlaylc2138@gmail.com

## 免責事項

独立したプロジェクトです — Aternos GmbH および Mojang Studios とは無関係です。
