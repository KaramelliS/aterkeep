# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="логотип aterkeep" width="280"/>
</p>

**Менеджер сервера Aternos и панель 24/7.** Один бинарный файл Rust (~1.7 МБ) держит ваш бесплатный сервер Minecraft Aternos онлайн круглосуточно и даёт современную веб-панель — без автоматизации браузера, чистый HTTP.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a>
</p>

## Возможности

- **Цикл keep-alive** — проверка каждые 90 с, авто-перезапуск сервера (отключается)
- **Веб-панель** — статус в реальном времени, запуск/остановка/перезапуск, переключатель авто-старта
- **Консоль сервера** — живой лог сервера в браузере
- **Редактор настроек** — чтение/изменение `server.properties` из панели
- **Список игроков** — кто онлайн
- **Request inspector** — каждый HTTP-запрос с JSON-ответом (обучающе)
- **14 языков** — интерфейс переключается в шапке
- **Шифрованная сессия** — куки в AES-256-GCM, ключ никогда не покидает ваш ПК

## Требования

- Windows 10/11 (использует встроенный `curl.exe`)
- Rust toolchain (только для сборки)

## Установка

```powershell
cd rust
cargo build --release
# бинарник: target/release/aterkeep.exe
```

## Экспорт сессии (один раз)

1. Откройте **https://aternos.org** и войдите.
2. `F12` → **Console**: `window.AJAX_TOKEN` → `token`; `window.generateAjaxToken()` → часть после `:` → `sec`
3. `F12` → **Application → Cookies → https://aternos.org**: скопируйте `ATERNOS_SESSION` и `ATERNOS_SERVER`
4. Создайте `http/session.json` (формат: [English README](README.md#setup--export-your-session-once)):

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

5. Импортируйте:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

Создаёт `session.enc` + `aterkeep.key` — **не теряйте ключ**, это единственный способ расшифровать сессию.

## Запуск

```powershell
.\target\release\aterkeep.exe
```

Откройте **http://127.0.0.1:4041**.

## Вкладки панели

| Вкладка | Функция |
|---|---|
| **Статус** | бейдж состояния, управление, авто-старт, живой лог, инспектор |
| **Консоль** | поток лога сервера (обновление 10 с) |
| **Настройки** | редактирование `server.properties` и сохранение |
| **Игроки** | список игроков онлайн |

**Переключатель авто-старта важен:** выключен = сервер больше никогда не запустится. **Остановить** выключает его автоматически.

## Срок жизни сессии

Куки сессии Aternos живут **~30 дней**. Когда панель показывает `OTURUM BİTTİ`/`LOGGED OUT`, повторите экспорт и импорт.

## Безопасность

- Сессия зашифрована (`session.enc`, AES-256-GCM)
- `aterkeep.key` никогда не коммитится
- Строки API зашифрованы в бинарнике, расшифровываются в рантайме вашим ключом
- Панель слушает только `127.0.0.1`

## Лицензия

**aterkeep — коммерческое программное обеспечение, а не open source.**

Исходный код опубликован исключительно для прозрачности и ознакомления. Разрешено
личное некоммерческое использование. Распространение, перепродажа, производные
работы и коммерческое использование **запрещены**. Полные условия: [LICENSE](LICENSE).

## Покупка лицензии

Коммерческое использование, распространение, white-labelling и доступ к исходному
коду движка keep-alive (`aterkeep-core`) доступны по платной лицензии.

**Контакт:** berlaylc2138@gmail.com

## Отказ от ответственности

Независимый проект — не связан с Aternos GmbH или Mojang Studios.
