#!/usr/bin/env bash
# aterkeep — systemd servisi kurulumu (Linux).
#
# aterkeep'in oturum sifreleme anahtari panel parolasindan turetilir ve diske
# yazilmaz; boylece config/ klasoru calinsa bile Aternos cerezleri okunamaz.
# Makine yeniden basladiginda daemon'in parolayi soracak kimsesi olmadigi icin,
# acilista baslatmak istiyorsan parolanin bir yerde durmasi gerekir.
#
# Burada parola /etc/aterkeep/aterkeep.env dosyasinda, 0600 izniyle ve dosyanin
# sahibi servis kullanicisi olacak sekilde tutulur. TAKAS ACIKTIR: o kullaniciya
# ya da root'a erisen biri Aternos oturumunu da ele gecirir. Bunu istemiyorsan
# servisi kurma ve daemon'i elle baslat:
#     ATERKEEP_KEY='parola' ./aterkeep
set -euo pipefail

INSTALL_DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SERVICE_USER="${SUDO_USER:-$USER}"
UNIT=/etc/systemd/system/aterkeep.service
ENV_DIR=/etc/aterkeep
ENV_FILE="$ENV_DIR/aterkeep.env"

if [ "$(id -u)" -ne 0 ]; then
  echo "root gerekiyor:  sudo $0 $INSTALL_DIR" >&2
  exit 1
fi
if [ ! -x "$INSTALL_DIR/aterkeep" ]; then
  echo "aterkeep bulunamadi: $INSTALL_DIR/aterkeep" >&2
  echo "kullanim: sudo $0 /kurulum/dizini" >&2
  exit 1
fi

# Parolayi ekrana yazmadan al.
read -r -s -p "Panel parolasi (kurulumda belirledigin): " PASS
echo
[ -n "$PASS" ] || { echo "parola bos olamaz" >&2; exit 1; }

install -d -m 700 -o "$SERVICE_USER" "$ENV_DIR"
# umask: dosya bir an bile herkese okunur halde olusmasin.
( umask 077; printf 'ATERKEEP_KEY=%s\n' "$PASS" > "$ENV_FILE" )
chown "$SERVICE_USER" "$ENV_FILE"
chmod 600 "$ENV_FILE"
unset PASS

cat > "$UNIT" <<UNITEOF
[Unit]
Description=aterkeep — Aternos 24/7 server keeper
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$ENV_FILE
ExecStart=$INSTALL_DIR/aterkeep
Restart=always
RestartSec=10
# Parola ortam degiskeninde tasiniyor; surecin cekirdek dokumu ile diske
# dusmesini engelle.
LimitCORE=0
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=read-only
ReadWritePaths=$INSTALL_DIR

[Install]
WantedBy=multi-user.target
UNITEOF

systemctl daemon-reload
systemctl enable --now aterkeep
echo
echo "kuruldu."
echo "  durum : systemctl status aterkeep"
echo "  log   : journalctl -u aterkeep -f"
echo "  parola: $ENV_FILE (0600, sahibi $SERVICE_USER)"
echo "  kaldir: systemctl disable --now aterkeep && rm -f $UNIT $ENV_FILE"
