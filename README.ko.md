# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="aterkeep 로고" width="280"/>
</p>

**Aternos 서버 관리자 & 24/7 대시보드.** 단일 Rust 바이너리(약 1.7MB)로 무료 Aternos Minecraft 서버를 하루 24시간 온라인으로 유지하고 현대적인 웹 패널로 제어합니다 — 브라우저 자동화 없음, 순수 HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a> · <a href="README.zh.md">中文</a> · <a href="README.ja.md">日本語</a>
</p>

## 기능

- **대기열 자동 확인** — 차례가 오면 Aternos가 약 30초짜리 확인 창을 여는데, 아무도 응답하지 않으면 맨 뒤로 밀립니다. 무인 24시간 운영을 가능하게 하는 단계가 바로 이것입니다.
- **대신 로그인합니다** — Aternos 계정으로 설정하면 aterkeep이 세션 쿠키를 직접 받아오고 만료되면 스스로 갱신합니다. DevTools도, 매달 복사·붙여넣기도 없습니다.
- **안티 아이들 봇** — 서버가 켜지면 접속해서 비어 있다는 이유로 종료되지 않게 하는 마인크래프트 클라이언트
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

## 설치 (한 번만)

바이너리를 실행하고 **http://127.0.0.1:4041** 을 엽니다. 마법사는 세 가지를
묻습니다: 패널 언어, 패널 비밀번호, Aternos 세션.

**패널 비밀번호**는 패널을 보호하는 동시에 세션을 암호화합니다. 키는 디스크에
**저장되지 않고** 실행할 때마다 비밀번호에서 유도됩니다(PBKDF2-HMAC-SHA256,
600,000회). **복구 방법은 없습니다.**

**세션을 넘기는 방법은 두 가지입니다:**

**1. Aternos 계정 (기본값).** 아이디와 비밀번호를 입력하면 aterkeep이 순수
HTTP로 로그인해 쿠키를 직접 가져옵니다. 서버가 여러 개면 어느 것을 유지할지
묻습니다. 자격 증명은 `config/session.enc` 안에 쿠키와 동일한 AES-256-GCM
암호화로 저장되며 `aternos.org` 외에는 어디에도 전송되지 않습니다.

> **2단계 인증**이 켜진 계정과 Aternos가 **캡차**를 요구하는 경우에는 쓸 수
> 없습니다. 두 경우 모두 별도의 메시지로 안내됩니다.

**2. 쿠키 붙여넣기 (대체).** `aternos.org`에서 F12 → **Network** → F5, 아무
요청의 `cookie:` 줄 전체를 복사하고 **Console**에서 `window.AJAX_TOKEN`을
실행합니다. 이렇게 만든 세션은 **스스로 갱신되지 않습니다**.

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

## 세션 수명

Aternos는 쿠키를 `Max-Age=2592000`으로 내려줍니다 — **정확히 30일**. 추측이
아니라 로그인 응답에서 측정한 값입니다.

**계정으로 설정한 경우:** 할 일이 없습니다. 만료되면 데몬이 다시 로그인하고
계속 동작합니다. 로그에 한 줄이 남습니다.

**쿠키를 붙여넣은 경우:** 패널이 `SESSION` 배지와 세션 만료 안내를 표시합니다.
예전처럼 서버가 꺼진 것으로 보이지 않으며, 버튼으로 마법사로 돌아갑니다.

패널은 **세션 경과 시간**도 보여주고, 첫 만료 이후에는 이전 세션이 얼마나
버텼는지도 알려줍니다.

재부팅 후 자동 시작: **[docs/AUTOSTART.md](docs/AUTOSTART.md)** (DPAPI로 보호되는 Windows 작업, systemd, Termux:Boot).

## 보안

- 세션은 암호화되어 저장됨 (`session.enc`, AES-256-GCM)
- **디스크에 키 파일 없음** — 키는 비밀번호에서 파생됩니다(PBKDF2, 600,000회, 설치별 무작위 솔트). `config/` 폴더를 복사해도 비밀번호 없이는 소용없습니다
- **패널은 로그인을 요구합니다** — 모든 엔드포인트가 `HttpOnly` 세션 쿠키 뒤에 있습니다
- API 문자열은 바이너리 내 암호화, 실행 시 내 키로 복호화
- 패널은 `127.0.0.1`에만 바인딩

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
