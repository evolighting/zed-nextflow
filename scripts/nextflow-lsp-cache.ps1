[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateSet("check", "update", "resolve")]
    [string]$Action = $(
        if ($env:NEXTFLOW_LSP_ACTION) {
            $env:NEXTFLOW_LSP_ACTION
        } else {
            "check"
        }
    ),

    [Parameter(Position = 1)]
    [string]$LanguageVersion = $(
        if ($env:NEXTFLOW_LANGUAGE_VERSION) {
            $env:NEXTFLOW_LANGUAGE_VERSION
        } else {
            "26.04"
        }
    )
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repository = "nextflow-io/language-server"
$AssetName = "language-server-all.jar"

if ($LanguageVersion -notmatch '^[0-9]+\.[0-9]+$') {
    [Console]::Error.WriteLine(
        "Invalid language version: {0} (expected e.g. 26.04)",
        $LanguageVersion
    )
    exit 2
}

$VersionPrefix = "v$LanguageVersion"
if ($env:NEXTFLOW_LSP_CACHE_DIR) {
    $CacheRoot = $env:NEXTFLOW_LSP_CACHE_DIR
} else {
    $UserHome = [Environment]::GetFolderPath(
        [Environment+SpecialFolder]::UserProfile
    )
    if (-not $UserHome) {
        throw "Unable to determine the user home directory."
    }
    $CacheRoot = Join-Path (Join-Path $UserHome ".nextflow") "lsp"
}
$CacheDir = Join-Path $CacheRoot $VersionPrefix
$VersionPattern = '^' + [Regex]::Escape($VersionPrefix) + '\.([0-9]+)\.jar$'

function Get-LatestLocal {
    if (-not (Test-Path -LiteralPath $CacheDir -PathType Container)) {
        return $null
    }

    $BestPath = $null
    $BestPatch = -1
    foreach ($File in Get-ChildItem -LiteralPath $CacheDir -File -ErrorAction SilentlyContinue) {
        $Match = [Regex]::Match($File.Name, $VersionPattern)
        if (-not $Match.Success) {
            continue
        }

        $Patch = [int]$Match.Groups[1].Value
        if ($Patch -gt $BestPatch) {
            $BestPatch = $Patch
            $BestPath = $File.FullName
        }
    }

    return $BestPath
}

function Get-LatestRemote {
    $Headers = @{
        "Accept" = "application/vnd.github+json"
        "User-Agent" = "zed-nextflow"
        "X-GitHub-Api-Version" = "2022-11-28"
    }
    if ($env:GITHUB_TOKEN) {
        $Headers["Authorization"] = "Bearer $($env:GITHUB_TOKEN)"
    }

    try {
        if ($PSVersionTable.PSEdition -eq "Desktop") {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        }
        $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases?per_page=100" -Headers $Headers -Method Get
    } catch {
        return $null
    }

    $TagPattern = '^' + [Regex]::Escape($VersionPrefix) + '\.([0-9]+)$'
    $BestTag = $null
    $BestPatch = -1
    foreach ($Release in @($Releases)) {
        $Tag = [string]$Release.tag_name
        $Match = [Regex]::Match($Tag, $TagPattern)
        if (-not $Match.Success) {
            continue
        }

        $Patch = [int]$Match.Groups[1].Value
        if ($Patch -gt $BestPatch) {
            $BestPatch = $Patch
            $BestTag = $Tag
        }
    }

    return $BestTag
}

function Test-JarHeader([string]$Path) {
    $Stream = [IO.File]::Open(
        $Path,
        [IO.FileMode]::Open,
        [IO.FileAccess]::Read,
        [IO.FileShare]::Read
    )
    try {
        $Header = New-Object byte[] 4
        if ($Stream.Read($Header, 0, 4) -ne 4) {
            return $false
        }
        return (
            $Header[0] -eq 0x50 -and
            $Header[1] -eq 0x4b -and
            (($Header[2] -eq 0x03 -and $Header[3] -eq 0x04) -or
             ($Header[2] -eq 0x05 -and $Header[3] -eq 0x06) -or
             ($Header[2] -eq 0x07 -and $Header[3] -eq 0x08))
        )
    } finally {
        $Stream.Dispose()
    }
}

function Get-VersionJar([string]$Version) {
    $Target = Join-Path $CacheDir "$Version.jar"
    if (Test-Path -LiteralPath $Target -PathType Leaf) {
        return (Get-Item -LiteralPath $Target).FullName
    }

    New-Item -ItemType Directory -Path $CacheDir -Force | Out-Null
    $Temporary = "$Target.part.$PID"
    try {
        $PreviousProgressPreference = $ProgressPreference
        $ProgressPreference = "SilentlyContinue"
        try {
            Invoke-WebRequest -Uri "https://github.com/$Repository/releases/download/$Version/$AssetName" -OutFile $Temporary -UseBasicParsing
        } finally {
            $ProgressPreference = $PreviousProgressPreference
        }

        if (-not (Test-JarHeader $Temporary)) {
            throw "Downloaded file is not a valid JAR: $Temporary"
        }

        Move-Item -LiteralPath $Temporary -Destination $Target -Force
    } catch {
        Remove-Item -LiteralPath $Temporary -Force -ErrorAction SilentlyContinue
        throw
    }

    return (Get-Item -LiteralPath $Target).FullName
}

switch ($Action) {
    "check" {
        $LocalPath = Get-LatestLocal
        $RemoteVersion = Get-LatestRemote
        $LocalVersion = if ($LocalPath) {
            Split-Path -Leaf $LocalPath
        } else {
            ""
        }

        [Console]::Out.WriteLine("Cache directory: {0}", $CacheDir)
        [Console]::Out.WriteLine("Local version:   {0}", $LocalVersion)
        [Console]::Out.WriteLine(
            "Remote version:  {0}",
            $(if ($RemoteVersion) { $RemoteVersion } else { "unavailable" })
        )
        if ($LocalPath -and $LocalVersion -eq "$RemoteVersion.jar") {
            [Console]::Out.WriteLine("Status:          up to date")
        } elseif ($RemoteVersion) {
            [Console]::Out.WriteLine("Status:          update available")
        } elseif ($LocalPath) {
            [Console]::Out.WriteLine("Status:          using local cache (remote check failed)")
        } else {
            [Console]::Out.WriteLine("Status:          no cached language server")
        }
    }
    "update" {
        $RemoteVersion = Get-LatestRemote
        if (-not $RemoteVersion) {
            throw "Unable to determine the latest $VersionPrefix language-server release."
        }
        $Previous = Get-LatestLocal
        $Resolved = Get-VersionJar $RemoteVersion
        if ($Resolved -eq $Previous) {
            [Console]::Out.WriteLine("Already up to date: {0}", $Resolved)
        } else {
            [Console]::Out.WriteLine("Updated Nextflow language server: {0}", $Resolved)
        }
    }
    "resolve" {
        $Resolved = Get-LatestLocal
        if (-not $Resolved) {
            $RemoteVersion = Get-LatestRemote
            if (-not $RemoteVersion) {
                throw "No cached JAR and unable to query GitHub for $VersionPrefix."
            }
            $Resolved = Get-VersionJar $RemoteVersion
        }
        [Console]::Out.WriteLine($Resolved)
    }
}
