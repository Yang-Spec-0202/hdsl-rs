Unicode true
!include "MUI2.nsh"

!ifndef SOURCE_DIR
  !error "SOURCE_DIR is required"
!endif
!ifndef OUTPUT_FILE
  !error "OUTPUT_FILE is required"
!endif

Name "HDSL"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\HDSL"
RequestExecutionLevel user
SetCompressor /SOLID lzma

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

Section "HDSL" SecMain
  SetOutPath "$INSTDIR"
  File /r "${SOURCE_DIR}\*"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\HDSL"
  CreateShortCut "$SMPROGRAMS\HDSL\HDSL.lnk" "$INSTDIR\hdsl.exe"
  CreateShortCut "$SMPROGRAMS\HDSL\卸载 HDSL.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\HDSL\HDSL.lnk"
  Delete "$SMPROGRAMS\HDSL\卸载 HDSL.lnk"
  RMDir "$SMPROGRAMS\HDSL"
  RMDir /r "$INSTDIR\help"
  Delete "$INSTDIR\hdsl.exe"
  Delete "$INSTDIR\hdsl-mark.svg"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\NOTICE"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
