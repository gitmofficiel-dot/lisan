param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

# ==============================================================================
# سكربت تحزيم لغة لسان واستوديو لسان لنظام Windows 10 / 11
# ينشئ: dist/lisan-windows-x86_64.zip مع Lisan-Studio.bat وإضافة VS Code والأمثلة
# ==============================================================================

$RootDir = Split-Path -Parent $PSScriptRoot
Set-Location $RootDir

if (-not $SkipBuild) {
    Write-Host "🔨 جاري بناء لغة لسان واستوديو سطح المكتب للهدف: $Target ..." -ForegroundColor Cyan
    cargo build --release --target $Target
    if ($LASTEXITCODE -ne 0) {
        throw "فشل بناء مشروع Rust للهدف $Target"
    }
}

$BinPath = Join-Path $RootDir "target\$Target\release\lisan.exe"
if (-not (Test-Path $BinPath)) {
    throw "لم يتم العثور على الملف التنفيذي: $BinPath"
}

$ArchSuffix = if ($Target -like "aarch64-*") { "arm64" } else { "x86_64" }
$ZipName = "lisan-windows-$ArchSuffix.zip"
$DistDir = Join-Path $RootDir "dist"
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

$StageDir = Join-Path ([System.IO.Path]::GetTempPath()) ("lisan-win-stage-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null

try {
    Copy-Item $BinPath (Join-Path $StageDir "lisan.exe") -Force
    Copy-Item (Join-Path $RootDir "README.md") (Join-Path $StageDir "README.md") -Force
    Copy-Item (Join-Path $RootDir "LICENSE") (Join-Path $StageDir "LICENSE") -Force

    # نسخ الأمثلة وإضافة VS Code
    Copy-Item (Join-Path $RootDir "examples") (Join-Path $StageDir "examples") -Recurse -Force
    $EditorsStage = Join-Path $StageDir "editors"
    New-Item -ItemType Directory -Force -Path $EditorsStage | Out-Null
    Copy-Item (Join-Path $RootDir "editors\vscode") (Join-Path $EditorsStage "vscode") -Recurse -Force

    # إنشاء مشغل استوديو لسان لسطح المكتب بنقرة مزدوجة
    $StudioBat = @"
@echo off
chcp 65001 >nul
title استوديو لسان لسطح المكتب - Lisan Studio
"%~dp0lisan.exe" --studio
"@
    Set-Content -Path (Join-Path $StageDir "Lisan-Studio.bat") -Value $StudioBat -Encoding UTF8

    # أيقونة لسان (ICO): صورة فقط، متطابقة على كل الأنظمة
    $IconSrc = Join-Path $RootDir "assets\favicon.ico"
    if (Test-Path $IconSrc) {
        Copy-Item $IconSrc (Join-Path $StageDir "lisan.ico") -Force
    }

    $ZipPath = Join-Path $DistDir $ZipName
    if (Test-Path $ZipPath) {
        Remove-Item $ZipPath -Force
    }

    Compress-Archive -Path (Join-Path $StageDir "*") -DestinationPath $ZipPath -Force
    $Hash = (Get-FileHash -Algorithm SHA256 -Path $ZipPath).Hash.ToLower()
    "$Hash  $ZipName" | Out-File -FilePath "$ZipPath.sha256" -Encoding ascii

    Write-Host "✅ تم إنشاء حزمة ويندوز بنجاح: dist\$ZipName" -ForegroundColor Green
}
finally {
    if (Test-Path $StageDir) {
        Remove-Item $StageDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
