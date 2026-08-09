# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Gestor de servidor Aternos & dashboard 24/7.** Un único binario Rust (~1.7 MB) mantiene tu servidor Minecraft Aternos gratuito en línea todo el día y te da un panel web moderno — sin automatización de navegador, HTTP puro.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a>
</p>

## Características

- **Bucle keep-alive** — comprueba cada 90 s y reinicia el servidor automáticamente (desactivable)
- **Dashboard web** — estado en vivo, iniciar/detener/reiniciar, interruptor auto-inicio
- **Consola del servidor** — log del servidor en vivo desde el navegador
- **Editor de ajustes** — leer/cambiar `server.properties` desde el panel
- **Lista de jugadores** — quién está en línea
- **Request inspector** — cada llamada HTTP con su respuesta JSON (educativo)
- **14 idiomas** — interfaz conmutable en la cabecera
- **Sesión cifrada** — cookies en AES-256-GCM, la clave nunca sale de tu equipo

## Requisitos

- Windows 10/11 (usa el `curl.exe` integrado)
- Toolchain de Rust (solo para compilar)

## Instalación

```powershell
cd rust
cargo build --release
# binario: target/release/aterkeep.exe
```

## Exportar sesión (una vez)

1. Abre **https://aternos.org** e inicia sesión.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → parte tras `:` → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: copia `ATERNOS_SESSION` y `ATERNOS_SERVER`
4. Crea `http/session.json` (formato: [English README](README.md#setup--export-your-session-once)):

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

5. Importa:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

Durante la instalación defines una **contraseña del panel**: protege el panel *y* cifra la sesión. La clave **nunca se escribe en disco**; se deriva de la contraseña en cada arranque. Todo queda en una única carpeta `config/`. **Si la olvidas, no hay recuperación.** Para ejecución desatendida: `ATERKEEP_KEY='tu-contraseña' ./aterkeep`

## Ejecutar

```powershell
.\target\release\aterkeep.exe
```

Abre **http://127.0.0.1:4041**.

## Pestañas del panel

| Pestaña | Función |
|---|---|
| **Estado** | insignia de estado, controles, auto-inicio, log en vivo, inspector |
| **Consola** | flujo de log del servidor (refresco 10 s) |
| **Ajustes** | editar `server.properties` y guardar |
| **Jugadores** | lista de jugadores en línea |

**Interruptor auto-inicio importante:** apagado = el servidor no se reinicia nunca. **Detener** lo apaga automáticamente.

## Duración de la sesión

Las cookies de sesión de Aternos duran **~30 días**. Cuando el panel muestre `OTURUM BİTTİ`/`LOGGED OUT`, repite la exportación y vuelve a importar.

## Seguridad

- Sesión cifrada en reposo (`session.enc`, AES-256-GCM)
- **Sin archivo de clave en disco** — la clave se deriva de la contraseña (PBKDF2, 600 000 iteraciones, sal aleatoria por instalación). Copiar la carpeta `config/` no sirve de nada sin la contraseña
- **El panel exige inicio de sesión** — todos los endpoints tras una cookie de sesión `HttpOnly`
- Cadenas API cifradas en el binario, decodificadas en tiempo de ejecución con tu clave
- Panel solo enlazado a `127.0.0.1`

## Licencia

**aterkeep es software comercial — no es de código abierto.**

El código se publica únicamente por transparencia y para su evaluación. Se permite
el uso personal y no comercial. La redistribución, la reventa, las obras derivadas
y el uso comercial **no** están permitidos. Términos completos: [LICENSE](LICENSE).

## Comprar una licencia

El uso comercial, la redistribución, el white-labelling y el acceso al código del
motor keep-alive (`aterkeep-core`) requieren una licencia comercial de pago.

**Contacto:** berlaylc2138@gmail.com

## Aviso legal

Proyecto independiente — sin relación con Aternos GmbH ni Mojang Studios.
