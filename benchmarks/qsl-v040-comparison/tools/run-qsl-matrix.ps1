$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $root
$env:RUSTFLAGS = '-C target-cpu=native'
$env:CARGO_TARGET_DIR = Join-Path $root 'target'
$result = Join-Path $root 'results/qsl-matrix'
if (Test-Path -LiteralPath $result) { throw 'Archive the previous results/qsl-matrix before recording a new run' }

cargo fmt --manifest-path qsl-all-features/Cargo.toml -- --check
if ($LASTEXITCODE -ne 0) { throw 'Rust formatting check failed' }
cargo build --offline --locked --release --manifest-path qsl-all-features/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'QSL matrix build failed' }
$exe = Join-Path $env:CARGO_TARGET_DIR 'release/qsl-all-features-bench.exe'
New-Item -ItemType Directory -Path $result | Out-Null

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class MatrixCpu {
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
foreach ($file in @('qsl-all-features/Cargo.toml','qsl-all-features/Cargo.lock','qsl-all-features/build.rs',
    'qsl-all-features/src/main.rs','tools/run-qsl-matrix.ps1')) {
    $sources[$file] = (Get-FileHash -LiteralPath $file).Hash
}
$processor = Get-CimInstance Win32_Processor | Select-Object Name,NumberOfLogicalProcessors
if ($processor.NumberOfLogicalProcessors -lt 3) { throw 'Logical core 2 is unavailable' }
[ordered]@{ startedAt=(Get-Date -Format o); cpu=$processor.Name; rust=(& rustc -V);
    os=(Get-CimInstance Win32_OperatingSystem).Version; rustflags=$env:RUSTFLAGS; affinityCore=2;
    executableSha256=(Get-FileHash -LiteralPath $exe).Hash; sources=$sources } |
    ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $result 'environment.json') -Encoding utf8

$runs = @()
foreach ($phase in @('tune','confirm-1','confirm-2')) {
    $dir = Join-Path $result $phase
    New-Item -ItemType Directory -Path $dir | Out-Null
    $env:BENCH_OUTPUT = $dir
    Remove-Item Env:BENCH_SELECTION,Env:BENCH_REVERSE,Env:BENCH_VERIFY -ErrorAction SilentlyContinue
    $warm = '0.05'
    $measure = '0.105'
    $samples = 24
    if ($phase -ne 'tune') {
        $env:BENCH_SELECTION = Join-Path $result 'selection.json'
        $warm = '0.20'
        $measure = '0.40'
        $samples = 60
    }
    if ($phase -eq 'confirm-2') { $env:BENCH_REVERSE = '1' }
    $before = [MatrixCpu]::Active()
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $arguments = @('--bench','--noplot','--warm-up-time',$warm,'--measurement-time',$measure,'--sample-size',"$samples")
    $process = Start-Process -FilePath $exe -ArgumentList $arguments -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $dir 'stdout.log') -RedirectStandardError (Join-Path $dir 'stderr.log')
    $handle = $process.Handle
    $process.ProcessorAffinity = [IntPtr]4
    $process.PriorityClass = 'AboveNormal'
    Write-Host "Started $phase on logical core 2, PID $($process.Id)"
    $process.WaitForExit()
    $timer.Stop()
    $code = [MatrixCpu]::ExitCode($handle)
    $cpu = [MatrixCpu]::Active($handle)
    $background = [math]::Max([double]0,([MatrixCpu]::Active()-$before-$cpu)/$timer.Elapsed.TotalSeconds)
    if ($code -ne 0) {
        Get-Content -LiteralPath (Join-Path $dir 'stderr.log') -Tail 22
        throw "$phase failed with $code"
    }
    $runs += [ordered]@{phase=$phase;exitCode=$code;seconds=$timer.Elapsed.TotalSeconds;
        backgroundLogicalCores=$background;sampleSize=$samples;warmup=$warm;measurement=$measure}
    ConvertTo-Json -InputObject $runs -Depth 5 | Set-Content -LiteralPath (Join-Path $result 'runs.json') -Encoding utf8
    if ($phase -eq 'tune') {
        node tools/qsl-matrix-report.mjs select
        if ($LASTEXITCODE -ne 0) { throw 'Freeze of tuning results failed' }
    }
}
