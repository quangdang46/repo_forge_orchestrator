<#
.SYNOPSIS
  rfo installer for Windows (PowerShell 5.1+).

.DESCRIPTION
  Downloads the latest (or pinned) rfo release from GitHub, verifies its
  SHA256, extracts it, and installs `rfo.exe` into a per-user directory.
  Optionally adds the install directory to the user's PATH.

.EXAMPLE
  irm https://raw.githubusercontent.com/quangdang46/repo_forge/main/install.ps1 | iex

.EXAMPLE
  $env:RFO_VERSION = "v0.1.0"; irm https://raw.githubusercontent.com/quangdang46/repo_forge/main/install.ps1 | iex

.NOTES
  Environment overrides:
    RFO_VERSION       Tag to install (e.g. v0.1.0). Default: latest GitHub release.
    RFO_INSTALL_DIR   Where rfo.exe is placed. Default: $env:LOCALAPPDATA\Programs\rfo.
    RFO_NO_VERIFY     If "1", skip SHA256 verification (NOT recommended).
    RFO_NO_MODIFY_PATH If "1", do not add the install directory to user PATH.
#>

[CmdletBinding()]
param()

# When piped through `iex`, $ErrorActionPreference defaults to Continue.
# Make failures hard so we never silently install a broken binary.
$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'  # speeds up Invoke-WebRequest

# ---------- config ----------
$Repo           = 'quangdang46/repo_forge'
$BinName        = 'rfo'
$DefaultDir     = Join-Path $env:LOCALAPPDATA 'Programs\rfo'
$Version        = if ($env:RFO_VERSION)        { $env:RFO_VERSION }        else { 'latest' }
$InstallDir     = if ($env:RFO_INSTALL_DIR)    { $env:RFO_INSTALL_DIR }    else { $DefaultDir }
$NoVerify       = $env:RFO_NO_VERIFY -eq '1'
$NoModifyPath   = $env:RFO_NO_MODIFY_PATH -eq '1'

# ---------- pretty output ----------
function Write-Info  { param([string]$Msg) Write-Host "==> $Msg" -ForegroundColor Cyan }
function Write-Ok    { param([string]$Msg) Write-Host " OK $Msg" -ForegroundColor Green }
function Write-Warn2 { param([string]$Msg) Write-Host "  ! $Msg" -ForegroundColor Yellow }
function Write-Err   { param([string]$Msg) Write-Host "  X $Msg" -ForegroundColor Red }

# ---------- helpers ----------
function Test-PowerShellVersion {
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        throw "rfo installer requires PowerShell 5.1 or later (found $($PSVersionTable.PSVersion))."
    }
}

function Resolve-Target {
    # rfo only ships x86_64-pc-windows-msvc for Windows today.
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        'AMD64' { return 'x86_64-pc-windows-msvc' }
        'ARM64' {
            throw "Windows ARM64 is not yet a published target for rfo. " +
                  "Track https://github.com/$Repo/issues for ARM64 support, or build from source."
        }
        default {
            throw "Unsupported Windows architecture: $arch (expected AMD64)."
        }
    }
}

function Resolve-Tag {
    if ($Version -ne 'latest') {
        if ($Version.StartsWith('v')) { return $Version } else { return "v$Version" }
    }

    $api = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $resp = Invoke-RestMethod -Uri $api -Method Get -UseBasicParsing -Headers @{
            'User-Agent' = 'rfo-installer'
            'Accept'     = 'application/vnd.github+json'
        }
    } catch {
        throw "Could not query GitHub for latest release: $($_.Exception.Message). " +
              "GitHub may be rate-limiting; pin a version with `$env:RFO_VERSION = 'v0.1.0'`."
    }

    if (-not $resp.tag_name) {
        throw "GitHub API returned no tag_name for latest release of $Repo."
    }
    return $resp.tag_name
}

function Invoke-Download {
    param(
        [Parameter(Mandatory)] [string]$Url,
        [Parameter(Mandatory)] [string]$Destination
    )
    try {
        # Force TLS 1.2 (Windows PowerShell 5.1 defaults to TLS 1.0 on older systems).
        [Net.ServicePointManager]::SecurityProtocol =
            [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing `
            -Headers @{ 'User-Agent' = 'rfo-installer' } -ErrorAction Stop
    } catch {
        throw "Download failed: $Url`n  $($_.Exception.Message)"
    }
}

function Add-ToUserPath {
    param([Parameter(Mandatory)] [string]$Dir)

    $current = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $current) { $current = '' }

    $parts = $current.Split(';', [StringSplitOptions]::RemoveEmptyEntries)
    foreach ($p in $parts) {
        if ([string]::Equals($p.TrimEnd('\'), $Dir.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) {
            return $false  # already on PATH
        }
    }

    $new = if ($current) { "$current;$Dir" } else { $Dir }
    [Environment]::SetEnvironmentVariable('Path', $new, 'User')
    # Make this session see it too.
    $env:Path = "$env:Path;$Dir"
    return $true
}

# ---------- main ----------
function Install-Rfo {
    Test-PowerShellVersion

    Write-Info "rfo installer"
    Write-Info "repo:    https://github.com/$Repo"
    Write-Info "user:    $env:USERNAME"

    $target = Resolve-Target
    Write-Info "target:  $target"

    $tag = Resolve-Tag
    Write-Info "version: $tag"

    $archiveName  = "$BinName-$target.zip"
    $archiveUrl   = "https://github.com/$Repo/releases/download/$tag/$archiveName"
    $checksumUrl  = "$archiveUrl.sha256"

    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("rfo-install-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null

    try {
        $archivePath  = Join-Path $tmp $archiveName
        $checksumPath = "$archivePath.sha256"

        Write-Info "downloading $archiveName"
        Invoke-Download -Url $archiveUrl -Destination $archivePath
        $size = (Get-Item $archivePath).Length
        Write-Ok ("downloaded {0:N0} bytes" -f $size)

        if (-not $NoVerify) {
            Write-Info "verifying SHA256"
            try {
                Invoke-Download -Url $checksumUrl -Destination $checksumPath
            } catch {
                throw "Failed to download checksum from $checksumUrl. " +
                      "Set `$env:RFO_NO_VERIFY = '1'` to skip (not recommended)."
            }
            $expected = ((Get-Content $checksumPath -Raw) -split '\s+')[0].Trim().ToLowerInvariant()
            $actual   = (Get-FileHash -Path $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($expected -ne $actual) {
                throw "SHA256 mismatch!`n  expected: $expected`n  actual:   $actual"
            }
            Write-Ok "SHA256 verified"
        } else {
            Write-Warn2 "RFO_NO_VERIFY=1; skipping checksum"
        }

        Write-Info "extracting archive"
        $extractDir = Join-Path $tmp 'extract'
        New-Item -ItemType Directory -Path $extractDir -Force | Out-Null
        Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force

        # cargo-dist layout: <bin>-<target>/<bin>.exe
        $exe = Get-ChildItem -Path $extractDir -Recurse -Filter "$BinName.exe" -File `
            | Select-Object -First 1
        if (-not $exe) {
            throw "Could not locate $BinName.exe inside $archiveName"
        }

        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        $dest = Join-Path $InstallDir "$BinName.exe"
        Copy-Item -Path $exe.FullName -Destination $dest -Force
        Write-Ok "installed: $dest"

        # Sanity check
        try {
            $verLine = & $dest --version 2>$null | Select-Object -First 1
            if ($verLine) { Write-Ok $verLine }
        } catch {
            Write-Warn2 "installed binary did not respond to --version (may still work)"
        }

        # PATH handling
        if ($NoModifyPath) {
            Write-Warn2 "RFO_NO_MODIFY_PATH=1; not modifying PATH"
            Write-Warn2 "add manually: $InstallDir"
        } else {
            $added = Add-ToUserPath -Dir $InstallDir
            if ($added) {
                Write-Ok  "added $InstallDir to user PATH"
                Write-Warn2 "open a new terminal for PATH changes to take effect"
            } else {
                Write-Ok "$InstallDir already on PATH"
            }
        }

        Write-Ok "done. run: $BinName --help"
    }
    finally {
        if (Test-Path $tmp) {
            Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

Install-Rfo
