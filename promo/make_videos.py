#!/usr/bin/env python3
"""aterkeep tanitim videolarini uretir — dil basina bir MP4.

Kareler Pillow ile CIZILIR (tarayici yok, ekran kaydi yok), sonra ffmpeg ile
H.264'e kodlanir. Boylece sonuc deterministik: ayni girdi her seferinde ayni
videoyu verir ve bir metni degistirmek icin kimsenin ekran kaydi almasi
gerekmez.

Renkler ve olculer docs/DESIGN.md tokenlarindan gelir; video urunun kendisiyle
ayni dili konusur.

KULLANIM
    python promo/make_videos.py                # 14 dilin hepsi
    python promo/make_videos.py tr en          # yalnizca secilenler
    python promo/make_videos.py --fps 30 tr

GEREKSINIM
    Pillow, ffmpeg (PATH'te ya da ATERKEEP_FFMPEG ile gosterilmis)
    Arapca icin: pip install arabic-reshaper python-bidi
"""
import os
import shutil
import subprocess
import sys
import tempfile

from PIL import Image, ImageChops, ImageDraw, ImageFont

# ------------------------------------------------------------------ tokenlar
W, H = 1920, 1080
BG        = (11, 13, 16)
PANEL     = (18, 21, 25)
PANEL_2   = (23, 27, 33)
LINE      = (43, 50, 61)
LINE_STR  = (58, 67, 79)
FG        = (223, 228, 236)
FG_DIM    = (152, 162, 179)
FG_MUTE   = (134, 142, 153)
OK        = (39, 196, 107)
WARN      = (226, 163, 54)
ERR       = (229, 72, 77)

FONTS = r"C:\Windows\Fonts"
# Latin/Kiril icin Segoe UI; CJK ve Arapca kendi ailelerini ister.
FACE = {
    "sans":      os.path.join(FONTS, "segoeui.ttf"),
    "sans_bold": os.path.join(FONTS, "segoeuib.ttf"),
    "sans_semi": os.path.join(FONTS, "seguisb.ttf"),
    "mono":      os.path.join(FONTS, "consola.ttf"),
    "mono_bold": os.path.join(FONTS, "consolab.ttf"),
}
FACE_LANG = {
    "zh": {"sans": "msyh.ttc", "sans_bold": "msyhbd.ttc", "sans_semi": "msyh.ttc"},
    "ja": {"sans": "YuGothM.ttc", "sans_bold": "YuGothB.ttc", "sans_semi": "YuGothM.ttc"},
    "ko": {"sans": "malgun.ttf", "sans_bold": "malgunbd.ttf", "sans_semi": "malgun.ttf"},
    "ar": {"sans": "segoeui.ttf", "sans_bold": "segoeuib.ttf", "sans_semi": "seguisb.ttf"},
}

_font_cache = {}


def font(kind, size, lang="en"):
    path = FACE[kind]
    if lang in FACE_LANG and kind in FACE_LANG[lang]:
        cand = os.path.join(FONTS, FACE_LANG[lang][kind])
        if os.path.exists(cand):
            path = cand
    key = (path, size)
    if key not in _font_cache:
        _font_cache[key] = ImageFont.truetype(path, size)
    return _font_cache[key]


def shape(text, lang):
    """Arapca'yi baglar ve yonunu duzeltir.

    Pillow raqm olmadan derlenmisse Arapca harfleri birlestirmez ve soldan saga
    dizer — okunaksiz bir sonuc. Metni onceden sekillendirip ters cevirmek,
    ciziciden bagimsiz olarak dogru goruntuyu verir.
    """
    if lang != "ar":
        return text
    try:
        import arabic_reshaper
        from bidi.algorithm import get_display
        return get_display(arabic_reshaper.reshape(text))
    except Exception:
        return text


# ------------------------------------------------------------------ yardimcilar
def ease(t):
    """cubic-bezier(0.5,0,0.1,1) yaklasimi — DESIGN.md'nin standart egrisi."""
    t = max(0.0, min(1.0, t))
    return 1 - pow(1 - t, 3)


def lerp(a, b, t):
    return a + (b - a) * t


def mix(c1, c2, t):
    return tuple(int(lerp(c1[i], c2[i], t)) for i in range(3))


def rrect(d, box, r, fill=None, outline=None, width=1):
    d.rounded_rectangle(box, radius=r, fill=fill, outline=outline, width=width)


# Consolas'in KAPSAMADIGI yazi sistemleri. Bu araliklardan bir karakter
# gorunce mono fonttan dilin sans fontuna duseriz; yoksa ekranda tofu kutusu
# (□□□) cikiyor. Arapca sekillendirici ciktiyi "sunum bicimleri" araliginda
# uretir (FB50+/FE70+), o yuzden onlar da listede.
_NON_LATIN = (
    (0x0590, 0x08FF),   # Ibranice, Arapca, Suryanice, Tana, NKo
    (0x0900, 0x0DFF),   # Hint yazilari
    (0x0E00, 0x0FFF),   # Tay, Lao, Tibet
    (0x1100, 0x11FF),   # Hangul Jamo
    (0x2E80, 0xA4CF),   # CJK
    (0xAC00, 0xD7AF),   # Hangul heceleri
    (0xF900, 0xFAFF),   # CJK uyumluluk
    (0xFB50, 0xFDFF),   # Arapca sunum bicimleri A
    (0xFE70, 0xFEFF),   # Arapca sunum bicimleri B
)


def _needs_fallback(s):
    return any(any(lo <= ord(c) <= hi for lo, hi in _NON_LATIN) for c in s)


def text(d, xy, s, f, fill, anchor="la", lang="en"):
    """Metni cizer; font karakterleri kapsamiyorsa dilin sans fontuna duser."""
    s = shape(s, lang)
    if _needs_fallback(s):
        f = font("sans", f.size, lang)
    d.text(xy, s, font=f, fill=fill, anchor=anchor)


def logo(d, x, y, size):
    """Marka isareti: yesil kare + konsol istemi."""
    rrect(d, (x, y, x + size, y + size), int(size * 0.22), fill=OK)
    s, k = size, size / 64.0
    d.line([(x + 20 * k, y + 22 * k), (x + 30 * k, y + 32 * k),
            (x + 20 * k, y + 42 * k)], fill=(6, 20, 12),
           width=max(2, int(5 * k)), joint="curve")
    d.line([(x + 34 * k, y + 43 * k), (x + 46 * k, y + 43 * k)],
           fill=(6, 20, 12), width=max(2, int(5 * k)))
    return s


def fade_layer(img, alpha):
    """Sahne girisleri: siyaha karsi opaklik."""
    if alpha >= 1.0:
        return img
    black = Image.new("RGB", img.size, BG)
    return Image.blend(black, img, max(0.0, alpha))


# ------------------------------------------------------------------ sahneler
def scene_title(d, L, p, lang):
    a = ease(p / 0.25) if p < 0.25 else 1.0
    cx = W // 2
    logo(d, cx - 300, 380, 150)
    text(d, (cx - 120, 400), "aterkeep", font("sans_bold", 118, lang), FG, "la", lang)
    y = 580
    text(d, (cx, y), L["tagline1"], font("sans", 46, lang), FG_DIM, "ma", lang)
    text(d, (cx, y + 70), L["tagline2"], font("sans_semi", 46, lang), OK, "ma", lang)
    _ = a


def server_card(d, x, y, w, state, addr, L, lang, metrics=None):
    """Panelin sunucu kartinin sadelestirilmis hali."""
    h = 300 if metrics else 190
    rrect(d, (x, y, x + w, y + h), 10, fill=PANEL, outline=LINE)
    col = {"online": OK, "offline": FG_MUTE, "queue": WARN}[state]
    label = {"online": L["st_online"], "offline": L["st_offline"], "queue": L["st_queue"]}[state]
    # durum bandi
    rrect(d, (x + 1, y + 1, x + w - 1, y + 74), 10,
          fill=mix(PANEL, col, 0.12))
    d.ellipse((x + 34, y + 31, x + 48, y + 45), fill=col)
    text(d, (x + 66, y + 22), label, font("sans_bold", 30, lang), col, "la", lang)
    text(d, (x + w - 34, y + 27), addr, font("mono", 24), FG_DIM, "ra")
    if metrics:
        mw = (w - 80) // 3
        for i, (k, v, c) in enumerate(metrics):
            mx = x + 40 + i * mw
            text(d, (mx, y + 120), k, font("sans_semi", 20, lang), FG_MUTE, "la", lang)
            text(d, (mx, y + 152), v, font("mono_bold", 46), c, "la")


def scene_problem(d, L, p, lang):
    cx = W // 2
    text(d, (cx, 250), L["eyebrow_problem"], font("mono", 24), FG_MUTE, "ma", lang)
    text(d, (cx, 310), L["problem_h"], font("sans_bold", 62, lang), FG, "ma", lang)
    server_card(d, cx - 520, 450, 1040, "offline", "beeestork.aternos.me", L, lang)
    text(d, (cx, 700), L["problem_p"], font("sans", 34, lang), FG_DIM, "ma", lang)


def scene_queue(d, L, p, lang):
    cx = W // 2
    text(d, (cx, 190), L["eyebrow_queue"], font("mono", 24), FG_MUTE, "ma", lang)
    text(d, (cx, 250), L["queue_h"], font("sans_bold", 62, lang), FG, "ma", lang)

    x, y, w = cx - 520, 400, 1040
    rrect(d, (x, y, x + w, y + 250), 10, fill=PANEL, outline=LINE)

    # Sayac 824 -> 0. Videonun asil ani: bir keep-alive scriptinin YAPAMADIGI sey.
    prog = ease(min(1.0, p / 0.55))
    pos = int(824 * (1 - prog))
    done = prog >= 1.0
    text(d, (x + 46, y + 40), f"{pos}", font("mono_bold", 80), OK if done else FG, "la")
    text(d, (x + w - 46, y + 62), L["eta"], font("mono", 28), FG_MUTE, "ra", lang)

    # ilerleme
    ty = y + 158
    rrect(d, (x + 46, ty, x + w - 46, ty + 14), 7, fill=PANEL_2)
    fw = int((w - 92) * prog)
    if fw > 8:
        rrect(d, (x + 46, ty, x + 46 + fw, ty + 14), 7, fill=OK if done else WARN)

    if done:
        # Onay isareti CIZILIYOR: U+2713 Segoe UI Semibold'da yok ve tofu
        # kutusu olarak cikiyordu. Cizim her dilde, her fontta ayni gorunur.
        f = font("sans_semi", 30, lang)
        msg = shape(L["confirmed"], lang)
        if lang == "ar":
            # RTL: metin saga yaslanir, isaret onun SAGINDA durur.
            right = x + w - 46
            d.text((right, y + 196), msg, font=f, fill=OK, anchor="ra")
            cxk, cyk = right + 14, y + 212
        else:
            cxk, cyk = x + 52, y + 212
            d.text((x + 92, y + 196), msg, font=f, fill=OK, anchor="la")
        d.line([(cxk, cyk), (cxk + 9, cyk + 10), (cxk + 26, cyk - 12)],
               fill=OK, width=4, joint="curve")

    text(d, (cx, 730), L["queue_p1"], font("sans", 32, lang), FG_DIM, "ma", lang)
    text(d, (cx, 780), L["queue_p2"], font("sans_semi", 32, lang), FG, "ma", lang)


def scene_login(d, L, p, lang):
    cx = W // 2
    text(d, (cx, 210), L["eyebrow_setup"], font("mono", 24), FG_MUTE, "ma", lang)
    text(d, (cx, 270), L["login_h"], font("sans_bold", 62, lang), FG, "ma", lang)

    x, y, w = cx - 400, 420, 800
    rrect(d, (x, y, x + w, y + 250), 10, fill=PANEL, outline=LINE)
    for i, (lab, val) in enumerate([(L["f_user"], "beeestork"), (L["f_pass"], "••••••••••")]):
        fy = y + 36 + i * 100
        text(d, (x + 36, fy), lab, font("sans", 22, lang), FG_MUTE, "la", lang)
        rrect(d, (x + 36, fy + 32, x + w - 36, fy + 88), 6, fill=BG, outline=LINE)
        text(d, (x + 52, fy + 46), val, font("mono", 28), FG_DIM, "la")
        if i == 1 and int(p * 100) % 2 == 0:   # imlec
            cxr = x + 52 + int(font("mono", 28).getlength(val)) + 4
            d.line([(cxr, fy + 44), (cxr, fy + 76)], fill=OK, width=2)

    old = L["login_old"]
    ow = font("sans", 30, lang).getlength(shape(old, lang))
    text(d, (cx, 730), old, font("sans", 30, lang), FG_MUTE, "ma", lang)
    d.line([(cx - ow / 2, 748), (cx + ow / 2, 748)], fill=FG_MUTE, width=2)
    text(d, (cx, 790), L["login_new"], font("sans_semi", 34, lang), FG, "ma", lang)


def scene_online(d, L, p, lang):
    cx = W // 2
    text(d, (cx, 230), L["eyebrow_result"], font("mono", 24), FG_MUTE, "ma", lang)
    text(d, (cx, 290), L["online_h"], font("sans_bold", 62, lang), FG, "ma", lang)
    server_card(d, cx - 560, 430, 1120, "online", "beeestork.aternos.me", L, lang,
                metrics=[(L["m_players"], "2 / 20", FG),
                         (L["m_tps"], "20.0", OK),
                         (L["m_ram"], "2450 MB", FG)])


def scene_features(d, L, p, lang):
    cx = W // 2
    text(d, (cx, 200), L["eyebrow_what"], font("mono", 24), FG_MUTE, "ma", lang)
    items = [(L["f1_t"], L["f1_d"]), (L["f2_t"], L["f2_d"]),
             (L["f3_t"], L["f3_d"]), (L["f4_t"], L["f4_d"])]
    bw, bh, gap = 520, 170, 30
    x0 = cx - bw - gap // 2
    y0 = 300
    for i, (tt, dd) in enumerate(items):
        bx = x0 + (i % 2) * (bw + gap)
        by = y0 + (i // 2) * (bh + gap)
        # her kutu sirayla belirir
        appear = ease(min(1.0, max(0.0, (p - i * 0.08) / 0.25)))
        if appear <= 0.02:
            continue
        rrect(d, (bx, by, bx + bw, by + bh), 10,
              fill=mix(BG, PANEL, appear), outline=mix(BG, LINE, appear))
        text(d, (bx + 32, by + 34), tt, font("sans_bold", 32, lang),
             mix(BG, FG, appear), "la", lang)
        text(d, (bx + 32, by + 88), dd, font("sans", 24, lang),
             mix(BG, FG_DIM, appear), "la", lang)


def scene_cta(d, L, p, lang):
    cx = W // 2
    logo(d, cx - 250, 250, 110)
    text(d, (cx - 110, 268), "aterkeep", font("sans_bold", 86, lang), FG, "la", lang)
    text(d, (cx, 440), L["cta_h"], font("sans_bold", 56, lang), FG, "ma", lang)
    mail = "berlaylc2138@gmail.com"
    mw = font("mono_bold", 44).getlength(mail)
    rrect(d, (cx - mw / 2 - 40, 530, cx + mw / 2 + 40, 610), 8,
          fill=(16, 34, 24), outline=(39, 196, 107, 255))
    text(d, (cx, 548), mail, font("mono_bold", 44), OK, "ma")
    text(d, (cx, 680), L["cta_lic"], font("sans", 26, lang), FG_MUTE, "ma", lang)
    text(d, (cx, 726), L["cta_tos"], font("sans", 26, lang), FG_MUTE, "ma", lang)
    text(d, (cx, 772), L["cta_aff"], font("sans", 26, lang), FG_MUTE, "ma", lang)


SCENES = [
    (scene_title,    3.4),
    (scene_problem,  4.0),
    (scene_queue,    5.6),
    (scene_login,    4.2),
    (scene_online,   4.0),
    (scene_features, 4.2),
    (scene_cta,      4.6),
]


def scene_offset(fn, L, lang):
    """Sahnenin dikey ortalanmasi icin SABIT kaydirma.

    Kare basina olcseydik sayac daralinca ya da kutular belirince tum
    kompozisyon oynardi. Son kareye (p=1) gore bir kez olculur.
    """
    probe = Image.new("RGB", (W, H), BG)
    fn(ImageDraw.Draw(probe), L, 1.0, lang)
    # Arka plan SIYAH DEGIL (11,13,16); "sifir olmayan" testi tum kareyi
    # icerik sanip kaydirmayi sifirliyordu. Farki gercek zemine gore al.
    bbox = ImageChops.difference(probe, Image.new("RGB", (W, H), BG)).getbbox()
    if not bbox:
        return 0
    top, bottom = bbox[1], bbox[3]
    # Optik merkez tam ortanin biraz uzerinde durur.
    target = (H - (bottom - top)) * 0.46
    return int(target - top)


def render(lang, L, outdir, fps):
    frames = 0
    total = sum(dur for _, dur in SCENES)
    for si, (fn, dur) in enumerate(SCENES):
        n = int(dur * fps)
        dy = scene_offset(fn, L, lang)
        for i in range(n):
            p = i / max(1, n - 1)
            img = Image.new("RGB", (W, H), BG)
            layer = Image.new("RGB", (W, H), BG)
            fn(ImageDraw.Draw(layer), L, p, lang)
            img.paste(layer, (0, dy))
            d = ImageDraw.Draw(img)
            # ust sol marka serisi — sahneyle birlikte kaymaz
            logo(d, 60, 56, 34)
            text(d, (106, 60), "aterkeep", font("mono", 26), FG_MUTE, "la")
            # ilerleme cizgisi
            done = (sum(dd for _, dd in SCENES[:si]) + p * dur) / total
            d.rectangle((0, H - 5, int(W * done), H), fill=OK)
            # sahne girisinde kisa fade
            img = fade_layer(img, min(1.0, (i / fps) / 0.22))
            img.save(os.path.join(outdir, f"f{frames:05d}.png"))
            frames += 1
    return frames


def encode(ffmpeg, outdir, dest, fps):
    cmd = [ffmpeg, "-y", "-loglevel", "error",
           "-framerate", str(fps), "-i", os.path.join(outdir, "f%05d.png"),
           "-c:v", "libx264", "-preset", "slow", "-crf", "20",
           # yuv420p + cift boyut: her oynaticida acilsin
           "-pix_fmt", "yuv420p", "-movflags", "+faststart", dest]
    subprocess.run(cmd, check=True)


def main():
    from promo_strings import STRINGS  # ayni klasorde

    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    fps = 25
    if "--fps" in sys.argv:
        fps = int(sys.argv[sys.argv.index("--fps") + 1])

    ffmpeg = os.environ.get("ATERKEEP_FFMPEG") or shutil.which("ffmpeg")
    if not ffmpeg:
        sys.exit("ffmpeg bulunamadi: PATH'e ekle ya da ATERKEEP_FFMPEG ile yolunu ver")

    langs = args or list(STRINGS)
    here = os.path.dirname(os.path.abspath(__file__))
    outroot = os.path.join(here, "video")
    os.makedirs(outroot, exist_ok=True)

    for lang in langs:
        if lang not in STRINGS:
            print(f"  {lang}: ceviri yok, atlandi")
            continue
        L = dict(STRINGS["en"])
        L.update(STRINGS[lang])          # eksik anahtar Ingilizce'ye duser
        with tempfile.TemporaryDirectory() as tmp:
            n = render(lang, L, tmp, fps)
            dest = os.path.join(outroot, f"aterkeep-{lang}.mp4")
            encode(ffmpeg, tmp, dest, fps)
        mb = os.path.getsize(dest) / 1e6
        print(f"  {lang}: {n} kare -> {os.path.basename(dest)} ({mb:.1f} MB)")


if __name__ == "__main__":
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    main()
