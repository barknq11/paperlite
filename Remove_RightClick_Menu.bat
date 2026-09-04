@echo off
reg delete "HKCU\Software\Classes\Directory\shell\PaperLite" /f >nul 2>&1
reg delete "HKCU\Software\Classes\Directory\Background\shell\PaperLite" /f >nul 2>&1
echo Right-Click Context Menu removed.
pause
