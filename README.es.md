# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Gestor de servidor Aternos & dashboard 24/7.** Un único binario Rust (~1.7 MB) mantiene tu servidor Minecraft Aternos gratuito en línea todo el día y te da un panel web moderno — sin automatización de navegador, HTTP puro.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a>
</p>

## Características

- **Confirmación automática de la cola** — cuando llega tu turno, Aternos abre una ventana de unos 30 segundos; si nadie responde, vuelves al final. Este paso es lo que hace posible el 24/7 desatendido.
- **Inicia sesión por ti** — con tu cuenta de Aternos, aterkeep obtiene la cookie de sesión por su cuenta y la renueva cuando caduca. Sin DevTools, sin copiar y pegar cada mes.
- **Bot anti-idle** — un cliente de Minecraft que entra cuando el servidor está encendido para que no lo apaguen por estar vacío
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

## Instalación (una vez)

Ejecuta el binario y abre **http://127.0.0.1:4041**. El asistente pide tres
cosas: idioma del panel, contraseña del panel y sesión de Aternos.

La **contraseña del panel** protege el panel *y* cifra la sesión. La clave
**nunca se guarda en disco**: se deriva de la contraseña en cada arranque
(PBKDF2-HMAC-SHA256, 600 000 iteraciones). **No hay recuperación.**

**Dos formas de dar la sesión:**

**1. Cuenta de Aternos (predeterminado).** Escribe tu usuario y contraseña:
aterkeep inicia sesión por HTTP puro y obtiene la cookie. Si tu cuenta tiene
varios servidores, el asistente pregunta cuál mantener. Las credenciales se
guardan en `config/session.enc`, bajo el mismo cifrado AES-256-GCM que las
cookies, y solo se envían a `aternos.org`.

> No funciona con **verificación en dos pasos** ni si Aternos pide un
> **captcha**; ambos se informan con su propio mensaje.

**2. Pegar cookies (alternativa).** En `aternos.org`: F12 → **Network** → F5,
copia la línea `cookie:` completa de cualquier petición y ejecuta
`window.AJAX_TOKEN` en la **Console**. Una sesión creada así **no se renueva
sola**.

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

Aternos entrega la cookie con `Max-Age=2592000` — **exactamente 30 días**,
medido en la respuesta del inicio de sesión, no supuesto.

**Con cuenta:** nada que hacer. Al caducar, el demonio vuelve a iniciar sesión y
continúa — una línea en el registro.

**Con cookies pegadas:** el panel muestra una insignia `SESSION` y un aviso de
sesión caducada — no un servidor apagado — con un botón que lleva al asistente.

El panel también muestra la **antigüedad de la sesión** y, tras la primera
caducidad, cuánto duró la anterior.

Arranque automático tras reiniciar: **[docs/AUTOSTART.md](docs/AUTOSTART.md)** (tarea programada de Windows con DPAPI, systemd, Termux:Boot).

## Seguridad

- Sesión cifrada en reposo (`session.enc`, AES-256-GCM)
- **Sin archivo de clave en disco** — la clave se deriva de la contraseña (PBKDF2, 600 000 iteraciones, sal aleatoria por instalación). Copiar la carpeta `config/` no sirve de nada sin la contraseña
- **El panel exige inicio de sesión** — todos los endpoints tras una cookie de sesión `HttpOnly`
- Cadenas API cifradas en el binario, decodificadas en tiempo de ejecución con tu clave
- Panel solo enlazado a `127.0.0.1`

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
