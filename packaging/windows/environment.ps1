# Administra solo la entrada PATH de esta instalación y conserva las demás.
# El registro evita el límite de longitud de cadenas de NSIS y no expande %VAR%.
param([ValidateSet('install','remove')][string]$Mode, [string]$Bin)
$ErrorActionPreference = 'Stop'
try {
    $environment = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
    $settings = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Software\PyForge')
    $value = [string]$environment.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    $parts = @($value -split ';')
    $found = @($parts | Where-Object { $_.Trim().TrimEnd('\') -ieq $Bin.TrimEnd('\') }).Count -gt 0
    if ($Mode -eq 'install' -and -not $found) {
        $separator = if ($value -and -not $value.EndsWith(';')) { ';' } else { '' }
        $environment.SetValue('Path', "$value$separator$Bin", [Microsoft.Win32.RegistryValueKind]::ExpandString)
        $settings.SetValue('PathAdded', 1)
    }
    if ($Mode -eq 'remove' -and $settings.GetValue('PathAdded', 0) -eq 1) {
        $remaining = @($parts | Where-Object { $_.Trim().TrimEnd('\') -ine $Bin.TrimEnd('\') }) -join ';'
        $environment.SetValue('Path', $remaining, [Microsoft.Win32.RegistryValueKind]::ExpandString)
        $settings.DeleteValue('PathAdded', $false)
    }
    $environment.Close()
    $settings.Close()
    exit 0
} catch {
    Write-Error $_
    exit 1
}
