#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Installe les dépendances nécessaires pour compiler civium-tauri sur Windows.

.DESCRIPTION
    Le crate `rusqlite` avec la feature `bundled-sqlcipher-vendored-openssl`
    compile SQLCipher + OpenSSL depuis les sources. Cela exige :
      - Strawberry Perl  (le Perl MSYS fourni avec Git manque des modules requis)
      - cmake            (pour la phase de configuration d'OpenSSL)

    Ce script installe ces deux outils via winget, puis reconfigure
    la variable d'environnement PATH (machine) pour que Strawberry Perl
    soit prioritaire sur le Perl MSYS de Git.

.NOTES
    Lancer une fois par poste développeur, en tant qu'Administrateur :
        powershell -ExecutionPolicy Bypass -File scripts\setup-windows-dev.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Step([string]$msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Ok([string]$msg)   { Write-Host "    OK  $msg" -ForegroundColor Green }
function Write-Skip([string]$msg) { Write-Host "    --  $msg (déjà installé)" -ForegroundColor DarkGray }

# ---------------------------------------------------------------------------
# 1. Strawberry Perl
# ---------------------------------------------------------------------------
Write-Step "Vérification de Strawberry Perl..."
$perlExe = "C:\Strawberry\perl\bin\perl.exe"
if (Test-Path $perlExe) {
    Write-Skip "Strawberry Perl"
} else {
    Write-Host "    Installation via winget..." -ForegroundColor Yellow
    winget install --id StrawberryPerl.StrawberryPerl --silent --accept-package-agreements --accept-source-agreements
    if (!(Test-Path $perlExe)) {
        Write-Error "Strawberry Perl introuvable après installation. Vérifiez winget."
    }
    Write-Ok "Strawberry Perl installé"
}

# ---------------------------------------------------------------------------
# 2. cmake
# ---------------------------------------------------------------------------
Write-Step "Vérification de cmake..."
$cmakeExe = "C:\Program Files\CMake\bin\cmake.exe"
if (Test-Path $cmakeExe) {
    Write-Skip "cmake"
} else {
    Write-Host "    Installation via winget..." -ForegroundColor Yellow
    winget install --id Kitware.CMake --silent --accept-package-agreements --accept-source-agreements
    if (!(Test-Path $cmakeExe)) {
        Write-Error "cmake introuvable après installation. Vérifiez winget."
    }
    Write-Ok "cmake installé"
}

# ---------------------------------------------------------------------------
# 3. Configurer le PATH machine pour prioriser Strawberry Perl sur MSYS perl
# ---------------------------------------------------------------------------
Write-Step "Configuration du PATH système..."

$machinePath = [System.Environment]::GetEnvironmentVariable('PATH', 'Machine')
$pathEntries  = $machinePath -split ';' | Where-Object { $_ -ne '' }

$toAdd = @(
    'C:\Strawberry\perl\bin',
    'C:\Strawberry\c\bin',
    'C:\Program Files\CMake\bin'
)

$changed = $false
foreach ($entry in $toAdd) {
    if ($pathEntries -notcontains $entry) {
        # Insérer en tête pour avoir la priorité sur Git's MSYS perl
        $pathEntries = @($entry) + $pathEntries
        $changed = $true
        Write-Ok "Ajouté en tête du PATH : $entry"
    } else {
        Write-Skip $entry
    }
}

if ($changed) {
    $newPath = ($pathEntries | Where-Object { $_ -ne '' }) -join ';'
    [System.Environment]::SetEnvironmentVariable('PATH', $newPath, 'Machine')
    Write-Host "`n    PATH machine mis à jour." -ForegroundColor Green
    Write-Host "    Redémarrez votre terminal (ou VS Code / IntelliJ) pour appliquer." -ForegroundColor Yellow
} else {
    Write-Host "`n    PATH déjà configuré correctement." -ForegroundColor DarkGray
}

# ---------------------------------------------------------------------------
# 4. Vérification finale
# ---------------------------------------------------------------------------
Write-Step "Vérification finale..."

$env:PATH = 'C:\Strawberry\perl\bin;C:\Strawberry\c\bin;C:\Program Files\CMake\bin;' + $env:PATH

$perlVersion  = & 'C:\Strawberry\perl\bin\perl.exe' --version 2>&1 | Select-String 'v\d+\.\d+\.\d+'
$cmakeVersion = & 'C:\Program Files\CMake\bin\cmake.exe' --version 2>&1 | Select-Object -First 1

Write-Host "    perl  : $perlVersion"
Write-Host "    cmake : $cmakeVersion"

Write-Host "`nSetup terminé. Vous pouvez maintenant lancer :" -ForegroundColor Green
Write-Host "    cd desktop\civium-tauri && cargo tauri dev" -ForegroundColor White
