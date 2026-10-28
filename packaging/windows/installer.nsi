; Instalador por usuario. El payload Linux se ejecuta con WSL, no como PE nativo.
Unicode true
!include "MUI2.nsh"
Name "PyForge ST (WSL)"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\PyForge"
InstallDirRegKey HKCU "Software\PyForge" "InstallDir"
RequestExecutionLevel user
ShowInstDetails show
; El icono se genera desde website/logo.svg en cada construcción.
!define MUI_ICON "${PAYLOAD}/st.ico"
!define MUI_UNICON "${PAYLOAD}/st.ico"
!define MUI_WELCOMEPAGE_TEXT "Instala PyForge y ST IDE para el usuario actual.$\r$\n$\r$\nRequiere Windows x64 y WSL configurado con una distribución Linux x86-64, glibc ${GLIBC} o superior, Python 3.11 y binutils.$\r$\n$\r$\nLos programas generados son ejecutables Linux. No son aplicaciones Windows nativas."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_TEXT "Instalación terminada. Abre una terminal nueva para usar pyforge, pyforgec y pyforge-ide.$\r$\n$\r$\nSi falta WSL, consulta INSTALACION.md en la carpeta instalada. No se instala ni modifica automáticamente tu distribución Linux."
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "Spanish"

Section "Instalar"
    ; Una actualización retira recursos antiguos. No reclama una carpeta payload ajena.
    IfFileExists "$INSTDIR\payload\*.*" 0 copy_payload
    IfFileExists "$INSTDIR\payload\usr\share\pyforge\build.json" owned_payload
    MessageBox MB_ICONSTOP "Ya existe una carpeta payload ajena a PyForge. Elige otra carpeta."
    Abort
owned_payload:
    RMDir /r "$INSTDIR\payload"
copy_payload:
    ; Los archivos vienen de la misma instantánea que el .deb.
    SetOutPath "$INSTDIR"
    File /r "${PAYLOAD}/*"
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    WriteRegStr HKCU "Software\PyForge" "InstallDir" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PyForge" "DisplayName" "PyForge ST (WSL)"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PyForge" "DisplayVersion" "${VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PyForge" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    ; PowerShell preserva PATH largos, y registra si añadió nuestra entrada.
    ExecWait 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$INSTDIR\environment.ps1" -Mode install -Bin "$INSTDIR\bin"' $0
    IntCmp $0 0 path_ok
    MessageBox MB_ICONSTOP "No se pudo actualizar PATH. La instalación se puede retirar desde Aplicaciones."
    Abort
path_ok:
    ; No fuerza UserChoice: si ya hay otro editor predeterminado, lo conserva.
    ExecWait 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$INSTDIR\associations.ps1" -Mode install -InstallDir "$INSTDIR"' $0
    IntCmp $0 0 association_ok
    MessageBox MB_ICONSTOP "No se pudo registrar el icono de ST. Reinstala para reintentarlo."
    Abort
association_ok:
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

Section "Uninstall"
    ExecWait 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$INSTDIR\environment.ps1" -Mode remove -Bin "$INSTDIR\bin"' $0
    IntCmp $0 0 remove_ok
    MessageBox MB_ICONSTOP "No se pudo retirar la entrada PATH; se conservan los archivos para reintentar."
    Abort
remove_ok:
    ExecWait 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$INSTDIR\associations.ps1" -Mode remove -InstallDir "$INSTDIR"' $0
    IntCmp $0 0 association_removed
    MessageBox MB_ICONSTOP "No se pudo retirar la asociación de ST; se conservan los recursos para reintentar."
    Abort
association_removed:
    ; Solo retira archivos que instaló el paquete; conserva documentos ajenos.
    RMDir /r "$INSTDIR\payload"
    Delete "$INSTDIR\bin\pyforge.cmd"
    Delete "$INSTDIR\bin\pyforgec.cmd"
    Delete "$INSTDIR\bin\pyforge-ide.cmd"
    RMDir "$INSTDIR\bin"
    Delete "$INSTDIR\launch.py"
    Delete "$INSTDIR\launch.ps1"
    Delete "$INSTDIR\environment.ps1"
    Delete "$INSTDIR\associations.ps1"
    Delete "$INSTDIR\st.ico"
    Delete "$INSTDIR\INSTALACION.md"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\PyForge"
    DeleteRegKey HKCU "Software\PyForge"
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd
