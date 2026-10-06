<#
.SYNOPSIS
  Install DocAnvil on Windows.

.EXAMPLE
  irm https://github.com/docanvil/docanvil/releases/latest/download/install.ps1 | iex

.EXAMPLE
  & ([scriptblock]::Create((irm https://github.com/docanvil/docanvil/releases/latest/download/install.ps1))) -Version 1.2.0
#>
param(
    [string]$Version = $env:DOCANVIL_VERSION,
    [string]$InstallDir = $(if ($env:DOCANVIL_INSTALL_DIR) { $env:DOCANVIL_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'docanvil\bin' }),
    [switch]$Force,
    [switch]$Quiet
)

# Everything runs in a child scope: under `irm | iex` the script shares the
# caller's session, and we mustn't leave our preferences or variables behind.
# Errors are thrown, never `exit`ed, because exit would close the user's shell.
& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'   # Invoke-WebRequest is very slow with the progress bar
    if ($PSVersionTable.PSVersion.Major -lt 6) {
        # Windows PowerShell 5.1 may default to TLS 1.0, which GitHub rejects.
        # (Don't do this on PowerShell 7: it restricts .NET to TLS 1.2 only.)
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    }

    $Repo = if ($env:DOCANVIL_REPO_URL) { $env:DOCANVIL_REPO_URL } else { 'https://github.com/docanvil/docanvil' }
    $Api = if ($env:DOCANVIL_API_URL) { $env:DOCANVIL_API_URL } else { 'https://api.github.com/repos/docanvil/docanvil' }

    function Say([string]$Message) { if (-not $Quiet) { Write-Host $Message } }

    function Get-LatestVersion {
        # releases/latest redirects to .../releases/tag/vX.Y.Z; read where we landed.
        $resp = Invoke-WebRequest -UseBasicParsing -Method Head -Uri "$Repo/releases/latest"
        $final = if ($resp.BaseResponse.ResponseUri) {
            $resp.BaseResponse.ResponseUri             # Windows PowerShell 5.1
        } else {
            $resp.BaseResponse.RequestMessage.RequestUri   # PowerShell 7+
        }
        $tag = ("$final".TrimEnd('/') -split '/')[-1]
        if ($tag -notmatch '^v?\d+\.\d+\.\d+') { throw "couldn't work out the latest DocAnvil version from $Repo/releases/latest" }
        return $tag.TrimStart('v')
    }

    function Get-ExpectedSha([string]$Base, [string]$Asset, [string]$Tmp) {
        $sums = Join-Path $Tmp 'SHA256SUMS'
        try {
            Invoke-WebRequest -UseBasicParsing -Uri "$Base/SHA256SUMS" -OutFile $sums
        } catch {
            # Releases before SHA256SUMS existed: use GitHub's per-asset digest.
            try {
                $release = Invoke-RestMethod -Uri "$Api/releases/tags/v$Version" -Headers @{ 'User-Agent' = 'docanvil-installer' }
            } catch {
                return $null
            }
            $match = $release.assets | Where-Object { $_.name -eq $Asset } | Select-Object -First 1
            if ($match -and $match.digest) { return $match.digest.Replace('sha256:', '').ToLower() }
            return $null
        }
        foreach ($line in Get-Content $sums) {
            $parts = $line -split '\s+', 2
            if ($parts.Count -eq 2 -and $parts[1].TrimStart('*') -eq $Asset) { return $parts[0].ToLower() }
        }
        return $null
    }

    if ($env:PROCESSOR_ARCHITECTURE -notin @('AMD64', 'ARM64')) {
        throw "no prebuilt DocAnvil for '$($env:PROCESSOR_ARCHITECTURE)'. Install from source with: cargo install docanvil"
    }
    # ARM64 Windows runs the x86_64 build under emulation.
    $Target = 'x86_64-pc-windows-msvc'

    if ($Version) { $Version = $Version.TrimStart('v') } else { $Version = Get-LatestVersion }
    if ($Version -notmatch '^\d+\.\d+\.\d+') { throw "'$Version' isn't a valid DocAnvil version" }

    $Bin = Join-Path $InstallDir 'docanvil.exe'
    if (-not $Force) {
        $existing = if (Test-Path $Bin) { $Bin } else { (Get-Command docanvil -ErrorAction SilentlyContinue).Source }
        if ($existing) {
            $have = try { ((& $existing --version) -split ' ')[1] } catch { $null }
            if ($have -eq $Version) {
                Say "docanvil v$Version is already installed at $existing"
                return
            }
        }
    }

    $Asset = "docanvil-v$Version-$Target.zip"
    $Base = "$Repo/releases/download/v$Version"
    $Tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("docanvil-" + [System.Guid]::NewGuid())
    New-Item -ItemType Directory -Path $Tmp | Out-Null
    try {
        Say "Downloading docanvil v$Version ($Target)..."
        $zip = Join-Path $Tmp $Asset
        try {
            Invoke-WebRequest -UseBasicParsing -Uri "$Base/$Asset" -OutFile $zip
        } catch {
            throw "download failed: $Base/$Asset (does v$Version exist? see $Repo/releases)"
        }

        $expected = Get-ExpectedSha $Base $Asset $Tmp
        if (-not $expected) { throw "couldn't find a published checksum for $Asset; refusing to install an unverified binary" }
        $actual = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower()
        if ($actual -ne $expected) { throw "checksum mismatch for $Asset (expected $expected, got $actual)" }

        Expand-Archive -Path $zip -DestinationPath $Tmp -Force
        $extracted = Join-Path $Tmp 'docanvil.exe'
        if (-not (Test-Path $extracted)) { throw "the archive didn't contain docanvil.exe" }

        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Copy-Item $extracted $Bin -Force
    } finally {
        Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
    }

    Say "Installed docanvil v$Version to $Bin"

    if ($IsWindows -or $PSVersionTable.PSVersion.Major -lt 6) {
        # Read and write the raw registry value so entries like %USERPROFILE%\bin
        # stay unexpanded and the value keeps its REG_EXPAND_SZ type.
        $envKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
        try {
            $userPath = [string]$envKey.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
            if (-not (($userPath -split ';') -contains $InstallDir)) {
                $newPath = if ($userPath) { "$($userPath.TrimEnd(';'));$InstallDir" } else { $InstallDir }
                $envKey.SetValue('Path', $newPath, [Microsoft.Win32.RegistryValueKind]::ExpandString)
                # Setting any user variable broadcasts WM_SETTINGCHANGE so new terminals see the change.
                [Environment]::SetEnvironmentVariable('DOCANVIL_INSTALLER', '1', 'User')
                [Environment]::SetEnvironmentVariable('DOCANVIL_INSTALLER', $null, 'User')
                $env:Path = "$env:Path;$InstallDir"
                Say "Added $InstallDir to your user PATH. Open a new terminal to use docanvil everywhere."
            }
        } finally {
            $envKey.Close()
        }
    }
}
