param([string]$Output="$PSScriptRoot/results/2026-11-06")
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location -LiteralPath $root
$artifacts=cargo bench --manifest-path "$PSScriptRoot/Cargo.toml" --bench modernization --locked --no-run --message-format=json
if($LASTEXITCODE -ne 0){throw 'Benchmark build failed'}
$artifact=$artifacts|ForEach-Object{$_|ConvertFrom-Json}|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'modernization' -and $_.executable}|Select-Object -Last 1
if(-not $artifact){throw 'Benchmark executable missing'}
$Output=[IO.Path]::GetFullPath($Output);[void][IO.Directory]::CreateDirectory($Output)
Add-Type -Path (Join-Path $root 'benchmarks/protocols/tools/Measure.cs')
$gate=Join-Path $Output ('attached-'+[Guid]::NewGuid().ToString('N'));$env:QSL_BENCH_START_FILE=$gate;$env:CRITERION_HOME=Join-Path $Output 'criterion'
$job=[QslStrictMetrics]::CreateJob();$p=[Diagnostics.Process]::new();$p.StartInfo=[Diagnostics.ProcessStartInfo]::new($artifact.executable,'--bench --noplot');$p.StartInfo.UseShellExecute=$false;$p.StartInfo.CreateNoWindow=$true;$p.StartInfo.RedirectStandardOutput=$true;$p.StartInfo.RedirectStandardError=$true
try{[void]$p.Start();[QslStrictMetrics]::Attach($job,$p.Handle);[IO.File]::WriteAllText($gate,'attached');$stdout=$p.StandardOutput.ReadToEndAsync();$stderr=$p.StandardError.ReadToEndAsync();$p.WaitForExit();$metrics=[QslStrictMetrics]::Finish($job,$p.Handle);$log=$stdout.GetAwaiter().GetResult()+"`n"+$stderr.GetAwaiter().GetResult();[IO.File]::WriteAllText((Join-Path $Output 'run.log'),$log);@{exitCode=$p.ExitCode;limitGiB=8;peakJobBytes=$metrics[0];warmupSeconds=15;measurementSeconds=10;samples=200;confidence=0.99;ring=1024;stringUtf8Scalars=1024;stringUsefulBytes=2048;numericUsefulBytes=24;idlePolicy='user_waived_5_percent';hotMetricsSampler=$false;binarySha256=(Get-FileHash -LiteralPath $artifact.executable -Algorithm SHA256).Hash;rustc=(& rustc -Vv|Out-String)}|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $Output 'metadata.json');Write-Output $log;if($p.ExitCode -ne 0){throw 'Benchmark failed'}}finally{if($p.Id -and -not $p.HasExited){$p.Kill($true)};$p.Dispose();[void][QslStrictMetrics]::CloseHandle($job);Remove-Item Env:QSL_BENCH_START_FILE -ErrorAction SilentlyContinue}
