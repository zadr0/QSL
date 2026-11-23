param([int]$Passes=2,[int]$Core=2,[ValidateSet('dyn','layout','realistic','qsl-features')][string]$Suite='dyn',[string]$Prefix="$Suite-",[string]$Compiler=$env:CAPNP,[int]$StartPass=1,[double]$MaxCpuPercent=20,[int]$GateTimeoutSeconds=600)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot "../$Suite"))
Set-Location -LiteralPath $root
if($Compiler){$env:CAPNP=$Compiler}
$env:RUSTFLAGS='-C target-cpu=native'
if($Suite -eq 'qsl-features'){
 $env:CARGO_TARGET_DIR=[IO.Path]::GetFullPath((Join-Path $root '../target'))
 cargo test --offline --release --locked
 if($LASTEXITCODE -ne 0){throw 'QSL feature contracts failed'}
 cargo bench --offline --locked --bench comparison --no-run --message-format=json > build-artifacts.jsonl
 if($LASTEXITCODE -ne 0){throw 'QSL feature Criterion build failed'}
 New-Item -ItemType Directory -Force -Path '../results/qsl-features' | Out-Null
 cargo run --offline --release --locked --bin probe > ../results/qsl-features/profiles.json
 if($LASTEXITCODE -ne 0){throw 'QSL feature allocation probe failed'}
}
if(-not(Test-Path -LiteralPath 'build-artifacts.jsonl')){throw 'Build first with the documented cargo bench --no-run command.'}
$artifact=Get-Content -LiteralPath 'build-artifacts.jsonl' | ForEach-Object {try{$_|ConvertFrom-Json}catch{}} | Where-Object {$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'comparison' -and $_.executable} | Select-Object -Last 1
if(-not $artifact){throw 'Benchmark executable was not found in build-artifacts.jsonl'}
$exe=$artifact.executable
$results=Join-Path $root "../results/$Suite"
New-Item -ItemType Directory -Force -Path $results | Out-Null
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class BenchSystemTimes {
 [DllImport("kernel32.dll",SetLastError=true)]
 static extern bool GetSystemTimes(out long idle,out long kernel,out long user);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr handle,out uint code);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr handle,out long created,out long exited,out long kernel,out long user);
 public static uint ExitCode(IntPtr handle){uint code;if(!GetExitCodeProcess(handle,out code))throw new Exception("GetExitCodeProcess");return code;}
 public static double CpuSeconds(IntPtr handle){long c,e,k,u;if(!GetProcessTimes(handle,out c,out e,out k,out u))throw new Exception("GetProcessTimes");return (k+u)/10000000.0;}
 public static long[] Read(){long i,k,u;if(!GetSystemTimes(out i,out k,out u))throw new Exception("GetSystemTimes");return new long[]{i,k,u};}
}
'@
$test=Start-Process -FilePath "$env:SystemRoot\System32\cmd.exe" -ArgumentList @("/c","exit","7") -WindowStyle Hidden -PassThru
$testHandle=$test.Handle;$test.WaitForExit();if([BenchSystemTimes]::ExitCode($testHandle) -ne 7){throw "Exit-code self-test failed"}
$cpu=Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed
$os=Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,TotalVisibleMemorySize
$memory=Get-CimInstance Win32_PhysicalMemory | Select-Object Capacity,Speed
$hashes=@{}
Get-ChildItem -LiteralPath src,schemas,benches,tests -Recurse -File | Sort-Object FullName | ForEach-Object {$hashes[$_.FullName.Substring($root.Length+1)]=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
foreach($name in @('Cargo.toml','Cargo.lock','build.rs')){$hashes[$name]=(Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash}
if($Suite -eq 'dyn'){
 Get-ChildItem -LiteralPath ../src,../schemas -Recurse -File | ForEach-Object {$relative='../'+$_.FullName.Substring(([IO.Path]::GetFullPath((Join-Path $root '..'))).Length+1).Replace('\','/');$hashes[$relative]=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
 foreach($name in @('../Cargo.toml','../Cargo.lock','../build.rs')){$hashes[$name]=(Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash}
}
$environment=[ordered]@{
 dateLocal=(Get-Date -Format o);timeZone=(Get-TimeZone).Id;cpu=$cpu;os=$os;memory=$memory;
 rust=(& rustc -Vv | Out-String);cargo=(& cargo -V | Out-String);rustFlags=$env:RUSTFLAGS;
 affinityCore=$Core;priority='AboveNormal';mode='released desktop: total CPU <=20% pre-run; affinity core 2; two reverse-order passes';
 warmupSeconds=1;measurementSeconds=3;samples=200;confidence=0.99;resamples=10000;
 executable=$exe;executableSha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;
 sources=$hashes;powerPlan=(& powercfg /getactivescheme | Out-String)
}
$runs=@()
if($StartPass -gt 1){
 $previous=Get-Content -Raw -LiteralPath (Join-Path $results 'environment.json')|ConvertFrom-Json
 if($previous.executableSha256 -ne $environment.executableSha256){throw 'Resume executable differs'}
 foreach($name in $hashes.Keys){if($previous.sources.$name -ne $hashes[$name]){throw "Resume source differs: $name"}}
 $runs=@(Get-Content -Raw -LiteralPath (Join-Path $results 'runs.json')|ConvertFrom-Json)
 if($runs.Count -ne $StartPass-1 -or @($runs|Where-Object {$_.exitCode -ne 0}).Count){throw 'Resume requires consecutive successful earlier passes'}
 $environment=$previous
 if(-not $environment.PSObject.Properties['resumedAt']){$environment|Add-Member NoteProperty resumedAt (Get-Date -Format o)}
 if(-not $environment.PSObject.Properties['resumeCpuLimitPercent']){$environment|Add-Member NoteProperty resumeCpuLimitPercent $MaxCpuPercent}else{$environment.resumeCpuLimitPercent=$MaxCpuPercent}
 $environment.mode="desktop background recorded; original pass gate <=20%; resumed pass gate <=$MaxCpuPercent%; affinity core $Core; two reverse-order passes"
}elseif(Test-Path -LiteralPath (Join-Path $results 'runs.json')){throw 'Existing Dyn results: use -StartPass to resume, or archive them before a new run'}
$environment|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $results 'environment.json') -Encoding utf8
for($pass=$StartPass;$pass -le $Passes;$pass++){
 $run="${Prefix}run-$pass"
 $env:BENCH_OUTPUT=Join-Path $results "$run/criterion"
 if($pass%2 -eq 0){$env:BENCH_REVERSE='1'}else{Remove-Item Env:BENCH_REVERSE -ErrorAction SilentlyContinue}
 $quiet=0;$gate=[Diagnostics.Stopwatch]::StartNew();$trace=@()
 do {
  $idleStart=[BenchSystemTimes]::Read();Start-Sleep -Seconds 2;$idleEnd=[BenchSystemTimes]::Read()
  $idleTotal=($idleEnd[1]-$idleStart[1])+($idleEnd[2]-$idleStart[2])
  $preLoad=100*(1-($idleEnd[0]-$idleStart[0])/[double]$idleTotal)
  $coreLoad=Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter ('Name="'+$Core+'"') | Select-Object -ExpandProperty PercentProcessorTime
  $trace+=[ordered]@{elapsed=$gate.Elapsed.TotalSeconds;totalCpuPercent=$preLoad;selectedCorePercent=$coreLoad}
  if($preLoad -le $MaxCpuPercent){$quiet++}else{$quiet=0}
  Write-Output "Idle gate: total $([math]::Round($preLoad,1))%, core $coreLoad%, consecutive $quiet/3"
  if($gate.Elapsed.TotalSeconds -gt $GateTimeoutSeconds){throw 'CPU did not reach the requested background threshold; no timing process was launched'}
 }while($quiet -lt 3)
 $trace|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $results "$run.idle.json") -Encoding utf8
 $before=[BenchSystemTimes]::Read();$clock=[Diagnostics.Stopwatch]::StartNew()
 $p=Start-Process -FilePath $exe -ArgumentList @('--bench','--noplot','--warm-up-time','1','--measurement-time','3','--sample-size','200') -WorkingDirectory $root -RedirectStandardOutput (Join-Path $results "$run.stdout.log") -RedirectStandardError (Join-Path $results "$run.stderr.log") -PassThru -WindowStyle Hidden
 $processHandle=$p.Handle
 $p.ProcessorAffinity=[IntPtr](1 -shl $Core)
 $p.PriorityClass='AboveNormal'
 Write-Output "Started $run, PID $($p.Id), core $Core, pre-run total CPU $([math]::Round($preLoad,1))%"
 $p.WaitForExit();$exitCode=[BenchSystemTimes]::ExitCode($processHandle);$clock.Stop();$after=[BenchSystemTimes]::Read()
 $cpuSeconds=[BenchSystemTimes]::CpuSeconds($processHandle);$peakBytes=$null
 $activeSeconds=((($after[1]-$before[1])+($after[2]-$before[2]))-($after[0]-$before[0]))/10000000.0
 $row=[ordered]@{run=$run;exitCode=$exitCode;elapsedSeconds=$clock.Elapsed.TotalSeconds;processCpuSeconds=$cpuSeconds;systemBefore=$before;systemAfter=$after;activeCpuSeconds=$activeSeconds;otherSystemCpuCoreEquivalent=$(if($null -ne $cpuSeconds){[math]::Max([double]0,($activeSeconds-$cpuSeconds)/$clock.Elapsed.TotalSeconds)}else{$null});preRunTotalCpuPercent=$preLoad;peakWorkingSetBytes=$peakBytes;reverseOrder=($pass%2 -eq 0)}
 $runs+=$row
 $runs|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $results 'runs.json') -Encoding utf8
 Write-Output "Finished $run, exit $($exitCode), $([math]::Round($clock.Elapsed.TotalSeconds,1)) s"
 if($exitCode -ne 0){Get-Content -LiteralPath (Join-Path $results "$run.stderr.log") -Tail 15;throw "Benchmark $run failed"}
}
