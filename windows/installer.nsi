Unicode true
Name "MD Preview"
OutFile "..\dist\MD-Preview-windows-x64-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\MD Preview"
InstallDirRegKey HKCU "Software\MD Preview" "InstallDir"
RequestExecutionLevel user
Icon "..\assets\icon.ico"
UninstallIcon "..\assets\icon.ico"
BrandingText "MD Preview"

!define APP_NAME "MD Preview"
!define APP_EXE "md-preview.exe"
!define APP_PROGID "MDPreview.md"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\MD Preview"
!ifndef VERSION
!define VERSION "0.0.0"
!endif

Page directory
Page instfiles

UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  SetOverwrite on
  IfSilent 0 +2
    Sleep 1200

  File /oname=${APP_EXE} "..\target\release\md-preview.exe"
  File "..\dist\WinSparkle.dll"

  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\MD Preview" "InstallDir" "$INSTDIR"

  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"

  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\App Paths\${APP_EXE}" "" "$INSTDIR\${APP_EXE}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\App Paths\${APP_EXE}" "Path" "$INSTDIR"

  WriteRegStr HKCU "Software\Classes\${APP_PROGID}" "" "Markdown Document"
  WriteRegStr HKCU "Software\Classes\${APP_PROGID}" "FriendlyTypeName" "Markdown Document"
  WriteRegStr HKCU "Software\Classes\${APP_PROGID}\DefaultIcon" "" "$INSTDIR\${APP_EXE},0"
  WriteRegStr HKCU "Software\Classes\${APP_PROGID}\shell\open\command" "" '"$INSTDIR\${APP_EXE}" "%1"'

  WriteRegStr HKCU "Software\Classes\.md\OpenWithProgids" "${APP_PROGID}" ""
  WriteRegStr HKCU "Software\Classes\.markdown\OpenWithProgids" "${APP_PROGID}" ""
  WriteRegStr HKCU "Software\Classes\.mdown\OpenWithProgids" "${APP_PROGID}" ""
  WriteRegStr HKCU "Software\Classes\.mkd\OpenWithProgids" "${APP_PROGID}" ""

  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}" "FriendlyAppName" "${APP_NAME}"
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\shell\open\command" "" '"$INSTDIR\${APP_EXE}" "%1"'
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".md" ""
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".markdown" ""
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".mdown" ""
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".mkd" ""

  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE},0"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "vorojar"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "https://vorojar.github.io/md-preview/"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1

  DetailPrint "MD Preview installed."
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"

  Delete "$INSTDIR\WinSparkle.dll"
  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\App Paths\${APP_EXE}"
  DeleteRegKey HKCU "Software\Classes\${APP_PROGID}"
  DeleteRegValue HKCU "Software\Classes\.md\OpenWithProgids" "${APP_PROGID}"
  DeleteRegValue HKCU "Software\Classes\.markdown\OpenWithProgids" "${APP_PROGID}"
  DeleteRegValue HKCU "Software\Classes\.mdown\OpenWithProgids" "${APP_PROGID}"
  DeleteRegValue HKCU "Software\Classes\.mkd\OpenWithProgids" "${APP_PROGID}"
  DeleteRegKey HKCU "Software\Classes\Applications\${APP_EXE}"
SectionEnd
