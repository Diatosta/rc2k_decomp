<#
tools\windows\build.ps1: builds rc2k.exe on Windows itself (without cargo-xwin) into dist\<target>\, and
with -Deploy copies that folder into the game's folder. The Windows side of build.sh.

    build [windows|windows-x64] [-Deploy] [-GameDir <folder>] [-DebugBuild]
    (build.cmd, in the repository's top folder, runs this script whatever the machine's script
    policy is; or powershell -ExecutionPolicy Bypass -File tools\windows\build.ps1 [-Target ...])

    windows-x64           64 bits (the default)       dist\windows-x64\
    windows               32 bits                     dist\windows\
    -Deploy               also copy dist\<target>\ into the game's folder: -GameDir, or the
                          line DEPLOY_WINDOWS=<folder> (DEPLOY_WINDOWS_X64 for 64 bits) of
                          deploy.local in the repository's root
    -GameDir <folder>     the game's folder (the one that holds `files\` and `var\`); naming
                          it deploys there
    -DebugBuild           an unoptimised build

dist\<target>\ holds rc2k.exe and SDL3.dll: all the program needs beside the game's own
files.

What it does:
  1. finds Visual Studio's C++ tools (vswhere);
  2. downloads SDL's ready-made library (the version and SHA-256 pinned in tools/fetch_sdl3.sh)
     and checks its hash;
  3. runs cargo for the target and gathers dist\<target>\.

Needs:
  - rustup (the toolchain named in rust-toolchain.toml is installed on first use);
  - Visual Studio 2022 or its Build Tools with "Desktop development with C++" (MSVC and a
    Windows SDK);
  - a checkout without line-ending conversion (.gitattributes asks for that).
#>
param(
    [ValidateSet('windows', 'windows-x64')][string]$Target = 'windows-x64',
    [switch]$Deploy,
    [string]$GameDir = '',
    [switch]$DebugBuild
)
$ErrorActionPreference = 'Stop'
# This script is tools\windows\build.ps1: the repository is two folders up.
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Fail($message) { Write-Host "build.ps1: $message" -ForegroundColor Red; exit 1 }

$x64 = $Target -eq 'windows-x64'
if ($x64) { $Arch = 'x64'; $triple = 'x86_64-pc-windows-msvc' } else { $Arch = 'x86'; $triple = 'i686-pc-windows-msvc' }
if ($DebugBuild) { $profileName = 'debug'; $cargoProfile = '' } else { $profileName = 'release'; $cargoProfile = '--release' }
$Dist = Join-Path $Repo "dist\$Target"

# Where -Deploy copies to, worked out before the build so that a missing line stops at once.
function Read-DeployLine($name) {
    $file = Join-Path $Repo 'deploy.local'
    if (-not (Test-Path $file)) { return '' }
    foreach ($line in Get-Content $file) {
        if ($line -match "^\s*$name=(.*)$") { return $Matches[1].Trim().Trim('"').Trim("'") }
    }
    return ''
}
$deployTo = ''
if ($GameDir) {
    $deployTo = $GameDir
} elseif ($Deploy) {
    if ($x64) { $lineName = 'DEPLOY_WINDOWS_X64' } else { $lineName = 'DEPLOY_WINDOWS' }
    $deployTo = Read-DeployLine $lineName
    if (-not $deployTo) { Fail "-Deploy needs -GameDir <folder>, or a line $lineName=<folder> in deploy.local" }
}

# The tools.
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Fail 'no cargo: install rustup (https://rustup.rs)' }
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path $vswhere)) { Fail 'no Visual Studio installer found (vswhere.exe): install Visual Studio 2022 or its Build Tools' }
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { Fail 'no Visual Studio with the C++ tools ("Desktop development with C++")' }
$vcvars = Join-Path $vs 'VC\Auxiliary\Build\vcvarsall.bat'

# A developer environment for the target processor, built from this machine's.
$hostX64 = $env:PROCESSOR_ARCHITECTURE -eq 'AMD64'
if ($x64) { if ($hostX64) { $vsArch = 'x64' } else { $vsArch = 'x86_x64' } }
else { if ($hostX64) { $vsArch = 'x64_x86' } else { $vsArch = 'x86' } }

# Runs batch lines inside that environment, from the repository's root. A batch file of
# its own, so that no quoting passes through PowerShell into cmd.
function Invoke-Vs([string[]]$lines) {
    $batch = Join-Path $env:TEMP "rc2k_build_$PID.cmd"
    $text = @('@echo off', "call `"$vcvars`" $vsArch >nul || exit /b 1", "cd /d `"$Repo`"") + $lines
    Set-Content -Path $batch -Value $text -Encoding Ascii
    try {
        cmd /c $batch
        if ($LASTEXITCODE -ne 0) { Fail "failed: $($lines -join ' / ')" }
    } finally { Remove-Item -Force $batch }
}

# SDL's ready-made library for Visual C into third_party\sdl3, as tools/fetch_sdl3.sh does on
# the other systems; the version and the hash are read from that script, the one place that
# pins them.
function Get-Sdl {
    $pins = Get-Content (Join-Path $Repo 'tools\fetch_sdl3.sh')
    $version = ($pins | Select-String '^VERSION=(.+)$').Matches[0].Groups[1].Value.Trim()
    $sha256 = ($pins | Select-String '^SHA256=([0-9a-fA-F]{64})\s*$').Matches[0].Groups[1].Value
    $dest = Join-Path $Repo 'third_party\sdl3'
    $stamp = Join-Path $dest 'VERSION'
    if ((Test-Path $stamp) -and ((Get-Content $stamp -Raw).Trim() -eq $version)) {
        Write-Host "SDL $version is already in $dest"
        return
    }
    $url = "https://github.com/libsdl-org/SDL/releases/download/release-$version/SDL3-devel-$version-VC.zip"
    $work = Join-Path $env:TEMP "rc2k_sdl_$PID"
    New-Item -ItemType Directory -Force $work | Out-Null
    try {
        $zip = Join-Path $work 'sdl3.zip'
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        $ProgressPreference = 'SilentlyContinue'
        try { Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing }
        catch { Fail "could not download $url ($($_.Exception.Message))" }
        $got = (Get-FileHash -Algorithm SHA256 $zip).Hash
        if ($got -ne $sha256) { Fail "SDL's download has SHA-256 $got, not the pinned ${sha256}: not used" }
        Expand-Archive -Path $zip -DestinationPath $work
        $from = Join-Path $work "SDL3-$version"
        if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
        New-Item -ItemType Directory -Force $dest | Out-Null
        Copy-Item -Recurse (Join-Path $from 'include'), (Join-Path $from 'lib'), (Join-Path $from 'LICENSE.txt') $dest
        [IO.File]::WriteAllText($stamp, "$version`n")
        Write-Host "SDL $version is in $dest"
    } finally { Remove-Item -Recurse -Force $work }
}

Write-Host "== SDL"
Get-Sdl
Write-Host "== rc2k.exe ($triple, $profileName)"
# The repository's cargo config names LLVM's linker for 32 bits (the Linux cross build);
# here Visual Studio's own is used.
Invoke-Vs @('set CARGO_TARGET_I686_PC_WINDOWS_MSVC_LINKER=link.exe',
    "rustup target add $triple || exit /b 1",
    "cargo build $cargoProfile --target $triple --bin rc2k || exit /b 1")

$files = @((Join-Path $Repo "target\$triple\$profileName\rc2k.exe"),
    (Join-Path $Repo "third_party\sdl3\lib\$Arch\SDL3.dll"))
foreach ($f in $files) { if (-not (Test-Path $f)) { Fail "missing after the build: $f" } }
if (Test-Path $Dist) { Remove-Item -Recurse -Force $Dist }
New-Item -ItemType Directory -Force $Dist | Out-Null
foreach ($f in $files) { Copy-Item -Force $f $Dist }
Write-Host "built $Dist\:"
Get-ChildItem $Dist | ForEach-Object { Write-Host ("  {0,10}  {1}" -f $_.Length, $_.Name) }

if ($deployTo) {
    if (-not (Test-Path $deployTo)) { New-Item -ItemType Directory -Force $deployTo | Out-Null }
    Get-ChildItem $Dist | ForEach-Object { Copy-Item -Force -Recurse $_.FullName $deployTo }
    Write-Host "deployed rc2k.exe and SDL3.dll to $deployTo"
}
