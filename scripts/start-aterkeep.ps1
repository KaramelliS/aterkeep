# aterkeep - zamanlanmis gorevin cagirdigi baslatici.
#
# DPAPI ile saklanan panel parolasini cozer, ATERKEEP_KEY olarak verir ve
# daemon'i baslatir. Parola yalnizca bu surecin bellegine girer; komut
# satirinda gorunmez (Get-Process/wmic ile okunamaz) ve log'a yazilmaz.
[CmdletBinding()]
param(
  [string]$InstallDir = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = "Stop"

$exe = Join-Path $InstallDir "aterkeep.exe"
$keyFile = Join-Path $InstallDir "config\autostart.key"
$log = Join-Path $InstallDir "autostart.log"

function Note($msg) {
  "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')  $msg" | Add-Content -Path $log -Encoding utf8
}

if (-not (Test-Path $exe)) { Note "aterkeep.exe yok: $exe"; exit 1 }
if (-not (Test-Path $keyFile)) { Note "autostart.key yok - install-autostart.ps1 calistir"; exit 1 }

try {
  # DPAPI: baska kullanici/makinede bu satir hata verir. Sessizce duz metne
  # dusmek yerine acikca basarisiz olmasi DOGRUDUR - yanlis parolayla acilan
  # bir daemon, kullaniciya sebebi belirsiz bir ariza olarak gorunurdu.
  # .Trim() sart: Set-Content sona satir sonu ekler ve ConvertTo-SecureString
  # bunu kabul etmez.
  $secure = (Get-Content $keyFile -Raw).Trim() | ConvertTo-SecureString
} catch {
  Note "parola cozulemedi (baska kullanici/makine?): $($_.Exception.Message)"
  exit 1
}

$bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
try {
  $env:ATERKEEP_KEY = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
} finally {
  # Parolayi yonetilmeyen bellekten hemen sil.
  [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
}

Note "aterkeep baslatiliyor ($InstallDir)"
try {
  # Daemon'in kendi ciktisi AYRI dosyalara yazilir. Onceden `*>> $log`
  # kullaniliyordu: PowerShell 5.1'de bu yonlendirme UTF-16 yazar, bizim
  # Add-Content ile yazdigimiz UTF-8 satirlarla ayni dosyada karisir ve log
  # okunamaz hale gelirdi (olculdu).
  #
  # WorkingDirectory SART: zamanlanmis gorevler C:\Windows\System32'de baslar;
  # bot/ ve config/ daemon'in baslatildigi yerde aranir, orada bulunmaz ve
  # daemon kurulum moduna duserdi.
  #
  # ATERKEEP_KEY cocuk surece ortam degiskeni olarak miras kalir; komut
  # satirinda gorunmez.
  $p = Start-Process -FilePath $exe -WorkingDirectory $InstallDir -NoNewWindow -PassThru `
    -RedirectStandardOutput (Join-Path $InstallDir "aterkeep.out.log") `
    -RedirectStandardError (Join-Path $InstallDir "aterkeep.err.log")
  $p.WaitForExit()
  Note "aterkeep kapandi (cikis kodu $($p.ExitCode))"
} finally {
  $env:ATERKEEP_KEY = $null
}
