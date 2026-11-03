param([string]$ResultDirectory = "$PSScriptRoot/../results/latest",[string]$Filter="",[string]$Python="python")
$ErrorActionPreference = 'Stop'
$project = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location -LiteralPath $project
# Build outside measurement. Only the examples benchmark executable is launched; preparation waits for job attachment.
$ManifestPath = Join-Path $project "examples/Cargo.toml"
$Benchmark = "showcase"
$artifacts = cargo bench --manifest-path $ManifestPath --bench $Benchmark --features native --locked --no-run --message-format=json
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$artifact = $artifacts | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq $Benchmark -and $_.executable } | Select-Object -Last 1
$binary = if ($artifact) { Get-Item -LiteralPath $artifact.executable } else { $null }
if (-not $binary) { throw 'Criterion executable was not built' }
$env:CRITERION_HOME = [IO.Path]::GetFullPath($ResultDirectory)
if (Test-Path -LiteralPath (Join-Path $env:CRITERION_HOME "memory.json")) { throw "Use a new ResultDirectory to preserve the previous run" }
[IO.Directory]::CreateDirectory($env:CRITERION_HOME) | Out-Null
& $Python -B -X utf8 (Join-Path $PSScriptRoot "capture-sources.py") (Join-Path $env:CRITERION_HOME "source-before.json") before
if ($LASTEXITCODE -ne 0) { throw "Source capture failed" }
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
public static class QslExampleBenchmarkJob {
    [StructLayout(LayoutKind.Sequential)] struct Basic {
        public long ProcessTime, JobTime; public uint Flags;
        public UIntPtr MinimumSet, MaximumSet; public uint Active;
        public UIntPtr Affinity; public uint Priority, Scheduling;
    }
    [StructLayout(LayoutKind.Sequential)] struct Counters { public ulong ReadOps, WriteOps, OtherOps, ReadBytes, WriteBytes, OtherBytes; }
    [StructLayout(LayoutKind.Sequential)] struct Extended {
        public Basic Basic; public Counters Counters;
        public UIntPtr ProcessLimit, JobLimit, PeakProcess, PeakJob;
    }
    [DllImport("kernel32", CharSet=CharSet.Unicode, SetLastError=true)] static extern IntPtr CreateJobObject(IntPtr security, string name);
    [DllImport("kernel32", SetLastError=true)] static extern bool SetInformationJobObject(IntPtr job, int type, ref Extended info, uint size);
    [DllImport("kernel32", SetLastError=true)] static extern bool QueryInformationJobObject(IntPtr job, int type, out Extended info, uint size, IntPtr returned);
    [DllImport("kernel32", SetLastError=true)] static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);
    [DllImport("kernel32")] public static extern bool CloseHandle(IntPtr handle);
    public static IntPtr Create() {
        var handle=CreateJobObject(IntPtr.Zero,null);
        if(handle==IntPtr.Zero)throw new Win32Exception();
        // JOB_MEMORY + KILL_ON_JOB_CLOSE. The job includes all descendants.
        var info=new Extended();info.Basic.Flags=0x200u|0x2000u;info.JobLimit=new UIntPtr(8UL*1024*1024*1024);
        if(!SetInformationJobObject(handle,9,ref info,(uint)Marshal.SizeOf<Extended>())) {CloseHandle(handle);throw new Win32Exception();}
        return handle;
    }
    public static void Assign(IntPtr job,IntPtr process){if(!AssignProcessToJobObject(job,process))throw new Win32Exception();}
    public static ulong Peak(IntPtr job){Extended info;if(!QueryInformationJobObject(job,9,out info,(uint)Marshal.SizeOf<Extended>(),IntPtr.Zero))throw new Win32Exception();return info.PeakJob.ToUInt64();}
}
'@
$job = [QslExampleBenchmarkJob]::Create()
$process = [Diagnostics.Process]::new()
$env:QSL_EXAMPLE_META = Join-Path $env:CRITERION_HOME 'cases.json'
[IO.Directory]::CreateDirectory($env:CRITERION_HOME) | Out-Null
$gate = Join-Path $env:CRITERION_HOME ('start-' + [guid]::NewGuid().ToString('N'))
$env:QSL_BENCH_START_FILE = $gate
$arguments = '--bench --noplot'
if ($Filter) { $arguments += ' ' + $Filter }

$process.StartInfo = [Diagnostics.ProcessStartInfo]::new($binary.FullName, $arguments)
$process.StartInfo.UseShellExecute = $false
$process.StartInfo.CreateNoWindow = $true
$process.StartInfo.RedirectStandardOutput = $true
$process.StartInfo.RedirectStandardError = $true
$exitCode = 1
try {
    [void]$process.Start()
    # Assignment precedes the 15-second warm-up and measurement. The executable
    # has a fixed tiny fixture and does not launch build tools or load clients.
    [QslExampleBenchmarkJob]::Assign($job,$process.Handle)
    [IO.File]::WriteAllText($gate,"attached")
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    $process.WaitForExit()
    $exitCode = $process.ExitCode
    & $Python -B -X utf8 (Join-Path $PSScriptRoot "capture-sources.py") (Join-Path $env:CRITERION_HOME "source-after.json") after
    $log = $stdout.GetAwaiter().GetResult() + "`n" + $stderr.GetAwaiter().GetResult()
    [IO.Directory]::CreateDirectory($env:CRITERION_HOME) | Out-Null
    [IO.File]::WriteAllText((Join-Path $env:CRITERION_HOME 'run.log'),$log)
    $measured = @(Get-ChildItem -LiteralPath $env:CRITERION_HOME -Filter estimates.json -Recurse | Where-Object { $_.Directory.Name -eq "new" }).Count
    $evidence = @{ measuredCases=$measured; warmupSeconds=15; measurementSeconds=10; samples=200; confidence=0.99; fixtureCount=1024; filter=$Filter; limitGiB = 8; peakJobBytes = [QslExampleBenchmarkJob]::Peak($job); exitCode = $exitCode; binary = $binary.FullName; sha256 = (Get-FileHash -LiteralPath $binary.FullName -Algorithm SHA256).Hash }
    $evidence | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $env:CRITERION_HOME 'memory.json')
    Write-Output ("Completed showcase; {0} Criterion cases" -f $measured)
    Write-Output ('Peak job memory: {0:N2} MiB; limit: 8 GiB' -f ($evidence.peakJobBytes/1MB))
} finally {
    if ($process.Id -and -not $process.HasExited) { $process.Kill($true) }
    $process.Dispose()
    [void][QslExampleBenchmarkJob]::CloseHandle($job)
}
exit $exitCode
