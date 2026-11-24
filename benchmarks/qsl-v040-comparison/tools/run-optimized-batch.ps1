$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath (Join-Path $PSScriptRoot '..')
$root = (Get-Location).Path
$result = Join-Path $root 'results/optimized-batch'
if(Test-Path -LiteralPath $result){throw 'Archive the previous batch results before running again'}
$env:RUSTFLAGS = '-C target-cpu=native'
$env:CARGO_TARGET_DIR = Join-Path $root 'target'
$env:CAPNP = (Resolve-Path tools/bin/capnp.exe).Path
cargo fmt --manifest-path optimized-batch/Cargo.toml -- --check
if($LASTEXITCODE -ne 0){throw 'Batch formatting check failed'}
cargo build --release --locked --offline --manifest-path optimized-batch/Cargo.toml
if($LASTEXITCODE -ne 0){throw 'Batch build failed'}
$exe = Join-Path $env:CARGO_TARGET_DIR 'release/qsl-optimized-batch.exe'
New-Item -ItemType Directory -Path $result | Out-Null
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class BatchCpu {
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetSystemTimes(out long idle, out long kernel, out long user);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetProcessTimes(IntPtr process, out long creation, out long exit, out long kernel, out long user);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetExitCodeProcess(IntPtr process, out uint code);
    public static double Active() {
        long idle,kernel,user;
        if (!GetSystemTimes(out idle,out kernel,out user)) throw new System.ComponentModel.Win32Exception();
        return (kernel+user-idle)/10000000.0;
    }
    public static double Active(IntPtr process) {
        long creation,exit,kernel,user;
        if (!GetProcessTimes(process,out creation,out exit,out kernel,out user)) throw new System.ComponentModel.Win32Exception();
        return (kernel+user)/10000000.0;
    }
    public static uint ExitCode(IntPtr process) {
        uint code;
        if (!GetExitCodeProcess(process,out code)) throw new System.ComponentModel.Win32Exception();
        return code;
    }
}
'@
$sources = @{}
foreach($file in @('optimized-batch/Cargo.toml','optimized-batch/Cargo.lock','optimized-batch/src/main.rs','tools/run-optimized-batch.ps1','tools/optimized-report.mjs')){
    $sources[$file]=(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
}
[ordered]@{startedAt=(Get-Date -Format o);rustflags=$env:RUSTFLAGS;affinityCore=2;
    executableSha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;
    sources=$sources} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $result 'environment.json') -Encoding utf8
$runs = @()
foreach($name in @('tune','confirm-1','confirm-2')){
    $dir = Join-Path $result $name
    New-Item -ItemType Directory -Path $dir | Out-Null
    $env:BENCH_OUTPUT = Join-Path $dir 'criterion'
    Remove-Item Env:BENCH_REVERSE,Env:BENCH_SELECTION -ErrorAction SilentlyContinue
    $warm = '0.065'
    $duration = '0.135'
    $samples = 24
    if($name -ne 'tune'){
        $env:BENCH_SELECTION = Join-Path $result 'selection.json'
        $warm = '0.18'
        $duration = '0.37'
        $samples = 60
    }
    if($name -eq 'confirm-2'){$env:BENCH_REVERSE='1'}
    $before = [BatchCpu]::Active()
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $process = Start-Process -FilePath $exe -ArgumentList @('--bench','--noplot','--warm-up-time',$warm,'--measurement-time',$duration,'--sample-size',"$samples") -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $dir 'stdout.log') -RedirectStandardError (Join-Path $dir 'stderr.log')
    $handle = $process.Handle
    $process.ProcessorAffinity = [IntPtr]4
    $process.PriorityClass = 'AboveNormal'
    Write-Host "Started batch $name, PID $($process.Id), core 2"
    $process.WaitForExit()
    $clock.Stop()
    $code = [BatchCpu]::ExitCode($handle)
    $cpu = [BatchCpu]::Active($handle)
    $background = [math]::Max([double]0,([BatchCpu]::Active()-$before-$cpu)/$clock.Elapsed.TotalSeconds)
    if($code -ne 0){Get-Content -LiteralPath (Join-Path $dir 'stderr.log') -Tail 25;throw "Batch $name failed with $code"}
    $runs += [ordered]@{name=$name;exitCode=$code;seconds=$clock.Elapsed.TotalSeconds;
        processCpuSeconds=$cpu;backgroundLogicalCores=$background;
        warmup=[double]$warm;measurement=[double]$duration;sampleSize=$samples;reverse=($name -eq 'confirm-2')}
    $runs | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $result 'runs.json') -Encoding utf8
    Write-Host "Completed batch $name in $($clock.Elapsed.TotalSeconds) s"
    if($name -eq 'tune'){
        node tools/optimized-report.mjs pick-batch
        if($LASTEXITCODE -ne 0){throw 'Batch profile selection failed'}
    }
}
$env:BENCH_SELECTION = Join-Path $result 'allocation-selection.json'
& (Join-Path $env:CARGO_TARGET_DIR 'release/optimized-probe.exe') > (Join-Path $result 'profiles.json')
if($LASTEXITCODE -ne 0){throw 'Batch allocation probe failed'}
