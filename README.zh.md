# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep 标志" width="280"/>
</p>

**Aternos 服务器管理与 24/7 看护面板。** 单个 Rust 二进制（约 1.7 MB）让您的免费 Aternos Minecraft 服务器全天在线，并提供现代 Web 面板——无需浏览器自动化，纯 HTTP。

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.ar.md">العربية</a>
</p>

## 功能

- **Keep-alive 循环** — 每 90 秒检查，服务器离线时自动重启（可关闭）
- **Web 面板** — 实时状态、启动/停止/重启、自动启动开关
- **服务器控制台** — 浏览器中查看实时服务器日志
- **设置编辑器** — 从面板读取/修改 `server.properties`
- **玩家列表** — 谁在线
- **请求检查器** — 每个 HTTP 请求及其 JSON 响应（教学）
- **14 种语言** — 界面可在顶部切换
- **加密会话** — Cookie 使用 AES-256-GCM，密钥绝不离开您的电脑

## 环境要求

- Windows 10/11（使用内置 `curl.exe`）
- Rust 工具链（仅用于编译）

## 安装

```powershell
cd rust
cargo build --release
# 二进制：target/release/aterkeep.exe
```

## 导出会话（一次）

1. 打开 **https://aternos.org** 并登录。
2. `F12` → **Console**：`window.AJAX_TOKEN` → `token`；`window.generateAjaxToken()` → `:` 之后的部分 → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**：复制 `ATERNOS_SESSION` 和 `ATERNOS_SERVER`
4. 创建 `http/session.json`（格式见 [English README](README.md#setup--export-your-session-once)）：

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

5. 导入：

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

安装时你需要设置**面板密码**：它既保护面板，也用于加密会话。密钥**绝不写入磁盘**，每次启动时都从密码派生。所有文件集中在单个 `config/` 目录中。**一旦忘记密码，将无法恢复。** 无人值守运行：`ATERKEEP_KEY='你的密码' ./aterkeep`

## 运行

```powershell
.\target\release\aterkeep.exe
```

打开 **http://127.0.0.1:4041**。

## 面板标签页

| 标签页 | 功能 |
|---|---|
| **状态** | 状态徽章、控制按钮、自动启动、实时日志、请求检查器 |
| **控制台** | 服务器日志流（10 秒刷新） |
| **设置** | 编辑并保存 `server.properties` |
| **玩家** | 在线玩家列表 |

**自动启动开关很重要：** 关闭后服务器永远不会被重启。**停止**按钮会自动关闭它。

## 会话有效期

Aternos 会话 Cookie 有效期约 **30 天**。面板显示 `OTURUM BİTTİ`/`LOGGED OUT` 时，重复导出并重新导入即可。

## 安全性

- 会话静态加密（`session.enc`，AES-256-GCM）
- **磁盘上没有密钥文件** — 密钥从密码派生（PBKDF2，600 000 次迭代，每次安装使用随机盐）。即使复制了 `config/` 目录，没有密码也无法解密
- **面板需要登录** — 所有接口均受 `HttpOnly` 会话 Cookie 保护
- API 字符串在二进制中加密，运行时用您的密钥解密
- 面板仅绑定 `127.0.0.1`

## 许可证

**aterkeep 是商业软件 — 并非开源软件。**

公开源代码仅用于透明性与评估。允许个人非商业使用。**禁止**再分发、转售、衍生作品
及商业使用。完整条款见 [LICENSE](LICENSE)。

## 购买许可

商业使用、再分发、白标以及 keep-alive 引擎（`aterkeep-core`）的源码访问需购买
商业许可证。

**联系方式：** berlaylc2138@gmail.com

## 免责声明

独立项目 — 与 Aternos GmbH 或 Mojang Studios 无任何关联。
