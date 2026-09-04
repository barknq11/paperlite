@echo off
setlocal
set "EXE_PATH=%~dp0paperlite.exe"
set "DESKTOP=%USERPROFILE%\Desktop"
set "STARTMENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs"

powershell -NoProfile -Command ^
  "$ws = New-Object -ComObject WScript.Shell; " ^
  "$s1 = $ws.CreateShortcut('%DESKTOP%\PaperLite.lnk'); $s1.TargetPath = '%EXE_PATH%'; $s1.WorkingDirectory = '%~dp0'; $s1.Description = 'PaperLite GPU Wallpaper Manager'; $s1.Save(); " ^
  "$s2 = $ws.CreateShortcut('%STARTMENU%\PaperLite.lnk'); $s2.TargetPath = '%EXE_PATH%'; $s2.WorkingDirectory = '%~dp0'; $s2.Description = 'PaperLite GPU Wallpaper Manager'; $s2.Save();"

echo ======================================================
echo Shortcuts created successfully!
echo 1. Desktop: %USERPROFILE%\Desktop\PaperLite.lnk
echo 2. Start Menu: You can now press Win + type 'paperlite'
echo ======================================================
pause
