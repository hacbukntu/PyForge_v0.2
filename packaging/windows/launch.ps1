# Traduce solamente la ruta del puente. Los argumentos se pasan como una lista,
# nunca se concatenan en una instrucción bash ni se usa Invoke-Expression.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$OutputEncoding = [Console]::OutputEncoding
try {
    $root = (& wsl.exe --exec wslpath -u $PSScriptRoot | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $root) { throw 'No se pudo abrir la distribución WSL predeterminada.' }
    if ($args -contains '--open-browser') {
        # Solo abre la URL local que emite nuestro servidor; la ruta ST nunca se ejecuta.
        & wsl.exe --exec python3 "$root/launch.py" @args | ForEach-Object {
            Write-Output $_
            if ($_ -match '^ST IDE: http://127\.0\.0\.1:([0-9]+)(/inicio\.html|/)?$') {
                Start-Process ('http://127.0.0.1:' + $Matches[1] + $Matches[2])
            }
        }
    } else {
        & wsl.exe --exec python3 "$root/launch.py" @args
    }
    exit $LASTEXITCODE
} catch {
    Write-Error "PyForge requiere WSL con Python 3.11 o superior: $_"
    exit 1
}
