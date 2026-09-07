param([string]$MsiPath = 'dist\Specter_0.7.4_x64.msi')
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$msi = if ([IO.Path]::IsPathRooted($MsiPath)) { $MsiPath } else { Join-Path $repo $MsiPath }
if (-not (Test-Path $msi)) { throw 'Missing MSI' }
$installer = New-Object -ComObject WindowsInstaller.Installer
$db = $installer.OpenDatabase($msi, 0)
$view = $db.OpenView('SELECT `FileName`, `FileSize` FROM `File`')
$view.Execute()
$files = @{}
while ($record = $view.Fetch()) {
    $name = ($record.StringData(1) -split '\|')[-1]
    if ($files.ContainsKey($name)) { throw "Duplicate MSI filename: $name" }
    $files[$name] = $record.IntegerData(2)
}
$counts = @{}
foreach ($group in @('workflow-presets','column-presets','corpora-plugins','system-prompts')) {
    $filter = if ($group -eq 'system-prompts') { '*.md' } else { '*.json' }
    $expected = @(Get-ChildItem "$repo\config\$group" -Recurse -File -Filter $filter)
    foreach ($file in $expected) {
        if (-not $files.ContainsKey($file.Name)) { throw "Missing MSI payload: $group/$($file.Name)" }
        if ($files[$file.Name] -ne $file.Length) { throw "MSI size mismatch: $($file.Name)" }
    }
    $counts[$group] = $expected.Count
}
$info = Get-Item $msi
@{ msi=$msi; bytes=$info.Length; sha256=(Get-FileHash $msi -Algorithm SHA256).Hash; modified=$info.LastWriteTime.ToString('o'); payloadCounts=$counts; verified=$true } | ConvertTo-Json -Depth 4
