$ErrorActionPreference='Stop'
$bin=Join-Path $PSScriptRoot 'bin'
$zip=Join-Path $bin 'capnp.zip'
New-Item -ItemType Directory -Force -Path $bin | Out-Null
Invoke-WebRequest -Uri 'https://capnproto.org/capnproto-c++-win32-1.5.0.zip' -OutFile $zip
if((Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash -ne '21501909DC051347563D4E83394EB848204F558E959501C8061DBEAAF0651988'){throw 'Unexpected compiler archive digest'}
$unpacked=Join-Path $bin 'unpacked'
Expand-Archive -LiteralPath $zip -DestinationPath $unpacked -Force
$compiler=Get-ChildItem -LiteralPath $unpacked -Filter capnp.exe -Recurse -File | Select-Object -First 1
if(-not $compiler){throw 'capnp.exe not found'}
Copy-Item -LiteralPath $compiler.FullName -Destination (Join-Path $bin 'capnp.exe') -Force
& (Join-Path $bin 'capnp.exe') --version
