$ErrorActionPreference = "Stop"

$manifest_path = Join-Path $PSScriptRoot "..\src-tauri\tauri.conf.json"
$config = Get-Content -Raw -Path $manifest_path | ConvertFrom-Json
$version = [string]$config.version
$tag = $env:CI_COMMIT_TAG
if ($tag -ne "v$version") {
    throw "Тег $tag не совпадает с версией клиента $version"
}
if (-not $env:TAURI_SIGNING_PRIVATE_KEY) {
    throw "Не задан секрет TAURI_SIGNING_PRIVATE_KEY"
}

npm ci
npm run tauri build

$installer = Get-ChildItem -Path "src-tauri\target\release\bundle\nsis" -Filter "*.exe" | Select-Object -First 1
if (-not $installer) {
    throw "Установщик NSIS не найден"
}
$signature_path = "$($installer.FullName).sig"
if (-not (Test-Path $signature_path)) {
    throw "Подпись обновления не найдена: $signature_path"
}

$token_header = "JOB-TOKEN"
$token = $env:CI_JOB_TOKEN
if ($env:GITLAB_TOKEN) {
    $token_header = "PRIVATE-TOKEN"
    $token = $env:GITLAB_TOKEN
}
$package_root = "$env:CI_API_V4_URL/projects/$env:CI_PROJECT_ID/packages/generic/note-gui"
$installer_url = "$package_root/$version/note-gui-setup.exe"
$manifest_url = "$package_root/latest/latest.json"

function Send-PackageFile {
    param([string]$FilePath, [string]$Url)
    & curl.exe --fail --silent --show-error --header "${token_header}: $token" --upload-file $FilePath $Url
    if ($LASTEXITCODE -ne 0) {
        throw "Не удалось загрузить $FilePath"
    }
}

Send-PackageFile -FilePath $installer.FullName -Url $installer_url
$latest_path = Join-Path $env:TEMP "note-gui-latest.json"
& "$PSScriptRoot\write_latest_json.ps1" -Version $version -InstallerPath $installer.FullName -SignaturePath $signature_path -DownloadUrl $installer_url -OutputPath $latest_path -Notes "Релиз $version"
Send-PackageFile -FilePath $latest_path -Url $manifest_url

$release_body = @{
    name        = "note-gui $version"
    tag_name    = $tag
    description = "Релиз $version"
    assets      = @{
        links = @(
            @{
                name      = "latest.json"
                url       = $manifest_url
                filepath  = "/latest.json"
                link_type = "other"
            },
            @{
                name      = "note-gui-setup.exe"
                url       = $installer_url
                filepath  = "/note-gui-setup.exe"
                link_type = "package"
            }
        )
    }
} | ConvertTo-Json -Depth 6

$release_file = Join-Path $env:TEMP "note-gui-release.json"
$utf8 = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($release_file, $release_body, $utf8)
& curl.exe --fail --silent --show-error --header "${token_header}: $token" --header "Content-Type: application/json" --data-binary "@$release_file" --request POST "$env:CI_API_V4_URL/projects/$env:CI_PROJECT_ID/releases"
if ($LASTEXITCODE -ne 0) {
    throw "Не удалось создать GitLab Release"
}
Write-Output "Релиз $tag опубликован. Клиент забирает $manifest_url или permalink /downloads/latest.json"
