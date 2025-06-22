
@echo off
setlocal
set ROOT=%~dp0


set SOURCE_PATH=%1
set TARGET_HOST=%2
set TARGET_PATH=%3

scp -r %SOURCE_PATH%\nut_hid_driver_package\* %SOURCE_PATH%\nut_hid_cli.exe %TARGET_HOST%:%TARGET_PATH%\
scp %ROOT%\install.bat %ROOT%\add.bat %TARGET_HOST%:%TARGET_PATH%\

echo "Finished deploying files"