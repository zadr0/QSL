param([string]$Output="",[string]$Filter="",[int]$Repeat=1,[double]$IdleMaxPercent=5,[int]$IdleSamples=3,[int]$IdleTimeoutSeconds=180,[switch]$Sampler,[int]$MaxDurationSeconds=14400,[string]$StopFile='',[switch]$SkipIdleGate)
$ErrorActionPreference="Stop"
$sessionClock=[Diagnostics.Stopwatch]::StartNew()
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $project
if(-not $Output){$Output=Join-Path $project ('results/'+(Get-Date -Format 'yyyyMMdd-HHmmss'))}
$Output=[IO.Path]::GetFullPath($Output)
[IO.Directory]::CreateDirectory($Output)|Out-Null
$env:RUSTFLAGS="-C target-cpu=native"
$artifacts=cargo bench --manifest-path Cargo.toml --bench comparison --locked --no-run --message-format=json
if($LASTEXITCODE -ne 0){throw "Build failed"}
$binary=($artifacts|ForEach-Object{$_|ConvertFrom-Json}|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'comparison' -and $_.executable}|Select-Object -Last 1).executable
if(-not $binary){throw "Missing executable"}
$binarySha=(Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash
@{createdUtc=(Get-Date).ToUniversalTime().ToString('o');policy=$(if($SkipIdleGate){'ambient'}else{'idle'});idleGateEnforced=(-not [bool]$SkipIdleGate);binarySha256=$binarySha}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Output 'run.json')
Add-Type -Path (Join-Path $PSScriptRoot 'Measure.cs')
$matrix=Get-Content -LiteralPath 'matrix.json' -Raw|ConvertFrom-Json
$cases=@($matrix.cases|Where-Object{-not $Filter -or $_ -match $Filter})
# Deterministic shuffled order; the matrix and order are retained.
$random=[Random]::new([int]$matrix.seed)
$cases=@($cases|Sort-Object { $random.Next() })
$cases|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Output 'order.json')
$sourceFiles=@(Get-ChildItem -LiteralPath 'src','schemas','benches','tools' -File -Recurse|Where-Object{$_.Extension -in '.rs','.qsl','.deq','.proto','.capnp','.fbs','.ps1','.cs'})+@(Get-Item 'Cargo.toml','Cargo.lock','matrix.json')+@(Get-ChildItem -LiteralPath '../../qsl/src' -File -Recurse -Filter '*.rs')+@(Get-Item '../../qsl/Cargo.toml','../../qsl/Cargo.lock','../../qsl/macros/Cargo.toml')+@(Get-ChildItem -LiteralPath '../../qsl/macros/src' -File -Recurse -Filter '*.rs')
$hashes=@{};foreach($file in $sourceFiles){$hashes[[IO.Path]::GetRelativePath($project,$file.FullName)]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash}
$hashes|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $Output 'sources-before.json')
if(-not $StopFile){$StopFile=Join-Path $project 'results/STOP'}
$summary=[Collections.Generic.List[object]]::new()
try {
foreach($round in 1..$Repeat){foreach($case in $cases){
 if($sessionClock.Elapsed.TotalSeconds -ge $MaxDurationSeconds -or (Test-Path -LiteralPath $StopFile)){throw 'Session deadline or STOP file reached'}
 $id=($case -replace '/','_')+'-r'+$round
 $directory=Join-Path $Output $id
 if(Test-Path -LiteralPath (Join-Path $directory 'metrics.json')){throw "Existing completed case: $id"}
 [IO.Directory]::CreateDirectory($directory)|Out-Null
 $kernelBefore=[QslStrictMetrics]::BeginKernel();
 $idle=[Collections.Generic.List[object]]::new();$consecutive=0;$begin=[Diagnostics.Stopwatch]::StartNew();$prior=[QslStrictMetrics]::CpuTimes()
 while((($SkipIdleGate -and $idle.Count -lt $IdleSamples) -or (-not $SkipIdleGate -and $consecutive -lt $IdleSamples)) -and $begin.Elapsed.TotalSeconds -lt $IdleTimeoutSeconds){if($sessionClock.Elapsed.TotalSeconds -ge $MaxDurationSeconds -or (Test-Path -LiteralPath $StopFile)){break};Start-Sleep -Milliseconds 1000;$next=[QslStrictMetrics]::CpuTimes();$busy=[QslStrictMetrics]::Busy($prior,$next);$peak=($busy|Measure-Object -Maximum).Maximum;$idle.Add(@{corePercent=$busy;maximum=$peak;atSeconds=$begin.Elapsed.TotalSeconds});if($peak -le $IdleMaxPercent){$consecutive++}else{$consecutive=0};$prior=$next}
 $idle|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $directory 'idle.json')
 if(-not $SkipIdleGate -and $consecutive -lt $IdleSamples){$summary.Add(@{case=$case;round=$round;status='idle_gate_failed'});$summary|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $Output 'progress.json');[void][QslStrictMetrics]::EndKernel($kernelBefore);Write-Output "IDLE REJECTED $case";throw "Idle gate failed; no timing result was accepted"}
 if($sessionClock.Elapsed.TotalSeconds -ge $MaxDurationSeconds -or (Test-Path -LiteralPath $StopFile)){[void][QslStrictMetrics]::EndKernel($kernelBefore);throw 'Session deadline or STOP file reached'}
 $job=[QslStrictMetrics]::CreateJob();$process=[Diagnostics.Process]::new();$gate=Join-Path $directory 'attached';$env:QSL_BENCH_START_FILE=$gate;$env:QSL_CASE=$case;$env:QSL_CASE_METADATA=Join-Path $directory 'case.json';$env:CRITERION_HOME=Join-Path $directory 'criterion'
 $process.StartInfo=[Diagnostics.ProcessStartInfo]::new($binary,'--bench --noplot');$process.StartInfo.UseShellExecute=$false;$process.StartInfo.CreateNoWindow=$true;$process.StartInfo.RedirectStandardOutput=$true;$process.StartInfo.RedirectStandardError=$true
 $otherBefore=[QslStrictMetrics]::BeginProcesses();[QslStrictMetrics]::MarkKernel($kernelBefore);$cpuBefore=[QslStrictMetrics]::CpuTimes();$clock=[Diagnostics.Stopwatch]::StartNew()
 try{[void]$process.Start();[QslStrictMetrics]::Attach($job,$process.Handle);[IO.File]::WriteAllText($gate,'attached');$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync();$diagnostics=[Collections.Generic.List[object]]::new();$lastCpu=$cpuBefore
 # Default latency run blocks on the process: no sampler, polling, counter or logging call during measurement.
 if($Sampler){while(-not $process.WaitForExit(1000)){$next=[QslStrictMetrics]::CpuTimes();$process.Refresh();$diagnostics.Add(@{time=$clock.Elapsed.TotalSeconds;corePercent=[QslStrictMetrics]::Busy($lastCpu,$next);workingSet=$process.WorkingSet64;privateBytes=$process.PrivateMemorySize64});$lastCpu=$next}}else{if(-not $process.WaitForExit(60000)){$process.Kill($true);throw "Case exceeded 60 seconds"}}
 $clock.Stop();$cpuAfter=[QslStrictMetrics]::CpuTimes();$jobData=[QslStrictMetrics]::Finish($job,$process.Handle);$cpu=$jobData[1]+$jobData[2];$systemBusy=[QslStrictMetrics]::BusySeconds($cpuBefore,$cpuAfter);$residual=[Math]::Max(0.0,100*($systemBusy-$cpu)/($clock.Elapsed.TotalSeconds*[Environment]::ProcessorCount));$other=[QslStrictMetrics]::EndProcesses($otherBefore,$process.Id);$kernel=[QslStrictMetrics]::EndKernel($kernelBefore);$background=100*$other[0]/($clock.Elapsed.TotalSeconds*[Environment]::ProcessorCount);$nonDpc=if($kernel[0] -ge 0 -and $kernel[1] -ge 0){[Math]::Max(0.0,$residual-$kernel[0]-$kernel[1])}else{$residual}
 $metrics=@{case=$case;round=$round;exitCode=$process.ExitCode;wallSeconds=$clock.Elapsed.TotalSeconds;cpuUserSeconds=$jobData[1];cpuKernelSeconds=$jobData[2];cpuCores=$cpu/$clock.Elapsed.TotalSeconds;processCycles=$jobData[3];peakJobBytes=$jobData[0];peakWorkingSetBytes=$jobData[4];limitGiB=8;perCorePercent=[QslStrictMetrics]::Busy($cpuBefore,$cpuAfter);metricsRevision=3;idleGateEnforced=(-not [bool]$SkipIdleGate);idlePolicy=$(if($SkipIdleGate){'user_waived'}else{'enforced'});noisePolicy=$(if($SkipIdleGate){'annotation_only'}else{'reject'});otherProcessCpuPercent=$background;unattributedCpuPercent=$residual;nonDpcResidualCpuPercent=$nonDpc;dpcCpuPercent=$kernel[0];interruptCpuPercent=$kernel[1];systemContextSwitchesPerSecond=$kernel[2];queriedProcesses=$other[1];unavailableProcesses=$other[2];backgroundCpuPercent=$background;idleMaxPercent=$IdleMaxPercent;idleSamples=$IdleSamples;sampler=[bool]$Sampler;noiseAccepted=($background -le 2 -and $nonDpc -le 2);binarySha256=$binarySha;warmupSeconds=15;measurementSeconds=10;samples=200;confidence=0.99;rustFlags=$env:RUSTFLAGS}
 $metrics|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $directory 'metrics.json');$diagnostics|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $directory 'diagnostics.json');[IO.File]::WriteAllText((Join-Path $directory 'run.log'),$stdout.GetAwaiter().GetResult()+"`n"+$stderr.GetAwaiter().GetResult());$summary.Add($metrics);$summary|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $Output 'progress.json');Write-Output("DONE {0} · background {1:N2}% · peak {2:N2} MiB" -f $case,$background,($jobData[0]/1MB));if($process.ExitCode -ne 0){throw "Benchmark failed: $case"}
 }finally{if($process.Id -and -not $process.HasExited){$process.Kill($true)};$process.Dispose();[void][QslStrictMetrics]::CloseHandle($job)}
}}
} finally {
$after=@{};foreach($file in $sourceFiles){$after[[IO.Path]::GetRelativePath($project,$file.FullName)]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash};$after|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $Output 'sources-after.json')
}
if((Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash -ne $binarySha){throw "Executable changed during the run"}
$after=@{};foreach($file in $sourceFiles){$after[[IO.Path]::GetRelativePath($project,$file.FullName)]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash};$after|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $Output 'sources-after.json');foreach($key in $hashes.Keys){if($hashes[$key] -ne $after[$key]){throw "Measured sources changed: $key"}}
Write-Output "Completed $($summary.Count) cases; original samples and noise flags preserved."
