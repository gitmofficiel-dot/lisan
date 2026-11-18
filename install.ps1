# ==============================================================================
# مثبت لغة البرمجة العربية "لسان" واستوديو لسان الرسمي لنظام Windows 10 / 11
# الاستخدام بأمر واحد في PowerShell:
#   irm https://raw.githubusercontent.com/gitmofficiel-dot/lisan/main/install.ps1 | iex
# ==============================================================================

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = "gitmofficiel-dot/lisan"
$Asset = "lisan-windows-x86_64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$Asset"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\Lisan"

Write-Host "🔍 جاري تحضير تثبيت لغة لسان 1.0 واستوديو سطح المكتب لنظام Windows..." -ForegroundColor Cyan

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("lisan-install-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $ZipPath = Join-Path $TempDir $Asset
    Write-Host "📥 جاري تنزيل أحدث إصدار من GitHub ($Asset)..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

    Write-Host "📦 جاري فك الضغط إلى: $InstallDir ..." -ForegroundColor Cyan
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

    $ExePath = Join-Path $InstallDir "lisan.exe"
    if (-not (Test-Path $ExePath)) {
        $Found = Get-ChildItem -Path $InstallDir -Filter "lisan.exe" -Recurse | Select-Object -First 1
        if ($null -eq $Found) {
            throw "تعذّر العثور على lisan.exe بعد فك ضغط الحزمة."
        }
        Copy-Item $Found.FullName $ExePath -Force
    }

    # إضافة مجلد لسان إلى متغير المسار PATH للمستخدم الحالي
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ([string]::IsNullOrEmpty($UserPath)) {
        $UserPath = ""
    }
    $PathEntries = $UserPath -split ";" | Where-Object { $_ -ne "" }
    if ($PathEntries -notcontains $InstallDir) {
        $NewUserPath = if ($UserPath.EndsWith(";") -or $UserPath.Length -eq 0) {
            "$UserPath$InstallDir"
        } else {
            "$UserPath;$InstallDir"
        }
        [Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
        Write-Host "✅ تمت إضافة مسار لسان تلقائياً إلى متغير البيئة PATH للمستخدم." -ForegroundColor Green
    }
    if (($env:Path -split ";") -notcontains $InstallDir) {
        $env:Path = "$InstallDir;$env:Path"
    }

    # إنشاء مشغل Lisan Studio.bat على سطح المكتب وفي مجلد التثبيت
    $LauncherContent = @"
@echo off
chcp 65001 >nul
title استوديو لسان لسطح المكتب - Lisan Studio
"$ExePath" --studio
"@
    $LocalLauncher = Join-Path $InstallDir "Lisan Studio.bat"
    Set-Content -Path $LocalLauncher -Value $LauncherContent -Encoding UTF8

    $DesktopDir = [Environment]::GetFolderPath("Desktop")
    if (-not [string]::IsNullOrEmpty($DesktopDir) -and (Test-Path $DesktopDir)) {
        $DesktopLauncher = Join-Path $DesktopDir "Lisan Studio.bat"
        Set-Content -Path $DesktopLauncher -Value $LauncherContent -Encoding UTF8
        Write-Host "🖥️ تم إنشاء مشغّل (Lisan Studio.bat) على سطح المكتب!" -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "✨ تم تثبيت لغة لسان واستوديو لسان بنجاح! 🎉" -ForegroundColor Green
    & $ExePath --version
    Write-Host ""
    Write-Host "🚀 للبدء فوراً في نافذة PowerShell:" -ForegroundColor Yellow
    Write-Host "   lisan                 فتح الطرفية التفاعلية (REPL)"
    Write-Host "   lisan --studio        تشغيل تطبيق استوديو لسان لسطح المكتب"
    Write-Host "   lisan ملف.lsn         ترجمة ملف لسان إلى ملف تنفيذي (.exe)"
}
finally {
    if (Test-Path $TempDir) {
        Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
