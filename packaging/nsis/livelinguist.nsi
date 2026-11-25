; Live Linguist - Windows 11 x64 per-user installer (no admin).
; Built by scripts/build-windows.ps1, which passes:
;   /DVERSION=1.2.3[-rc.1] /DNUMVERSION=1.2.3.0 /DSRCDIR=<dist\windows> /DOUTFILE=<setup.exe>
; Defaults below let `makensis livelinguist.nsi` run by hand from this folder.
;
; Foolproof by design: no choices during install, no admin prompt, the app
; starts when setup finishes, upgrades close a running copy first, and
; uninstall keeps the 1.1 GB of downloaded models unless the user ticks a box.

!ifndef VERSION
  !define VERSION "0.0.0-dev"
!endif
!ifndef NUMVERSION
  !define NUMVERSION "0.0.0.0"
!endif
!ifndef SRCDIR
  !define SRCDIR "..\..\dist\windows"
!endif
!ifndef OUTFILE
  !define OUTFILE "LiveLinguist-${VERSION}-setup.exe"
!endif

!define APPNAME "Live Linguist"
!define PUBLISHER "Live Linguist contributors"
!define EXE "livelinguist.exe"
!define URL "https://github.com/ngoldbla/livelinguist-pc"
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\LiveLinguist"
!define APPKEY "Software\LiveLinguist"
!define MODELDIR "$LOCALAPPDATA\LiveLinguist\models"

Unicode true
SetCompressor /SOLID lzma
RequestExecutionLevel user
ManifestDPIAware true

Name "${APPNAME}"
Caption "${APPNAME} ${VERSION} Setup"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\LiveLinguist"
InstallDirRegKey HKCU "${APPKEY}" "InstallDir"
BrandingText "${APPNAME} ${VERSION}"

VIProductVersion "${NUMVERSION}"
VIFileVersion "${NUMVERSION}"
VIAddVersionKey "ProductName" "${APPNAME}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "${APPNAME} Setup"
VIAddVersionKey "CompanyName" "${PUBLISHER}"
VIAddVersionKey "LegalCopyright" "MIT License"

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "WinVer.nsh"
!include "FileFunc.nsh"

!define MUI_ICON "..\icons\livelinguist.ico"
!define MUI_UNICON "..\icons\livelinguist.ico"
!define MUI_ABORTWARNING

!define MUI_WELCOMEPAGE_TEXT "This will install ${APPNAME} ${VERSION} for your Windows account. You do not need administrator rights.$\r$\n$\r$\nThe first time it starts, ${APPNAME} downloads its speech and simplification models (about 1.1 GB). After that it works offline, and audio never leaves this PC.$\r$\n$\r$\nClick Install to continue."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\${EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Start ${APPNAME} now"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!define MUI_COMPONENTSPAGE_TEXT_TOP "The downloaded models are kept by default, so installing again does not download them a second time."
!insertmacro MUI_UNPAGE_COMPONENTS
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; --- checks ---------------------------------------------------------------

Function .onInit
  ${If} ${IsNativeARM64}
  ${OrIfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "${APPNAME} needs a 64-bit Intel or AMD PC. It does not run on this computer." /SD IDOK
    Abort
  ${EndIf}
  ${IfNot} ${AtLeastBuild} 22000
    MessageBox MB_OK|MB_ICONSTOP "${APPNAME} needs Windows 11. This PC runs an older version of Windows." /SD IDOK
    Abort
  ${EndIf}
FunctionEnd

; A running copy locks livelinguist.exe and the .node files; ask the user to
; close it (silent installs close it themselves).
!macro CLOSE_RUNNING_APP
  ${If} ${FileExists} "$INSTDIR\${EXE}"
    retry_close:
    ClearErrors
    FileOpen $0 "$INSTDIR\${EXE}" a
    ${If} ${Errors}
      ${If} ${Silent}
        nsExec::Exec 'taskkill /IM "${EXE}" /F'
        Pop $0
        Sleep 1000
      ${Else}
        MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "${APPNAME} is running. Close it, then click Retry." IDRETRY retry_close
        Abort
      ${EndIf}
    ${Else}
      FileClose $0
    ${EndIf}
  ${EndIf}
!macroend

; Only files this installer owns are removed, never the whole folder, so a
; custom /D= folder with other content is safe.
!macro REMOVE_PROGRAM_FILES
  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\livelinguist-core.node"
  Delete "$INSTDIR\gpuix-native.win32-x64-msvc.node"
  RMDir /r "$INSTDIR\licenses"
!macroend

; --- install ----------------------------------------------------------------

Section "Install"
  SetShellVarContext current
  !insertmacro CLOSE_RUNNING_APP
  !insertmacro REMOVE_PROGRAM_FILES

  SetOutPath "$INSTDIR"
  File /r "${SRCDIR}\*.*"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateShortCut "$SMPROGRAMS\${APPNAME}.lnk" "$INSTDIR\${EXE}" "" "$INSTDIR\${EXE}" 0
  CreateShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\${EXE}" "" "$INSTDIR\${EXE}" 0

  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegStr HKCU "${APPKEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTKEY}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayIcon" "$INSTDIR\${EXE}"
  WriteRegStr HKCU "${UNINSTKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "URLInfoAbout" "${URL}"
  WriteRegStr HKCU "${UNINSTKEY}" "HelpLink" "${URL}/issues"
  WriteRegStr HKCU "${UNINSTKEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNINSTKEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINSTKEY}" "EstimatedSize" "$0"
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoRepair" 1
SectionEnd

; --- uninstall --------------------------------------------------------------

Section "-un.Program files"
  SetShellVarContext current
  !insertmacro CLOSE_RUNNING_APP
  Delete "$DESKTOP\${APPNAME}.lnk"
  Delete "$SMPROGRAMS\${APPNAME}.lnk"
  !insertmacro REMOVE_PROGRAM_FILES
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNINSTKEY}"
  DeleteRegKey HKCU "${APPKEY}"
SectionEnd

Section /o "un.Also delete downloaded language models (about 1.1 GB)" SecModels
  RMDir /r "${MODELDIR}"
  RMDir "$LOCALAPPDATA\LiveLinguist"
SectionEnd
