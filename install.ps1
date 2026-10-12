$ErrorActionPreference = 'Stop'

$release = Invoke-RestMethod -Headers @{ 'User-Agent' = 'ViGPU-Installer' } `
    -Uri 'https://api.github.com/repos/snkii/vigpu/releases/latest'
$asset = $release.assets | Where-Object { $_.name -eq 'vigpu-windows-x64.zip' } | Select-Object -First 1
if (-not $asset) {
    throw 'The latest ViGPU release does not include a Windows x64 package yet.'
}

$temporaryDirectory = Join-Path $env:TEMP ('vigpu-' + [guid]::NewGuid().ToString('N'))
$installDirectory = Join-Path $env:LOCALAPPDATA 'Programs\ViGPU'
New-Item -ItemType Directory -Path $temporaryDirectory -Force | Out-Null
try {
    $archive = Join-Path $temporaryDirectory 'vigpu.zip'
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $archive
    Expand-Archive -LiteralPath $archive -DestinationPath $temporaryDirectory -Force
    New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $temporaryDirectory 'vigpu.exe') `
        -Destination (Join-Path $installDirectory 'vigpu.exe') -Force
}
finally {
    Remove-Item -LiteralPath $temporaryDirectory -Recurse -Force -ErrorAction SilentlyContinue
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$pathEntries = @($userPath -split ';' | Where-Object { $_ })
if ($pathEntries -notcontains $installDirectory) {
    $pathEntries += $installDirectory
    [Environment]::SetEnvironmentVariable('Path', ($pathEntries -join ';'), 'User')
    Write-Host 'Added ViGPU to your user PATH. Open a new PowerShell window if this command is not found later.'
}
$env:Path = "$installDirectory;$env:Path"
Write-Host "Installed ViGPU $($release.tag_name) to $installDirectory"
Write-Host 'Run: vigpu'
