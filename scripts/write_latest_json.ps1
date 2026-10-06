param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,
    [Parameter(Mandatory = $true)]
    [string]$SignaturePath,
    [Parameter(Mandatory = $true)]
    [string]$DownloadUrl,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath,
    [string]$Notes = ""
)

$signature = (Get-Content -Raw -Path $SignaturePath).Trim()
$manifest = @{
    version  = $Version
    notes    = $Notes
    pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = $signature
            url       = $DownloadUrl
        }
    }
}
$json = $manifest | ConvertTo-Json -Depth 6
$utf8 = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($OutputPath, $json, $utf8)
Write-Output "Записан $OutputPath"
