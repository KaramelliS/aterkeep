# aterkeep - Windows otomatik baslatma kurulumu.
#
# SORUN: aterkeep'in oturum sifreleme anahtari panel parolasindan turetilir ve
# diske YAZILMAZ. Bu, config/ klasoru calinsa bile Aternos cerezlerinin
# okunamamasini saglar - ama makine yeniden basladiginda daemon'in parolayi
# soracak kimsesi olmadigi anlamina da gelir.
#
# COZUM: parola DPAPI ile sifrelenip saklanir. Windows'un DPAPI'si anahtari
# KULLANICI HESABINA ve MAKINEYE baglar: dosya baska bir makineye ya da baska
# bir kullaniciya kopyalanirsa cozulemez. Yani "klasoru kopyalayan her seyi
# alir" ozelligi korunur; kaybedilen sey yalnizca "bu makinede bu kullanici
# olarak oturum acmis birine karsi" korumadir - ki o kisi zaten senin
# oturumundasin demektir.
#
# Kullanim:
#   .\scripts\install-autostart.ps1                       # kurar, parolayi sorar
#   .\scripts\install-autostart.ps1 -Remove               # kaldirir
#   .\scripts\install-autostart.ps1 -InstallDir C:\aterkeep
[CmdletBinding()]
param(
  [string]$InstallDir = (Split-Path -Parent $PSScriptRoot),
  [string]$TaskName = "aterkeep",
  # Otomatik kurulum icin (birden fazla makineye dagitim, test). Verilmezse
  # parola ekranda gorunmeden sorulur. DIKKAT: bu parametreyi kullanirsan parola
  # PowerShell komut gecmisine duser.
  [string]$Password,
  [switch]$Remove
)

$ErrorActionPreference = "Stop"

if ($Remove) {
  if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    Write-Host "gorev kaldirildi: $TaskName"
  } else {
    Write-Host "gorev zaten yok: $TaskName"
  }
  $keyFile = Join-Path $InstallDir "config\autostart.key"
  if (Test-Path $keyFile) {
    Remove-Item $keyFile -Force
    Write-Host "saklanan parola silindi: $keyFile"
  }
  return
}

$exe = Join-Path $InstallDir "aterkeep.exe"
if (-not (Test-Path $exe)) {
  throw "aterkeep.exe bulunamadi: $exe  (-InstallDir ile dogru klasoru ver)"
}
$starter = Join-Path $PSScriptRoot "start-aterkeep.ps1"
if (-not (Test-Path $starter)) {
  throw "start-aterkeep.ps1 bulunamadi: $starter"
}

# Parolayi al ve DPAPI ile sifrele. ConvertFrom-SecureString anahtar
# verilmediginde DPAPI kullanir: cikti yalnizca AYNI kullanici + AYNI makinede
# cozulebilir. Parola hicbir zaman duz metin olarak diske yazilmaz.
if ($Password) {
  $secure = ConvertTo-SecureString $Password -AsPlainText -Force
} else {
  $secure = Read-Host -AsSecureString "Panel parolasi (kurulumda belirledigin)"
}
if ($secure.Length -eq 0) { throw "parola bos olamaz" }

$configDir = Join-Path $InstallDir "config"
if (-not (Test-Path $configDir)) { New-Item -ItemType Directory -Path $configDir | Out-Null }
$keyFile = Join-Path $configDir "autostart.key"
# ASCII: DPAPI ciktisi zaten onaltilik metindir ve BOM'suz olmasi gerekir -
# BOM'lu yazilirsa cozme asamasi "giris dizesi dogru bicimde degildi" ile
# patlar (olculdu).
ConvertFrom-SecureString -SecureString $secure | Set-Content -Path $keyFile -Encoding ASCII

# Dosyayi yalnizca bu kullaniciya ac. DPAPI zaten baskasinin cozmesini
# engelliyor; bu, dosyanin okunmasini da engelleyerek savunmayi katmanlar.
icacls $keyFile /inheritance:r /grant:r "$($env:USERNAME):(R,W)" | Out-Null

$action = New-ScheduledTaskAction -Execute "powershell.exe" `
  -Argument "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$starter`" -InstallDir `"$InstallDir`""
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
# Dizustunde pil moduna gecince gorevin durdurulmasini istemiyoruz: urunun
# vaadi 7/24 acik kalmak.
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries `
  -DontStopIfGoingOnBatteries -StartWhenAvailable -ExecutionTimeLimit ([TimeSpan]::Zero)

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger `
  -Settings $settings -Description "aterkeep - Aternos 7/24 sunucu bekcisi" -Force | Out-Null

Write-Host ""
Write-Host "kuruldu. oturum acildiginda aterkeep kendi baslayacak."
Write-Host "  gorev    : $TaskName"
Write-Host "  klasor   : $InstallDir"
Write-Host "  parola   : $keyFile (DPAPI - yalnizca $env:USERNAME @ $env:COMPUTERNAME cozebilir)"
Write-Host ""
Write-Host "hemen baslatmak icin : Start-ScheduledTask -TaskName $TaskName"
Write-Host "kaldirmak icin       : .\scripts\install-autostart.ps1 -Remove"
