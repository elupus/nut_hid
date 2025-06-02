@echo off
setlocal
set ROOT=%~dp0

pnputil.exe /delete-driver oem0.inf
pnputil.exe /delete-driver oem1.inf
pnputil.exe /add-driver %ROOT%/nut_hid.inf /install