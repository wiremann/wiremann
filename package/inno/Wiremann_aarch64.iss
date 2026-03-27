[Setup]
AppName=Wiremann
AppPublisher=Wiremann
AppPublisherURL=https://github.com/wiremann/wiremann
AppVersion=0.1.0
WizardStyle=modern
DefaultDirName={autopf}\Wiremann
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\wiremann.exe
Compression=lzma2
SolidCompression=yes
OutputDir=Output
OutputBaseFilename=WiremannSetup_aarch64
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
WizardSmallImageFile=SmallImage.png
WizardImageFile=LargeImage.png
LicenseFile=..\..\LICENSE
DisableWelcomePage=no

[Files]
Source: "..\..\target\aarch64-pc-windows-msvc\release\wiremann.exe"; DestDir: "{app}"

[Icons]
Name: "{autoprograms}\Wiremann"; Filename: "{app}\wiremann.exe"
Name: "{autodesktop}\Wiremann"; Filename: "{app}\wiremann.exe"

[Run]
Filename: "{app}\wiremann.exe"; Description: "Launch Wiremann"; Flags: postinstall nowait skipifsilent