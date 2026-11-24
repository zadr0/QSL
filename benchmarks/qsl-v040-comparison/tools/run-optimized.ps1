$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $root
$env:RUSTFLAGS = '-C target-cpu=native'
$env:CARGO_TARGET_DIR = Join-Path $root 'target'
$env:CAPNP = (Resolve-Path tools/bin/capnp.exe).Path
$result = Join-Path $root 'results/optimized'
if(Test-Path -LiteralPath $result){throw 'Archive the current results/optimized directory before a new run'}
New-Item -ItemType Directory -Path $result | Out-Null

cargo fmt --manifest-path optimized/Cargo.toml -- --check
if($LASTEXITCODE -ne 0){throw 'Rust formatting check failed'}
cargo test --locked --offline --release --manifest-path optimized/Cargo.toml
if($LASTEXITCODE -ne 0){throw 'Round-trip and zero-copy contracts failed'}
cargo build --locked --offline --release --manifest-path optimized/Cargo.toml --bin optimized-probe
if($LASTEXITCODE -ne 0){throw 'Allocation probe build failed'}
cargo bench --locked --offline --manifest-path optimized/Cargo.toml --bench comparison --no-run --message-format=json > optimized/build-artifacts.jsonl
if($LASTEXITCODE -ne 0){throw 'Criterion build failed'}

$artifacts = @(Get-Content -LiteralPath optimized/build-artifacts.jsonl | ForEach-Object {try{$_ | ConvertFrom-Json}catch{}})
$exe = ($artifacts | Where-Object {$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'comparison' -and $_.executable} | Select-Object -Last 1).executable
$schema = ($artifacts | Where-Object {$_.reason -eq 'build-script-executed' -and $_.package_id -match '#qsl-optimized-bench@'} | Select-Object -Last 1).out_dir
if(-not $exe -or -not $schema){throw 'Compiled benchmark or schema not found'}
Copy-Item -LiteralPath (Join-Path $schema 'flat.qsl') -Destination (Join-Path $result 'generated.qsl')

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class OptimizedRunTime {
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetExitCodeProcess(IntPtr handle, out uint code);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetSystemTimes(out long idle,out long kernel,out long user);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetProcessTimes(IntPtr handle, out long create, out long exit, out long kernel, out long user);
    public static uint ExitCode(IntPtr handle) {
        uint code;
        if (!GetExitCodeProcess(handle,out code)) throw new System.ComponentModel.Win32Exception();
        return code;
    }
    public static long[] SystemTimes() {
        long idle,kernel,user;
        if (!GetSystemTimes(out idle,out kernel,out user)) throw new System.ComponentModel.Win32Exception();
        return new long[]{idle,kernel,user};
    }
    public static double ProcessCpu(IntPtr handle) {
        long create,exit,kernel,user;
        if (!GetProcessTimes(handle,out create,out exit,out kernel,out user)) throw new System.ComponentModel.Win32Exception();
        return (kernel+user)/10000000.0;
    }
}
'@

$sources=@{}
$tracked = Get-ChildItem -LiteralPath optimized/src,optimized/schemas,optimized/benches,optimized/tests,realistic/src,realistic/schemas -File -Recurse
foreach($file in $tracked){$sources[$file.FullName.Substring($root.Length+1).Replace('\','/')]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash}
foreach($file in @('optimized/Cargo.toml','optimized/Cargo.lock','optimized/build.rs','realistic/Cargo.toml','realistic/Cargo.lock','realistic/build.rs','tools/run-optimized.ps1','tools/pick-optimized.mjs')){
    $sources[$file]=(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
}
$processor=Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors
$os=Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version
if($processor.NumberOfLogicalProcessors -lt 3){throw 'The recorded affinity requires logical core 2'}
$environment = [ordered]@{
    startedAt=(Get-Date -Format o);cpu=$processor;os=$os;
    rust=(& rustc -Vv | Out-String).Trim();rustflags=$env:RUSTFLAGS;
    affinityCore=2;powerScheme=(& powercfg /getactivescheme | Out-String).Trim();
    executable=$exe;executableSha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;
    generatedSchemaSha256=(Get-FileHash -LiteralPath (Join-Path $result 'generated.qsl') -Algorithm SHA256).Hash;
    sources=$sources;
}
$environment | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $result 'environment.json') -Encoding utf8

function Measure-Phase([string]$name,[string]$warm,[string]$duration,[int]$samples,[string]$manifest='', [switch]$Reverse){
    $dir=Join-Path $result $name
    New-Item -ItemType Directory -Path $dir | Out-Null
    $env:BENCH_OUTPUT=Join-Path $dir 'criterion'
    if($manifest){$env:BENCH_SELECTION=$manifest}else{Remove-Item Env:BENCH_SELECTION -ErrorAction SilentlyContinue}
    if($Reverse){$env:BENCH_REVERSE='1'}else{Remove-Item Env:BENCH_REVERSE -ErrorAction SilentlyContinue}
    $before = [OptimizedRunTime]::SystemTimes()
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $process=Start-Process -FilePath $exe -ArgumentList @('--bench','--noplot','--warm-up-time',$warm,'--measurement-time',$duration,'--sample-size',"$samples") -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $dir 'stdout.log') -RedirectStandardError (Join-Path $dir 'stderr.log')
    $handle=$process.Handle
    $process.ProcessorAffinity=[IntPtr]4
    $process.PriorityClass='AboveNormal'
    Write-Host "Started $name, PID $($process.Id), core 2"
    $process.WaitForExit()
    $clock.Stop()
    $code=[OptimizedRunTime]::ExitCode($handle)
    $cpu=[OptimizedRunTime]::ProcessCpu($handle)
    $after=[OptimizedRunTime]::SystemTimes()
    $active=(($after[1]-$before[1])+($after[2]-$before[2])-($after[0]-$before[0]))/10000000.0
    $background=[math]::Max([double]0,($active-$cpu)/$clock.Elapsed.TotalSeconds)
    if($code -ne 0){Get-Content -LiteralPath (Join-Path $dir 'stderr.log') -Tail 25;throw "$name failed with $code"}
    Write-Host "Completed $name in $($clock.Elapsed.TotalSeconds) s"
    [ordered]@{name=$name;exitCode=$code;seconds=$clock.Elapsed.TotalSeconds;processCpuSeconds=$cpu;
        backgroundLogicalCores=$background;warmup=$warm;measurement=$duration;sampleSize=$samples;
        reverse=[bool]$Reverse;confirmation=[bool]$manifest}
}
$runs=@()
function Save-Runs { $runs | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $result 'runs.json') -Encoding utf8 }
$runs+=Measure-Phase 'tune' '0.15' '0.35' 36
Save-Runs
node tools/pick-optimized.mjs
if($LASTEXITCODE -ne 0){throw 'Profile selection failed'}
$manifest=Join-Path $result 'selection.json'
$runs+=Measure-Phase 'confirm-1' '0.75' '1.5' 100 $manifest
Save-Runs
$runs+=Measure-Phase 'confirm-2' '0.75' '1.5' 100 $manifest -Reverse
Save-Runs
$env:BENCH_SELECTION=$manifest
& (Join-Path $env:CARGO_TARGET_DIR 'release/optimized-probe.exe') > (Join-Path $result 'profiles.json')
if($LASTEXITCODE -ne 0){throw 'Allocation probe failed'}
