param([ValidateSet('allocation_probe','latency_probe')][string]$Program,[string]$Output)
$ErrorActionPreference='Stop'
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'));Set-Location -LiteralPath $project
$env:RUSTFLAGS='-C target-cpu=native'
cargo build --manifest-path Cargo.toml --bin $Program --release --locked
if($LASTEXITCODE -ne 0){throw 'Probe build failed'}
if(-not $Output){$Output=Join-Path $project ('results/'+$Program)};$Output=[IO.Path]::GetFullPath($Output);[IO.Directory]::CreateDirectory($Output)|Out-Null
Add-Type -Path (Join-Path $PSScriptRoot 'Measure.cs')
if($Program -eq 'latency_probe'){$ok=0;$clock=[Diagnostics.Stopwatch]::StartNew();$previous=[QslStrictMetrics]::CpuTimes();while($ok -lt 3 -and $clock.Elapsed.TotalSeconds -lt 180){Start-Sleep -Seconds 1;$next=[QslStrictMetrics]::CpuTimes();if(([QslStrictMetrics]::Busy($previous,$next)|Measure-Object -Maximum).Maximum -le 5){$ok++}else{$ok=0};$previous=$next};if($ok -lt 3){throw 'Latency probe: CPU idle gate failed'}}
$binary=Join-Path $project ('target/release/'+$Program+'.exe');$job=[QslStrictMetrics]::CreateJob();$p=[Diagnostics.Process]::new();$gate=Join-Path $Output 'attached';$env:QSL_BENCH_START_FILE=$gate
$p.StartInfo=[Diagnostics.ProcessStartInfo]::new($binary);$p.StartInfo.UseShellExecute=$false;$p.StartInfo.CreateNoWindow=$true;$p.StartInfo.RedirectStandardOutput=$true;$p.StartInfo.RedirectStandardError=$true
try{[void]$p.Start();[QslStrictMetrics]::Attach($job,$p.Handle);[IO.File]::WriteAllText($gate,'attached');$stdout=$p.StandardOutput.ReadToEndAsync();$stderr=$p.StandardError.ReadToEndAsync();if(-not $p.WaitForExit(60000)){$p.Kill($true);throw 'Diagnostic timeout'};$stats=[QslStrictMetrics]::Finish($job,$p.Handle);[IO.File]::WriteAllText((Join-Path $Output 'data.json'),$stdout.GetAwaiter().GetResult());[IO.File]::WriteAllText((Join-Path $Output 'stderr.log'),$stderr.GetAwaiter().GetResult());@{program=$Program;exitCode=$p.ExitCode;limitGiB=8;peakJobBytes=$stats[0];binarySha256=(Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash;diagnosticOnly=$true;inLatencyBinary=$false}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Output 'metrics.json');if($p.ExitCode -ne 0){throw 'Diagnostic failed'}}finally{if($p.Id -and -not $p.HasExited){$p.Kill($true)};$p.Dispose();[void][QslStrictMetrics]::CloseHandle($job)}
