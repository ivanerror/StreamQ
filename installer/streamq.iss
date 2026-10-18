; Inno Setup script for the StreamQ Windows installer.
; Build after PyInstaller: iscc /DAppVersion=1.1.0 installer\streamq.iss
; Output: dist\installer\StreamQ-Setup-windows-x64.exe

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{6E5B3C1A-6F4D-4C7B-9F3E-2B8D5A1C7E42}
AppName=StreamQ
AppVersion={#AppVersion}
AppPublisher=ivanerror
AppPublisherURL=https://github.com/ivanerror/streamq
AppSupportURL=https://github.com/ivanerror/streamq/issues
DefaultDirName={autopf}\StreamQ
DefaultGroupName=StreamQ
DisableProgramGroupPage=yes
; Per-user install by default, so no administrator rights are needed
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\dist\installer
OutputBaseFilename=StreamQ-Setup-windows-x64
SetupIconFile=..\assets\streamq.ico
UninstallDisplayIcon={app}\StreamQ.exe
LicenseFile=..\LICENSE
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\dist\StreamQ\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\StreamQ"; Filename: "{app}\StreamQ.exe"
Name: "{autodesktop}\StreamQ"; Filename: "{app}\StreamQ.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\StreamQ.exe"; Description: "{cm:LaunchProgram,StreamQ}"; Flags: nowait postinstall skipifsilent
