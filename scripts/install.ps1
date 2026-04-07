# Installs the latest (or pinned) prebuilt specify binary from GitHub Releases.
# Usage: iwr -useb https://raw.githubusercontent.com/Pulko/specify/main/scripts/install.ps1 | iex
#
# Environment:
#   SPECIFY_REPO         GitHub repo root URL (default: https://github.com/Pulko/specify)
#   SPECIFY_VERSION      Pin version: "0.1.1" or "v0.1.1" (default: latest release)
#   SPECIFY_INSTALL_DIR  Directory for specify.exe (default: %USERPROFILE%\.cargo\bin, else %USERPROFILE%\.local\bin)

$OldErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = "Stop"

function Install-Specify {
    $DefaultRepo = "https://github.com/Pulko/specify"
    $Repo = if ($env:SPECIFY_REPO) { $env:SPECIFY_REPO.TrimEnd("/") } else { $DefaultRepo }

    if ($env:SPECIFY_VERSION) {
        $v = $env:SPECIFY_VERSION.Trim()
        if ($v.StartsWith("v")) {
            $Tag = $v
            $Ver = $v.Substring(1)
        }
        else {
            $Tag = "v$v"
            $Ver = $v
        }
    }
    else {
        $release = Invoke-RestMethod -Uri "$Repo/releases/latest" -Headers @{ Accept = "application/vnd.github+json" }
        $Tag = $release.tag_name
        $Ver = $Tag.TrimStart("v")
    }

    $procArch = [System.Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE", "Process")
    if ($procArch -ne "AMD64") {
        throw "Unsupported architecture ($procArch). Install Rust and run: cargo install --git ${Repo}.git"
    }

    $Triple = "x86_64-pc-windows-msvc"
    $Stem = "specify-v$Ver-$Triple"
    $Zip = "$Stem.zip"

    $specifyCmd = Get-Command specify.exe -CommandType Application -ErrorAction SilentlyContinue
    if ($specifyCmd) {
        $verLine = & $specifyCmd.Source -V 2>$null
        if ($verLine -match "specify\s+(\S+)") {
            $current = $Matches[1]
            if ($current -eq $Ver) {
                Write-Host "specify is already at $Tag."
                return
            }
            Write-Host "Updating specify from $current to $Tag ..."
        }
    }

    if ($env:SPECIFY_INSTALL_DIR) {
        $installDir = $env:SPECIFY_INSTALL_DIR
    }
    elseif ($env:CARGO_HOME) {
        $installDir = Join-Path $env:CARGO_HOME "bin"
    }
    elseif (Test-Path (Join-Path $HOME ".cargo\bin")) {
        $installDir = Join-Path $HOME ".cargo\bin"
    }
    else {
        $installDir = Join-Path $HOME ".local\bin"
    }

    New-Item -ItemType Directory -Force -Path $installDir | Out-Null

    $tmp = Join-Path $env:TEMP ("specify-install-" + [Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        $zipPath = Join-Path $tmp $Zip
        $hashPath = Join-Path $tmp "$Zip.sha256"
        $zipUrl = "$Repo/releases/download/$Tag/$Zip"
        Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath
        Invoke-WebRequest -Uri "$zipUrl.sha256" -OutFile $hashPath

        $hashLine = (Get-Content -LiteralPath $hashPath -Raw).Trim()
        $expected = ($hashLine -split "\s+")[0].ToLowerInvariant()
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $zipPath).Hash.ToLowerInvariant()
        if ($expected -ne $actual) {
            throw "SHA256 mismatch (file may be corrupted)"
        }

        Expand-Archive -LiteralPath $zipPath -DestinationPath $tmp -Force
        $exeSrc = Join-Path (Join-Path $tmp $Stem) "specify.exe"
        if (-not (Test-Path -LiteralPath $exeSrc)) {
            throw "expected binary not found in archive: $Stem\specify.exe"
        }
        $exeDest = Join-Path $installDir "specify.exe"
        Copy-Item -LiteralPath $exeSrc -Destination $exeDest -Force

        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        $normInstall = $installDir.TrimEnd("\")
        if ($userPath -notlike "*${normInstall}*") {
            $newPath = if ($userPath) { "$normInstall;$userPath" } else { $normInstall }
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            $env:Path = "$normInstall;$env:Path"
            Write-Host "Added $normInstall to user PATH. Restart the terminal if 'specify' is not found."
        }

        Write-Host "Installed specify $Tag to $exeDest"
    }
    finally {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

try {
    Install-Specify
}
finally {
    $ErrorActionPreference = $OldErrorActionPreference
}
