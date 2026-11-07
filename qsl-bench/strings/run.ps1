param([string]$Output="$PSScriptRoot/results/2026-11-08")
$ErrorActionPreference='Stop'
$qslRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location -LiteralPath $qslRoot
$qslBuild=cargo bench --manifest-path "$PSScriptRoot/Cargo.toml" --bench strings --locked --no-run --message-format=json --target-dir (Join-Path $qslRoot '.optimization/strings-refactor-target')
if($LASTEXITCODE -ne 0){throw 'Benchmark build failed'}
$qslArtifact=$qslBuild|Where-Object{$_.StartsWith('{')}|ForEach-Object{$_|ConvertFrom-Json}|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'strings' -and $_.executable}|Select-Object -Last 1
if(-not $qslArtifact){throw 'Benchmark executable was not returned by Cargo'}
$qslExe=$qslArtifact.executable
$Output=[IO.Path]::GetFullPath($Output);[void][IO.Directory]::CreateDirectory($Output)
if(-not ('QslStrictMetrics' -as [type])) { Add-Type -Path (Join-Path $qslRoot 'benchmarks/protocols/tools/Measure.cs') }
$qslIdleDeadline=[DateTime]::UtcNow.AddMinutes(10);$qslIdleStreak=0;$qslIdleWindows=@();$qslIdleLastMessage=[DateTime]::MinValue
while([DateTime]::UtcNow -lt $qslIdleDeadline){
 $qslBefore=[QslStrictMetrics]::CpuTimes();Start-Sleep -Seconds 2;$qslIdle=[QslStrictMetrics]::Busy($qslBefore,[QslStrictMetrics]::CpuTimes());$qslIdleMean=($qslIdle|Measure-Object -Average).Average
 $qslIdleWindows+=@{timeUtc=[DateTime]::UtcNow.ToString('o');meanPercent=$qslIdleMean;perCore=$qslIdle}
 if($qslIdleMean -le 5){$qslIdleStreak++}else{$qslIdleStreak=0}
 if(([DateTime]::UtcNow-$qslIdleLastMessage).TotalSeconds -ge 30){Write-Output ('CPU background: '+[Math]::Round($qslIdleMean,2)+'%; idle windows: '+$qslIdleStreak);$qslIdleLastMessage=[DateTime]::UtcNow}
 if($qslIdleStreak -ge 3){break}
}
@{thresholdPercent=5;windowSeconds=2;requiredConsecutiveWindows=3;passed=($qslIdleStreak -ge 3);windows=$qslIdleWindows}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $Output 'idle-gate.json') -Encoding UTF8
if($qslIdleStreak -lt 3){throw 'Idle gate did not pass; no benchmark started'}
$qslCore=0;$qslMin=101.0
for($i=1;$i -lt $qslIdle.Length;$i++){if($qslIdle[$i] -lt $qslMin){$qslMin=$qslIdle[$i];$qslCore=$i}}
$qslGate=Join-Path $Output ('attached-'+[Guid]::NewGuid().ToString('N'));$env:QSL_BENCH_START_FILE=$qslGate;$env:CRITERION_HOME=Join-Path $Output 'criterion'
$qslJob=[QslStrictMetrics]::CreateJob();$qslProcess=[Diagnostics.Process]::new();$qslProcess.StartInfo=[Diagnostics.ProcessStartInfo]::new($qslExe);$qslProcess.StartInfo.ArgumentList.Add('--bench');$qslProcess.StartInfo.ArgumentList.Add('--noplot');$qslProcess.StartInfo.UseShellExecute=$false;$qslProcess.StartInfo.CreateNoWindow=$true;$qslProcess.StartInfo.RedirectStandardOutput=$true;$qslProcess.StartInfo.RedirectStandardError=$true
try {
 $qslOther=[QslStrictMetrics]::BeginProcesses()
 [void]$qslProcess.Start();[QslStrictMetrics]::Attach($qslJob,$qslProcess.Handle);$qslProcess.ProcessorAffinity=[IntPtr](1L -shl $qslCore);$qslProcess.PriorityClass=[Diagnostics.ProcessPriorityClass]::AboveNormal
 $qslTimes=[QslStrictMetrics]::CpuTimes();$qslWatch=[Diagnostics.Stopwatch]::StartNew();[IO.File]::WriteAllText($qslGate,'attached')
 $qslStdout=$qslProcess.StandardOutput.ReadToEndAsync();$qslStderr=$qslProcess.StandardError.ReadToEndAsync();$qslProcess.WaitForExit();$qslWatch.Stop()
 $qslOtherMetrics=[QslStrictMetrics]::EndProcesses($qslOther,$qslProcess.Id);$qslBusy=[QslStrictMetrics]::Busy($qslTimes,[QslStrictMetrics]::CpuTimes());$qslMetrics=[QslStrictMetrics]::Finish($qslJob,$qslProcess.Handle)
 $qslLog=$qslStdout.GetAwaiter().GetResult()+"`n"+$qslStderr.GetAwaiter().GetResult();[IO.File]::WriteAllText((Join-Path $Output 'run.log'),$qslLog)
 @{date='2026-11-08';exitCode=$qslProcess.ExitCode;limitGiB=8;peakJobBytes=$qslMetrics[0];processUserSeconds=$qslMetrics[1];processKernelSeconds=$qslMetrics[2];processCycles=$qslMetrics[3];peakWorkingSetBytes=$qslMetrics[4];elapsedSeconds=$qslWatch.Elapsed.TotalSeconds;affinityCore=$qslCore;priority='AboveNormal';preBusyPercentPerCore=$qslIdle;duringBusyPercentPerCore=$qslBusy;otherProcessCpuSeconds=$qslOtherMetrics[0];otherProcessesObserved=$qslOtherMetrics[1];unreadableProcessCount=$qslOtherMetrics[2];warmupSeconds=15;measurementSeconds=10;samples=200;confidence=0.99;ring=1024;mainUtf8Scalars=1024;mainUsefulBytes=2048;smallUsefulBytes=255;stackLimitBytes=2048;hotMetricsSampler=$false;uncheckedPrevalidated=$true;includesAllocationAndDropForNewString=$true;binarySha256=(Get-FileHash -LiteralPath $qslExe -Algorithm SHA256).Hash;rustc=(& rustc -Vv|Out-String)} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Output 'metadata.json') -Encoding UTF8
 Write-Output $qslLog
 if($qslProcess.ExitCode -ne 0){throw 'Benchmark failed'}
} finally {
 if($qslProcess.Id -and -not $qslProcess.HasExited){$qslProcess.Kill($true)}
 $qslProcess.Dispose();[void][QslStrictMetrics]::CloseHandle($qslJob);Remove-Item Env:QSL_BENCH_START_FILE -ErrorAction SilentlyContinue
}
