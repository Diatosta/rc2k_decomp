<#
tools/windows/build.ps1: builds rc2k.exe on Windows itself, without cargo-xwin.

    powershell -ExecutionPolicy Bypass -File tools\windows\build.ps1 [-Arch x86|x64] [-GameDir <folder>]

What it does, from the repository's root:
  1. finds Visual Studio's C++ tools (vswhere) and MSYS2 (default C:\msys64);
  2. fetches SDL's ready-made library (tools/fetch_sdl3.sh, in MSYS2's bash);
  3. builds FFmpeg's decoders with Visual Studio's compiler (tools/build_ffmpeg.sh, in MSYS2's
     bash inside a developer environment for the chosen processor);
  4. runs `cargo build --release` for the target; build.rs puts SDL3.dll and FFmpeg's two DLLs
     next to rc2k.exe, so the release folder runs as it is;
  5. with -GameDir, copies rc2k.exe and the three DLLs into the game's folder (the one that
     holds `files\` and `var\`), as deploy_standalone.sh does on Linux.

Needs:
  - rustup (the toolchain named in rust-toolchain.toml is installed on first use);
  - Visual Studio 2022 or its Build Tools with "Desktop development with C++" (MSVC and a
    Windows SDK);
  - MSYS2 with: pacman -S --needed make diffutils gnupg tar xz unzip curl
  - a checkout without line-ending conversion (.gitattributes asks for that).
#>
param(
    [ValidateSet('x86', 'x64')][string]$Arch = 'x86',
    [string]$GameDir = '',
    [string]$Msys = 'C:\msys64'
)
$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Fail($message) { Write-Host "build.ps1: $message" -ForegroundColor Red; exit 1 }

# The tools.
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Fail 'no cargo: install rustup (https://rustup.rs)' }
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path $vswhere)) { Fail 'no Visual Studio installer found (vswhere.exe)' }
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { Fail 'no Visual Studio with the C++ tools ("Desktop development with C++")' }
$vcvars = Join-Path $vs 'VC\Auxiliary\Build\vcvarsall.bat'
$bash = Join-Path $Msys 'usr\bin\bash.exe'
if (-not (Test-Path $bash)) { Fail "no MSYS2 in $Msys (or pass -Msys <folder>)" }

# A developer environment for the target processor, built from this machine's.
$hostX64 = $env:PROCESSOR_ARCHITECTURE -eq 'AMD64'
$vsArch = if ($Arch -eq 'x64') { if ($hostX64) { 'x64' } else { 'x86_x64' } } else { if ($hostX64) { 'x64_x86' } else { 'x86' } }
$triple = if ($Arch -eq 'x64') { 'x86_64-pc-windows-msvc' } else { 'i686-pc-windows-msvc' }

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

# The batch lines that run a script in MSYS2's bash, in the repository's root, with the
# Windows PATH (and so Visual Studio's tools) inherited.
function Bash-Lines($script) {
    @('set MSYS2_PATH_TYPE=inherit', 'set CHERE_INVOKING=1', 'set MSYSTEM=MSYS', "`"$bash`" -lc `"$script`" || exit /b 1")
}

$missing = & $bash -lc 'for t in make diff gpg tar xz unzip curl; do command -v $t >/dev/null || echo $t; done'
if ($missing) { Fail "MSYS2 lacks: $($missing -join ' ') (pacman -S --needed make diffutils gnupg tar xz unzip curl)" }

Write-Host "== SDL"
Invoke-Vs (Bash-Lines 'tools/fetch_sdl3.sh')
Write-Host "== FFmpeg ($Arch)"
Invoke-Vs (Bash-Lines "tools/build_ffmpeg.sh $Arch")
Write-Host "== rc2k.exe ($triple)"
# The repository's cargo config names LLVM's linker for 32 bits (the Linux cross build);
# here Visual Studio's own is used.
Invoke-Vs @('set CARGO_TARGET_I686_PC_WINDOWS_MSVC_LINKER=link.exe',
    "rustup target add $triple || exit /b 1",
    "cargo build --release --target $triple --bin rc2k || exit /b 1")

$out = Join-Path $Repo "target\$triple\release"
$sdl = Join-Path $Repo "third_party\sdl3\lib\$Arch\SDL3.dll"
$ffmpeg = Join-Path $Repo $(if ($Arch -eq 'x64') { 'third_party\ffmpeg-x64\bin' } else { 'third_party\ffmpeg\bin' })
$files = @((Join-Path $out 'rc2k.exe'), $sdl, (Join-Path $ffmpeg 'avcodec-61.dll'), (Join-Path $ffmpeg 'avutil-59.dll'))
foreach ($f in $files) { if (-not (Test-Path $f)) { Fail "missing after the build: $f" } }

if ($GameDir) {
    if (-not (Test-Path (Join-Path $GameDir 'files'))) { Fail "$GameDir has no files\ folder: not the game's folder" }
    foreach ($f in $files) { Copy-Item -Force $f $GameDir }
    Write-Host "copied rc2k.exe, SDL3.dll and FFmpeg's two DLLs to $GameDir"
} else {
    Write-Host "built: $out\rc2k.exe (its DLLs beside it)"
}
