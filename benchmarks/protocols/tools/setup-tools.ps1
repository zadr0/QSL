$ErrorActionPreference='Stop'
Set-Location -LiteralPath $PSScriptRoot
$files=@(
 @{url='https://capnproto.org/capnproto-c++-win32-1.5.0.zip';file='capnp.zip';destination='capnp';sha='21501909DC051347563D4E83394EB848204F558E959501C8061DBEAAF0651988'},
 @{url='https://github.com/google/flatbuffers/releases/download/v25.12.19/Windows.flatc.binary.zip';file='flatc.zip';destination='flatc';sha='FFF9445C9DB907227BC64B54CC98743084C4949282AA4E576CFF6A955724DDC8'}
)
foreach($file in $files){if(-not(Test-Path -LiteralPath $file.file)){Invoke-WebRequest -Uri $file.url -OutFile $file.file};if((Get-FileHash -LiteralPath $file.file -Algorithm SHA256).Hash -ne $file.sha){throw ('Tool digest mismatch: '+$file.file)};Expand-Archive -LiteralPath $file.file -DestinationPath $file.destination -Force}
& './capnp/capnproto-tools-win32-1.5.0/capnp.exe' --version
& './flatc/flatc.exe' --version
