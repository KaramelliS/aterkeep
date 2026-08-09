#!/data/data/com.termux/files/usr/bin/bash
# aterkeep — Termux'ta acilista otomatik baslatma (Android).
#
# Termux'ta systemd yoktur; acilis icin Termux:Boot uygulamasi gerekir
# (F-Droid'den kurulur ve BIR KEZ acilir, yoksa ~/.termux/boot calismaz).
#
# Parola ~/.aterkeep.env icinde 0600 izniyle durur. Android'de uygulama basina
# ayri kullanici oldugundan bu dosyayi baska bir uygulama okuyamaz; ama cihaza
# fiziksel/root erisimi olan biri okuyabilir. Bunu istemiyorsan bu betigi
# kurma, daemon'i elle baslat:
#     ATERKEEP_KEY='parola' ./aterkeep
set -euo pipefail

INSTALL_DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ENV_FILE="$HOME/.aterkeep.env"
BOOT_DIR="$HOME/.termux/boot"
BOOT_SCRIPT="$BOOT_DIR/aterkeep"

if [ ! -x "$INSTALL_DIR/aterkeep" ]; then
  echo "aterkeep bulunamadi: $INSTALL_DIR/aterkeep" >&2
  echo "kullanim: $0 /kurulum/dizini" >&2
  exit 1
fi

read -r -s -p "Panel parolasi (kurulumda belirledigin): " PASS
echo
[ -n "$PASS" ] || { echo "parola bos olamaz" >&2; exit 1; }
( umask 077; printf 'ATERKEEP_KEY=%s\n' "$PASS" > "$ENV_FILE" )
chmod 600 "$ENV_FILE"
unset PASS

mkdir -p "$BOOT_DIR"
cat > "$BOOT_SCRIPT" <<BOOTEOF
#!/data/data/com.termux/files/usr/bin/bash
# aterkeep — Termux:Boot tarafindan acilista calistirilir.
# Uyanik kilit: Android, ekran kapaliyken sureci uyutur ve keep-alive dongusu
# durur; sunucu da kapanir. Bu, "7/24" vaadinin Android'deki on kosuludur.
termux-wake-lock
set -a; . "$ENV_FILE"; set +a
cd "$INSTALL_DIR"
exec ./aterkeep >> "$INSTALL_DIR/aterkeep.log" 2>&1
BOOTEOF
chmod +x "$BOOT_SCRIPT"

echo
echo "kuruldu."
echo "  acilis betigi : $BOOT_SCRIPT"
echo "  parola        : $ENV_FILE (0600)"
echo "  log           : $INSTALL_DIR/aterkeep.log"
echo
echo "ONEMLI: Termux:Boot uygulamasini F-Droid'den kur ve EN AZ BIR KEZ ac —"
echo "aksi halde Android acilista bu betigi calistirmaz."
echo "kaldirmak icin: rm -f $BOOT_SCRIPT $ENV_FILE"
