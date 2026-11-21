; Live Linguist PC — Windows 11 x64 NSIS installer (no admin).
; Build from the repo root after scripts/build-windows.ps1 has populated
; dist/windows. Output: packaging/nsis/LiveLinguist-0.1.0-setup.exe.

!include "MUI2.nsh"

!define APPNAME "Live Linguist"
!define COMPANY "Live Linguist"
!define VERSION "0.1.0"
!define REGKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\LiveLinguist"

Name "${APPNAME} ${VERSION}"
OutFile "LiveLinguist-${VERSION}-setup.exe"
InstallDir "$LOCALAPPDATA\LiveLinguist"
InstallDirRegKey HKCU "Software\LiveLinguist" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
Unicode true

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath "$INSTDIR"
  File /r "..\..\dist\windows\*.*"

  CreateDirectory "$SMPROGRAMS\Live Linguist"
  CreateShortCut "$SMPROGRAMS\Live Linguist\Live Linguist.lnk" "$INSTDIR\livelinguist.exe"
  CreateShortCut "$SMPROGRAMS\Live Linguist\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  CreateShortCut "$DESKTOP\Live Linguist.lnk" "$INSTDIR\livelinguist.exe"

  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\LiveLinguist" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${REGKEY}" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "${REGKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${REGKEY}" "Publisher" "${COMPANY}"
  WriteRegStr HKCU "${REGKEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegDWORD HKCU "${REGKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${REGKEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\Live Linguist.lnk"
  Delete "$SMPROGRAMS\Live Linguist\Live Linguist.lnk"
  Delete "$SMPROGRAMS\Live Linguist\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Live Linguist"
  Delete "$INSTDIR\livelinguist.exe"
  Delete "$INSTDIR\livelinguist-core.node"
  Delete "$INSTDIR\uninstall.exe"
  RMDir /r "$INSTDIR\assets"
  RMDir /r "$INSTDIR\backends"
  RMDir /r "$INSTDIR\licenses"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${REGKEY}"
  DeleteRegKey HKCU "Software\LiveLinguist"
SectionEnd
