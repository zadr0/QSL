param([int]$Port=7342,[switch]$TransportOnly)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $root
$env:RUSTFLAGS = '-C target-cpu=native'
$env:CARGO_TARGET_DIR = Join-Path $root 'target'
$env:CAPNP = (Resolve-Path tools/bin/capnp.exe).Path
$suite = 'realistic'
$result = Join-Path $root 'results/realistic'
if (-not $TransportOnly -and (Test-Path -LiteralPath (Join-Path $result 'runs.json'))) { throw 'Archive existing realistic results before measuring again' }
New-Item -ItemType Directory -Force -Path $result | Out-Null

$release = Join-Path $env:CARGO_TARGET_DIR 'release'
$api = Join-Path $release 'api.exe'
$socket = Join-Path $release 'socket.exe'
if (-not $TransportOnly) {
    cargo test --release --offline --locked --manifest-path realistic/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Round-trip tests failed' }
    cargo build --release --offline --locked --manifest-path realistic/Cargo.toml --bins
    if ($LASTEXITCODE -ne 0) { throw 'Transport build failed' }
    cargo bench --offline --locked --manifest-path realistic/Cargo.toml --bench comparison --no-run --message-format=json > realistic/build-artifacts.jsonl
    if ($LASTEXITCODE -ne 0) { throw 'Criterion build failed' }
    & (Join-Path $release 'probe.exe') > (Join-Path $result 'profiles.json')
    if ($LASTEXITCODE -ne 0) { throw 'Allocation probe failed' }
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/run-dyn.ps1 -Suite $suite -Prefix realistic-
    if ($LASTEXITCODE -ne 0) { throw 'Criterion trials failed' }
}

$socketStart = Get-Date -Format o
& $socket > (Join-Path $result 'socket.json')
if ($LASTEXITCODE -ne 0) { throw 'Socket verification failed' }

$k6 = (Get-Command k6).Source
$fixtures = Join-Path $result 'fixtures'
$http = Join-Path $result 'http'
New-Item -ItemType Directory -Path $http | Out-Null
& $api export $fixtures
if ($LASTEXITCODE -ne 0) { throw 'HTTP fixtures failed' }
$manifest = @(Get-Content -Raw -LiteralPath (Join-Path $fixtures 'manifest.json') | ConvertFrom-Json)
$names = @($manifest | ForEach-Object { $_.codec })
if ($names.Count -ne 6) { throw 'Expected six HTTP profiles' }
$metadata = [ordered]@{
    socketStartedAt = $socketStart
    socketExecutableSha256 = (Get-FileHash -LiteralPath $socket -Algorithm SHA256).Hash
    httpStartedAt = (Get-Date -Format o)
    serverExecutableSha256 = (Get-FileHash -LiteralPath $api -Algorithm SHA256).Hash
    k6 = (& $k6 version | Out-String).Trim()
    serverAffinity = 2
    clientAffinity = @(0,1,3)
    k6ScriptSha256 = (Get-FileHash -LiteralPath realistic/k6/api.js -Algorithm SHA256).Hash
    topology = 'loopback TCP and HTTP/1.1 persistent connections'
}
$metadata | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $result 'transport-environment.json') -Encoding utf8

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class RealisticBenchTimes {
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetExitCodeProcess(IntPtr handle, out uint code);
    public static uint ExitCode(IntPtr handle) {
        uint code;
        if (!GetExitCodeProcess(handle, out code)) throw new System.ComponentModel.Win32Exception();
        return code;
    }
}
'@

$server = Start-Process -FilePath $api -ArgumentList @('serve', "$Port") -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $http 'server.stdout.log') -RedirectStandardError (Join-Path $http 'server.stderr.log')
try {
    $server.ProcessorAffinity = [IntPtr]4
    $ready = $false
    $env:BASE_URL = "http://127.0.0.1:$Port"
    for ($i=0; $i -lt 20; $i++) {
        if ($server.HasExited) { throw 'Local HTTP server exited before readiness' }
        try {
            if ((Invoke-WebRequest -UseBasicParsing "$env:BASE_URL/health" -TimeoutSec 1).StatusCode -eq 200) { $ready=$true; break }
        } catch {}
        Start-Sleep -Milliseconds 250
    }
    if (-not $ready) { throw 'Local HTTP server did not become ready' }
    $env:FIXTURES = $fixtures.Replace('\','/')
    $env:K6_NO_USAGE_REPORT = 'true'
    $env:K6_NEW_MACHINE_READABLE_SUMMARY = 'false'
    $runs = @()
    for ($pass=1; $pass -le 2; $pass++) {
        if ($pass -eq 2) { [Array]::Reverse($names) }
        foreach ($name in $names) {
            $id = "pass-$pass-$name"
            $env:CODEC = $name
            $env:SUMMARY = Join-Path $http "$id.json"
            $before = [Diagnostics.Stopwatch]::StartNew()
            $client = Start-Process -FilePath $k6 -ArgumentList @('run','--quiet','--no-usage-report','realistic/k6/api.js') -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $http "$id.stdout.log") -RedirectStandardError (Join-Path $http "$id.stderr.log")
            $procHandle = $client.Handle
            if (-not $client.HasExited) { $client.ProcessorAffinity = [IntPtr]11 }
            $client.WaitForExit()
            $exitCode = [RealisticBenchTimes]::ExitCode($procHandle)
            $before.Stop()
            if ($exitCode -ne 0 -or -not (Test-Path -LiteralPath $env:SUMMARY)) {
                Get-Content -LiteralPath (Join-Path $http "$id.stderr.log") -Tail 15
                throw "k6 failed for $id (exit $exitCode)"
            }
            $runs += [ordered]@{ id=$id; pass=$pass; codec=$name; exitCode=$exitCode; elapsedSeconds=$before.Elapsed.TotalSeconds }
            $runs | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $result 'http-runs.json') -Encoding utf8
            Write-Output "HTTP $id passed all scenario checks"
        }
    }
} finally {
    if (-not $server.HasExited) { $server.Kill(); $server.WaitForExit() }
}
