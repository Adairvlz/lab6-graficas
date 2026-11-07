param([switch]$Capturas)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
$env:Path = "$env:USERPROFILE\.cargo\bin;C:\Program Files\CMake\bin;C:\Program Files\LLVM\bin;" + $env:Path
if (Test-Path -LiteralPath 'C:\Program Files\LLVM\bin\libclang.dll') {
    $env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
}
if ($Capturas) { cargo run --release --locked -- --capture }
else { cargo run --release --locked }
exit $LASTEXITCODE
