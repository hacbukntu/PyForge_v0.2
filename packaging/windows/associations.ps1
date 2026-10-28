# Registra ST solo para el usuario actual. No toca UserChoice ni asociaciones ajenas.
param(
    [Parameter(Mandatory=$true)][ValidateSet('install','remove')][string]$Mode,
    [Parameter(Mandatory=$true)][string]$InstallDir
)
$ErrorActionPreference = 'Stop'
$programId = 'PyForge.ST.Source'
$classes = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Software\Classes')
$state = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Software\PyForge\FileAssociation')
try {
    if ($Mode -eq 'install') {
        $icon = Join-Path $InstallDir 'st.ico'
        $launcher = Join-Path $InstallDir 'launch.ps1'
        if (-not (Test-Path -LiteralPath $icon -PathType Leaf) -or
            -not (Test-Path -LiteralPath $launcher -PathType Leaf)) {
            throw 'Falta el icono o el lanzador instalado de PyForge.'
        }
        $program = $classes.CreateSubKey($programId)
        try {
            $program.SetValue('', 'Código fuente ST', [Microsoft.Win32.RegistryValueKind]::String)
            $key = $program.CreateSubKey('DefaultIcon')
            try { $key.SetValue('', ('"{0}",0' -f $icon)) } finally { $key.Close() }
            $key = $program.CreateSubKey('shell\open\command')
            # -File trata la ruta como argumento; no hay cmd /c ni Invoke-Expression.
            try { $key.SetValue('', ('powershell.exe -NoProfile -ExecutionPolicy Bypass -File "{0}" pyforge-ide --open-browser --port 0 "%1"' -f $launcher)) }
            finally { $key.Close() }
        } finally { $program.Close() }
        $extension = $classes.CreateSubKey('.st')
        try {
            $current = $extension.GetValue('', $null)
            # HKCR también incluye asociaciones instaladas para todos los usuarios.
            $effective = [Microsoft.Win32.Registry]::ClassesRoot.OpenSubKey('.st')
            $existing = ''
            if ($null -ne $effective) {
                try { $existing = $effective.GetValue('', '') } finally { $effective.Close() }
            }
            $choice = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.st\UserChoice')
            $selected = ''
            if ($null -ne $choice) {
                try { $selected = $choice.GetValue('ProgId', '') } finally { $choice.Close() }
            }
            if ([string]::IsNullOrEmpty($current) -and [string]::IsNullOrEmpty($existing) -and [string]::IsNullOrEmpty($selected)) {
                # Solo reclama una extensión libre; recuerda si había un valor vacío.
                if ($null -eq $state.GetValue('Claimed', $null)) {
                    $state.SetValue('HadDefault', [int]($null -ne $current), [Microsoft.Win32.RegistryValueKind]::DWord)
                }
                $state.SetValue('Claimed', 1, [Microsoft.Win32.RegistryValueKind]::DWord)
                $extension.SetValue('', $programId)
            }
            $openWith = $extension.CreateSubKey('OpenWithProgids')
            try { $openWith.SetValue($programId, [byte[]]@(), [Microsoft.Win32.RegistryValueKind]::None) }
            finally { $openWith.Close() }
        } finally { $extension.Close() }
    } else {
        $extension = $classes.OpenSubKey('.st', $true)
        if ($null -ne $extension) {
            try {
                if ($state.GetValue('Claimed', 0) -eq 1 -and $extension.GetValue('', '') -eq $programId) {
                    if ($state.GetValue('HadDefault', 0) -eq 1) { $extension.SetValue('', '') }
                    else { $extension.DeleteValue('', $false) }
                }
                $openWith = $extension.OpenSubKey('OpenWithProgids', $true)
                if ($null -ne $openWith) {
                    try { $openWith.DeleteValue($programId, $false) } finally { $openWith.Close() }
                }
            } finally { $extension.Close() }
        }
        # Retira únicamente nuestro ProgID; conserva el resto de .st y otros editores.
        $classes.DeleteSubKeyTree($programId, $false)
    }
    # Notifica a Explorer para invalidar su caché de asociaciones/iconos.
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class PyForgeShellNotify {
    [DllImport("shell32.dll")]
    public static extern void SHChangeNotify(uint eventId, uint flags, IntPtr item1, IntPtr item2);
}
'@
    [PyForgeShellNotify]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
} finally {
    $classes.Close()
    $state.Close()
}
