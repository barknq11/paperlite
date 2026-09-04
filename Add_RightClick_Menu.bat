@echo off
setlocal
set "EXE=%~dp0paperlite.exe"
set "ESCAPED=%EXE:\=\\%"

reg add "HKCU\Software\Classes\Directory\shell\PaperLite" /ve /t REG_SZ /d "Browse with PaperLite" /f
reg add "HKCU\Software\Classes\Directory\shell\PaperLite" /v "Icon" /t REG_SZ /d "\"%EXE%\"" /f
reg add "HKCU\Software\Classes\Directory\shell\PaperLite\command" /ve /t REG_SZ /d "\"%EXE%\" \"%%1\"" /f

reg add "HKCU\Software\Classes\Directory\Background\shell\PaperLite" /ve /t REG_SZ /d "Browse with PaperLite" /f
reg add "HKCU\Software\Classes\Directory\Background\shell\PaperLite" /v "Icon" /t REG_SZ /d "\"%EXE%\"" /f
reg add "HKCU\Software\Classes\Directory\Background\shell\PaperLite\command" /ve /t REG_SZ /d "\"%EXE%\" \"%%V\"" /f

echo ======================================================
echo Right-Click Context Menu added!
echo You can now right-click ANY folder or background in Explorer
echo and click 'Browse with PaperLite'.
echo ======================================================
pause
