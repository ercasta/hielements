@echo off
chcp 65001 >nul
set SCRIPT_DIR=%~dp0scripts

powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%\build-windows.ps1" %*
if errorlevel 1 exit /b %errorlevel%

echo Build script finished.
