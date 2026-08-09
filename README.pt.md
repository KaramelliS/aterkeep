# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Gerenciador de servidor Aternos & painel 24/7.** Um único binário Rust (~1.7 MB) mantém seu servidor Minecraft Aternos gratuito online o dia inteiro e dá a você um painel web moderno — sem automação de navegador, HTTP puro.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a>
</p>

## Recursos

- **Loop keep-alive** — verifica a cada 90 s e reinicia o servidor automaticamente (desativável)
- **Painel web** — status ao vivo, iniciar/parar/reiniciar, interruptor auto-start
- **Console do servidor** — log ao vivo no navegador
- **Editor de configurações** — ler/alterar `server.properties` pelo painel
- **Lista de jogadores** — quem está online
- **Request inspector** — cada chamada HTTP com resposta JSON (educativo)
- **14 idiomas** — interface trocável no cabeçalho
- **Sessão criptografada** — cookies em AES-256-GCM, a chave nunca sai da sua máquina

## Requisitos

- Windows 10/11 (usa o `curl.exe` integrado)
- Toolchain Rust (só para compilar)

## Instalação

```powershell
cd rust
cargo build --release
# binário: target/release/aterkeep.exe
```

## Exportar sessão (uma vez)

1. Abra **https://aternos.org** e faça login.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → parte após `:` → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: copie `ATERNOS_SESSION` e `ATERNOS_SERVER`
4. Crie `http/session.json` (formato: [English README](README.md#setup--export-your-session-once)):

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

5. Importe:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

Na instalação você define uma **senha do painel**: ela protege o painel *e* criptografa a sessão. A chave **nunca é gravada em disco**; é derivada da senha a cada início. Tudo fica em uma única pasta `config/`. **Se esquecer, não há recuperação.** Para execução autônoma: `ATERKEEP_KEY='sua-senha' ./aterkeep`

## Executar

```powershell
.\target\release\aterkeep.exe
```

Abra **http://127.0.0.1:4041**.

## Abas do painel

| Aba | Função |
|---|---|
| **Status** | selo de status, controles, auto-start, log ao vivo, inspector |
| **Console** | fluxo de log do servidor (atualiza 10 s) |
| **Configurações** | editar `server.properties` e salvar |
| **Jogadores** | lista de jogadores online |

**Interruptor auto-start importante:** desligado = o servidor nunca reinicia. **Parar** o desliga automaticamente.

## Duração da sessão

Os cookies de sessão Aternos duram **~30 dias**. Quando o painel mostrar `OTURUM BİTTİ`/`LOGGED OUT`, repita a exportação e reimporte.

## Segurança

- Sessão criptografada em repouso (`session.enc`, AES-256-GCM)
- **Nenhum arquivo de chave em disco** — a chave é derivada da senha (PBKDF2, 600 000 iterações, sal aleatório por instalação). Copiar a pasta `config/` não adianta sem a senha
- **O painel exige login** — todos os endpoints atrás de um cookie de sessão `HttpOnly`
- Strings da API criptografadas no binário, decodificadas em runtime com sua chave
- Painel apenas em `127.0.0.1`

## Licença

**aterkeep é software comercial — não é código aberto.**

O código é publicado apenas para transparência e avaliação. É permitido o uso
pessoal e não comercial. Redistribuição, revenda, obras derivadas e uso comercial
**não** são permitidos. Termos completos: [LICENSE](LICENSE).

## Comprar uma licença

Uso comercial, redistribuição, white-labelling e acesso ao código do motor
keep-alive (`aterkeep-core`) exigem uma licença comercial paga.

**Contato:** berlaylc2138@gmail.com

## Aviso

Projeto independente — sem afiliação com a Aternos GmbH ou a Mojang Studios.
