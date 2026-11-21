@echo off
rem Builds rc2k.exe on Windows: "build" (64 bits), "build windows" (32 bits), and build.ps1's
rem other switches after the target ("build windows-x64 -GameDir C:\Games\Rally").
rem PowerShell runs build.ps1 for this one run whatever the machine's script policy is.
setlocal
set TARGET=%1
if "%TARGET%"=="" set TARGET=windows-x64
if /i "%TARGET:~0,1%"=="-" (set TARGET=windows-x64) else (shift)
set REST=
:collect
if "%~1"=="" goto run
set REST=%REST% %1
shift
goto collect
:run
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1" -Target %TARGET%%REST%
exit /b %ERRORLEVEL%
