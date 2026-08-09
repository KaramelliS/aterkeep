# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="شعار aterkeep" width="280"/>
</p>

**مدير خادم Aternos ولوحة 24/7.** ملف ثنائي واحد بلغة Rust (~1.7 م.ب) يُبقي خادم Minecraft المجاني على Aternos متصلًا على مدار الساعة ويمنحك لوحة ويب حديثة — بدون أتمتة متصفح، HTTP خالص.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português</a> · <a href="README.ru.md">Русский</a>
</p>

## المزايا

- **حلقة keep-alive** — فحص كل 90 ثانية وإعادة تشغيل تلقائية للخادم (قابلة للإيقاف)
- **لوحة ويب** — حالة مباشرة، تشغيل/إيقاف/إعادة تشغيل، مفتاح تشغيل تلقائي
- **كونسول الخادم** — سجل الخادم المباشر من المتصفح
- **محرر الإعدادات** — قراءة/تعديل `server.properties` من اللوحة
- **قائمة اللاعبين** — من المتصل الآن
- **مُفحص الطلبات** — كل طلب HTTP مع استجابة JSON (تعليمي)
- **14 لغة** — واجهة قابلة للتبديل من الشريط العلوي
- **جلسة مشفرة** — الكوكيز بتشفير AES-256-GCM، المفتاح لا يغادر جهازك

## المتطلبات

- Windows 10/11 (يستخدم `curl.exe` المدمج)
- أدوات Rust (للبناء فقط)

## التثبيت

```powershell
cd rust
cargo build --release
# الملف: target/release/aterkeep.exe
```

## تصدير الجلسة (مرة واحدة)

1. افتح **https://aternos.org** وسجّل الدخول.
2. `F12` ← **Console**: `window.AJAX_TOKEN` ← `token`؛ `window.generateAjaxToken()` ← الجزء بعد `:` ← `sec`
3. `F12` ← **Application → Cookies → https://aternos.org**: انسخ `ATERNOS_SESSION` و `ATERNOS_SERVER`
4. أنشئ `http/session.json` (التنسيق: [English README](README.md#setup--export-your-session-once)):

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

5. استورد:

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

أثناء التثبيت تحدّد **كلمة مرور اللوحة**: تحمي اللوحة *و* تُشفّر الجلسة. المفتاح **لا يُكتب على القرص أبدًا**، بل يُشتق من كلمة المرور عند كل تشغيل. كل الملفات في مجلد `config/` واحد. **لا توجد وسيلة للاسترجاع إذا نسيتها.** للتشغيل غير المراقب: `ATERKEEP_KEY='كلمة-المرور' ./aterkeep`

## التشغيل

```powershell
.\target\release\aterkeep.exe
```

افتح **http://127.0.0.1:4041**.

## تبويبات اللوحة

| التبويب | الوظيفة |
|---|---|
| **الحالة** | شارة الحالة، الأزرار، التشغيل التلقائي، السجل المباشر، المفحص |
| **الكونسول** | تدفق سجل الخادم (تحديث كل 10 ث) |
| **الإعدادات** | تعديل `server.properties` وحفظه |
| **اللاعبون** | قائمة اللاعبين المتصلين |

**مفتاح التشغيل التلقائي مهم:** عند إيقافه لن يُعاد تشغيل الخادم أبدًا. زر **إيقاف** يطفئه تلقائيًا.

## عمر الجلسة

كوكيز جلسة Aternos تدوم **~30 يومًا**. عندما تعرض اللوحة `OTURUM BİTTİ`/`LOGGED OUT`، كرر خطوات التصدير والاستيراد.

## الأمان

- الجلسة مشفرة (`session.enc`، AES-256-GCM)
- **لا يوجد ملف مفتاح على القرص** — يُشتق المفتاح من كلمة المرور (PBKDF2، 600000 تكرار، ملح عشوائي لكل تثبيت). نسخ مجلد `config/` لا يفيد بدون كلمة المرور
- **اللوحة تتطلب تسجيل الدخول** — جميع نقاط الوصول خلف ملف تعريف ارتباط جلسة `HttpOnly`
- نصوص API مشفرة داخل الملف الثنائي وتُفك في وقت التشغيل بمفتاحك
- اللوحة مرتبطة بـ `127.0.0.1` فقط

## الترخيص

**aterkeep برنامج تجاري — وليس مفتوح المصدر.**

يُنشر الكود المصدري لأغراض الشفافية والتقييم فقط. يُسمح بالاستخدام الشخصي غير
التجاري. أما إعادة التوزيع وإعادة البيع والأعمال المشتقة والاستخدام التجاري
**فغير مسموح بها**. الشروط الكاملة: [LICENSE](LICENSE).

## شراء ترخيص

الاستخدام التجاري وإعادة التوزيع والعلامة البيضاء والوصول إلى مصدر محرك
keep-alive (`aterkeep-core`) متاحة بموجب ترخيص تجاري مدفوع.

**للتواصل:** berlaylc2138@gmail.com

## إخلاء مسؤولية

مشروع مستقل — غير مرتبط بشركة Aternos GmbH أو Mojang Studios.
