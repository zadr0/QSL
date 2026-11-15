$ErrorActionPreference='Stop'
$qslRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location -LiteralPath $qslRoot
$env:CARGO_TARGET_DIR=Join-Path $qslRoot '.optimization/docs-site-target'
cargo build --manifest-path docs-site/native/Cargo.toml --offline
if($LASTEXITCODE -ne 0){throw 'Native examples build failed'}
$env:CARGO_TARGET_DIR=Join-Path $qslRoot '.optimization/unified-target'
cargo build --manifest-path proofs/qsl/Cargo.toml --features rust-macros --bin qsl-analyze --locked --offline
if($LASTEXITCODE -ne 0){throw 'Analyzer build failed'}
