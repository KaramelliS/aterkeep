# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep ロゴ" width="280"/>
</p>

**Aternos サーバーマネージャー & 24/7 ダッシュボード。** 単一の Rust バイナリ（約1.7MB）で無料の Aternos Minecraft サーバーを 24 時間オンラインに保ち、モダンな Web パネルで操作できます — ブラウザ自動化なし、純粋な HTTP。

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.zh.md">中文</a>
</p>

---

<p align="center">
  <a href="promo/video/aterkeep-ja.mp4"><img src="promo/gif/aterkeep-ja.gif" alt="aterkeep - 30秒の紹介" width="100%"/></a>
</p>

<p align="center">
  <sub><b>30秒の紹介動画を見る.</b> プレビューをクリックすると MP4 が開きます。<br/>
  他の言語: <a href="promo/video/aterkeep-en.mp4">English</a> · <a href="promo/video/aterkeep-tr.mp4">Türkçe</a> · <a href="promo/video/aterkeep-de.mp4">Deutsch</a> · <a href="promo/video/aterkeep-fr.mp4">Français</a> · <a href="promo/video/aterkeep-es.mp4">Español</a> · <a href="promo/video/aterkeep-it.mp4">Italiano</a> · <a href="promo/video/aterkeep-pt.mp4">Português</a> · <a href="promo/video/aterkeep-ru.mp4">Русский</a> · <a href="promo/video/aterkeep-ar.mp4">العربية</a> · <a href="promo/video/aterkeep-zh.mp4">中文</a> · <a href="promo/video/aterkeep-ko.mp4">한국어</a> · <a href="promo/video/aterkeep-nl.mp4">Nederlands</a> · <a href="promo/video/aterkeep-pl.mp4">Polski</a></sub>
</p>

---

## 機能

- **順番の自動承認** — 順番が来ると Aternos は約30秒の承認ウィンドウを開き、応答がなければ最後尾に戻されます。無人での24時間稼働を可能にしているのはこの手順です。
- **代わりにログインします** — Aternos アカウントで設定すれば、aterkeep がセッション Cookie を自分で取得し、期限が切れたら更新します。DevTools も毎月のコピーも不要です。
- **アンチアイドルボット** — サーバーが起動すると参加し、空だからという理由で停止されるのを防ぐ Minecraft クライアント
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

## セットアップ（初回のみ）

バイナリを実行し **http://127.0.0.1:4041** を開きます。ウィザードが尋ねるのは
3つ、パネルの言語・パネルのパスワード・Aternos セッションです。

**パネルのパスワード**はパネルを保護し、同時にセッションを暗号化します。鍵は
**ディスクに書き込まれず**、起動のたびにパスワードから導出されます
（PBKDF2-HMAC-SHA256、60万回）。**復旧手段はありません。**

**セッションの与え方は2通り:**

**1. Aternos アカウント（既定）。** ユーザー名とパスワードを入力すると、
aterkeep が純粋な HTTP でログインし Cookie を自分で取得します。サーバーが複数
ある場合はどれを維持するか尋ねます。認証情報は `config/session.enc` の中に、
Cookie と同じ AES-256-GCM 暗号の下に保存され、送信先は `aternos.org` だけです。

> **二段階認証**が有効なアカウント、および Aternos が **captcha** を要求した
> 場合は使えません。どちらも専用のメッセージで通知されます。

**2. Cookie の貼り付け（代替）。** `aternos.org` で F12 → **Network** → F5、
任意のリクエストの `cookie:` 行全体をコピーし、**Console** で
`window.AJAX_TOKEN` を実行します。この方法のセッションは**自動更新されません**。

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

## セッションの寿命

Aternos は Cookie を `Max-Age=2592000` で発行します — **ちょうど30日**。推測
ではなく、ログイン応答から実測した値です。

**アカウントで設定した場合:** 何もする必要はありません。期限が切れると
デーモンが再ログインして続行します。ログに1行出るだけです。

**Cookie を貼り付けた場合:** パネルは `SESSION` バッジと「セッションが切れた」
という表示を出します（以前は停止したサーバーのように見えていました）。ボタンで
ウィザードに戻れます。

パネルには**セッションの経過時間**も表示され、最初の失効後は前回どれだけ
持ったかも分かります。

再起動後の自動起動: **[docs/AUTOSTART.md](docs/AUTOSTART.md)**（DPAPI で保護された Windows タスク、systemd、Termux:Boot）。

## セキュリティ

- セッションは暗号化して保存（`session.enc`、AES-256-GCM）
- **ディスク上に鍵ファイルはありません** — 鍵はパスワードから導出されます（PBKDF2、600,000 回、インストールごとのランダムソルト）。`config/` フォルダーをコピーしてもパスワードなしでは無意味です
- **パネルはログインを要求します** — すべてのエンドポイントが `HttpOnly` セッション Cookie の背後にあります
- API 文字列はバイナリ内で暗号化され、実行時にあなたの鍵で復号されます
- パネルは `127.0.0.1` のみにバインド

## ⚠ Before you buy: Aternos' terms

Aternos' own support documentation says:

> *"Trying to bypass Aternos system by using bots, scripts, or other tricks to
> keep your server on 24/7 is against our rules… The system automatically
> checks for artificial activity."*
> — [24/7 Hosting](https://support.aternos.org/hc/en-us/articles/31771896948253-24-7-Hosting)

That describes this product. **Using aterkeep may get your server or your
Aternos account suspended or deleted.** There is no way for us to prevent that,
and the anti-idle bot makes the activity easier to spot, not harder.

This is sold as-is, for use on accounts you control and are willing to risk.
If that is not acceptable to you, do not buy it — a paid Minecraft host is the
supported way to run a server around the clock.

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
