# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep 로고" width="280"/>
</p>

**Aternos 서버 관리자 & 24/7 대시보드.** 단일 Rust 바이너리(약 1.7MB)로 무료 Aternos Minecraft 서버를 하루 24시간 온라인으로 유지하고 현대적인 웹 패널로 제어합니다 — 브라우저 자동화 없음, 순수 HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.ar.md">العربية</a> · <a href="README.zh.md">中文</a> · <a href="README.ja.md">日本語</a>
</p>

## 기능

- **Keep-alive 루프** — 90초마다 확인, 서버가 꺼지면 자동 재시작 (끌 수 있음)
- **웹 패널** — 실시간 상태, 시작/중지/재시작, 자동 시작 스위치
- **서버 콘솔** — 브라우저에서 실시간 서버 로그
- **설정 편집기** — `server.properties` 읽기/수정
- **플레이어 목록** — 누가 온라인인지
- **요청 검사기** — 모든 HTTP 요청과 JSON 응답 (교육용)
- **14개 언어** — 헤더에서 UI 전환
- **암호화 세션** — 쿠키를 AES-256-GCM으로 저장, 키는 PC 밖으로 나가지 않음

## 요구 사항

- Windows 10/11 (내장 `curl.exe` 사용)
- Rust 툴체인 (빌드 시에만)

## 설치

```powershell
cd rust
cargo build --release
# 바이너리: target/release/aterkeep.exe
```

## 세션 내보내기 (한 번)

1. **https://aternos.org** 열고 로그인.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → `:` 뒤 부분 → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: `ATERNOS_SESSION`, `ATERNOS_SERVER` 복사
4. `http/session.json` 생성 (형식: [English README](README.md#setup--export-your-session-once)):

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

5. 가져오기:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

설치 중에 **패널 비밀번호**를 설정합니다. 패널을 보호하는 동시에 세션을 암호화합니다. 키는 **디스크에 저장되지 않으며** 시작할 때마다 비밀번호에서 파생됩니다. 모든 파일은 단일 `config/` 폴더에 모입니다. **잊어버리면 복구할 수 없습니다.** 무인 실행 시: `ATERKEEP_KEY='비밀번호' ./aterkeep`

## 실행

```powershell
.\target\release\aterkeep.exe
```

브라우저에서 **http://127.0.0.1:4041** 열기.

## 패널 탭

| 탭 | 기능 |
|---|---|
| **상태** | 상태 배지, 제어 버튼, 자동 시작, 실시간 로그, 검사기 |
| **콘솔** | 서버 로그 스트림 (10초 갱신) |
| **설정** | `server.properties` 편집 및 저장 |
| **플레이어** | 온라인 플레이어 목록 |

**자동 시작 스위치 중요:** 꺼져 있으면 서버가 다시는 재시작되지 않습니다. **중지** 버튼이 자동으로 끕니다.

## 세션 유효 기간

Aternos 세션 쿠키는 **약 30일** 유효합니다. 패널에 `OTURUM BİTTİ`/`LOGGED OUT`이 표시되면 내보내기 단계를 반복하고 다시 가져오세요.

## 보안

- 세션은 암호화되어 저장됨 (`session.enc`, AES-256-GCM)
- **디스크에 키 파일 없음** — 키는 비밀번호에서 파생됩니다(PBKDF2, 600,000회, 설치별 무작위 솔트). `config/` 폴더를 복사해도 비밀번호 없이는 소용없습니다
- **패널은 로그인을 요구합니다** — 모든 엔드포인트가 `HttpOnly` 세션 쿠키 뒤에 있습니다
- API 문자열은 바이너리 내 암호화, 실행 시 내 키로 복호화
- 패널은 `127.0.0.1`에만 바인딩

## 라이선스

**aterkeep은 상용 소프트웨어입니다 — 오픈 소스가 아닙니다.**

소스 코드는 투명성과 평가 목적으로만 공개됩니다. 개인적·비상업적 사용은
허용됩니다. 재배포, 재판매, 2차적 저작물 및 상업적 사용은 **금지**됩니다.
전체 조건은 [LICENSE](LICENSE)를 참조하세요.

## 라이선스 구매

상업적 사용, 재배포, 화이트라벨링 및 keep-alive 엔진(`aterkeep-core`) 소스
접근은 유료 상용 라이선스로 제공됩니다.

**문의:** berlaylc2138@gmail.com

## 면책 조항

독립 프로젝트입니다 — Aternos GmbH 또는 Mojang Studios와 관련이 없습니다.
