@echo off
setlocal
set WDKBIN=e:\Program Files\Windows Kits\10\Tools\10.0.26100.0\x64

"%WDKBIN%\devgen.exe" /add /hardwareid root\NutHidDevice /wait
