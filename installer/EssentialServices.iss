#define AppName "Essential Services"
#define AppVersion "0.1.0"
#define AppExeName "essential-services-desktop.exe"

[Setup]
AppId={{BA191153-2F35-452E-803A-40188104C450}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=mariohdel14nasa-lang
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\target\installer
OutputBaseFilename=EssentialServices-Setup
SetupIconFile=..\assets\app-icon.ico
WizardSmallImageFile=..\assets\installer-wizard.bmp
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
UninstallDisplayIcon={app}\{#AppExeName}
CloseApplications=yes
RestartIfNeededByRun=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[CustomMessages]
english.LaunchApp=Launch Essential Services
spanish.LaunchApp=Iniciar Essential Services
english.CreateDesktopIcon=Create a desktop shortcut
spanish.CreateDesktopIcon=Crear un acceso directo en el escritorio

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"; IconFilename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; IconFilename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Parameters: "--language={language}"; Description: "{cm:LaunchApp}"; Flags: nowait postinstall skipifsilent
