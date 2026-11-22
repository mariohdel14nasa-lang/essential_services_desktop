$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (-not (Test-Path -LiteralPath $cargo)) {
    $cargo = (Get-Command cargo.exe -ErrorAction Stop).Source
}

Push-Location $repositoryRoot
try {
    & $cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed with exit code $LASTEXITCODE."
    }

    $compilerCandidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )
    $compiler = $compilerCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $compiler) {
        $compilerCommand = Get-Command ISCC.exe -ErrorAction SilentlyContinue
        if ($compilerCommand) {
            $compiler = $compilerCommand.Source
        }
    }
    if (-not $compiler) {
        throw "Inno Setup 6 was not found. Install Inno Setup 6, then run this script again."
    }

    New-Item -ItemType Directory -Path (Join-Path $repositoryRoot "target\installer") -Force | Out-Null
    & $compiler (Join-Path $PSScriptRoot "EssentialServices.iss")
    if ($LASTEXITCODE -ne 0) {
        throw "Inno Setup compilation failed with exit code $LASTEXITCODE."
    }

    Write-Output "Installer created at: $(Join-Path $repositoryRoot 'target\installer\EssentialServices-Setup.exe')"
}
finally {
    Pop-Location
}
