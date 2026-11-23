param([int]$Passes=2,[int]$Seconds=10,[int]$VUs=8,[int]$Port=7340,[switch]$SkipOpen)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $root
$env:RUSTFLAGS='-C target-cpu=native'
$env:CARGO_TARGET_DIR=Join-Path $root 'target'
$env:CAPNP=Join-Path $root 'tools/bin/capnp.exe'
cargo fmt --manifest-path http/Cargo.toml -- --check
if($LASTEXITCODE -ne 0){throw 'HTTP formatting check failed'}
cargo build --offline --locked --release --manifest-path http/Cargo.toml
if($LASTEXITCODE -ne 0){throw 'HTTP release build failed'}
$results=Join-Path $root 'results/k6'
$fixtures=Join-Path $results 'fixtures'
New-Item -ItemType Directory -Force -Path $results,(Join-Path $results 'trials') | Out-Null
$serverExe=Join-Path $root 'target/release/qsl-comparison-http.exe'
if(-not(Test-Path -LiteralPath $serverExe)){throw 'Build the HTTP harness first'}
$k6=(Get-Command k6 -ErrorAction Stop).Source
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class HttpBenchTimes {
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetSystemTimes(out long idle,out long kernel,out long user);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr handle,out uint code);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr handle,out long created,out long exited,out long kernel,out long user);
 public static uint ExitCode(IntPtr handle){uint code;if(!GetExitCodeProcess(handle,out code))throw new Exception("GetExitCodeProcess");return code;}
 public static double CpuSeconds(IntPtr handle){long c,e,k,u;if(!GetProcessTimes(handle,out c,out e,out k,out u))throw new Exception("GetProcessTimes");return (k+u)/10000000.0;}
 public static long[] Read(){long i,k,u;if(!GetSystemTimes(out i,out k,out u))throw new Exception("GetSystemTimes");return new long[]{i,k,u};}
}
'@
& $serverExe --export $fixtures
if($LASTEXITCODE -ne 0){throw 'Fixture export failed'}
$manifest=Get-Content -Raw -LiteralPath (Join-Path $fixtures 'manifest.json') | ConvertFrom-Json
if($manifest.Count -ne 33){throw 'Expected 12 protocol baseline profiles and 21 Dyn profiles'}
$hashes=@{}
foreach($folder in @('http/src','dyn/src','dyn/schemas','dyn/benches','dyn/tests','src','schemas','k6')){
 Get-ChildItem -LiteralPath $folder -Recurse -File | ForEach-Object {$hashes[$_.FullName.Substring($root.Length+1)]=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
}
foreach($name in @('Cargo.toml','Cargo.lock','http/Cargo.toml','http/Cargo.lock','dyn/Cargo.toml','dyn/Cargo.lock','http/src/main.rs','tools/run-k6.ps1')){$hashes[$name]=(Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash}
$environment=[ordered]@{dateLocal=(Get-Date -Format o);k6=(& $k6 version|Out-String);rust=(& rustc -Vv|Out-String);cpu=(Get-CimInstance Win32_Processor|Select-Object Name,NumberOfLogicalProcessors);os=(Get-CimInstance Win32_OperatingSystem|Select-Object Caption,Version);serverAffinityCore=2;clientAffinityCores=@(0,1,3);transport='loopback HTTP/1.1 keep-alive; no TLS; client and server on one host';warmupSeconds=2;measurementSeconds=$Seconds;closedVUs=$VUs;openRate=1000;openVUs=32;serverExecutableSha256=(Get-FileHash -LiteralPath $serverExe -Algorithm SHA256).Hash;sources=$hashes;powerPlan=(& powercfg /getactivescheme|Out-String)}
$environment|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $results 'environment.json') -Encoding utf8
$server=Start-Process -FilePath $serverExe -ArgumentList @("127.0.0.1:$Port") -WorkingDirectory $root -RedirectStandardOutput (Join-Path $results 'server.stdout.log') -RedirectStandardError (Join-Path $results 'server.stderr.log') -WindowStyle Hidden -PassThru
$serverHandle=$server.Handle
$server.ProcessorAffinity=[IntPtr]4
$server.PriorityClass='AboveNormal'
$trials=@()
try {
 $ready=$false
 for($i=0;$i -lt 20;$i++){
  if($server.HasExited){Get-Content -LiteralPath (Join-Path $results 'server.stderr.log');throw 'Our HTTP server exited before readiness'}
  try{$r=Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 1;if($r.Content -eq 'ok'){$ready=$true;break}}catch{}
  Start-Sleep -Milliseconds 250
 }
 if(-not $ready){throw 'Local HTTP server failed to become ready'}
 & node tools/test-http.mjs "http://127.0.0.1:$Port" $fixtures
 if($LASTEXITCODE -ne 0){throw 'HTTP contract verification failed'}
 $env:FIXTURES=$fixtures.Replace('\','/')
 $env:BASE_URL="http://127.0.0.1:$Port"
 $env:DURATION="${Seconds}s"
 $env:VUS="$VUs"
 $env:RATE='1000'
 $env:K6_NO_USAGE_REPORT='true'
 $env:K6_NEW_MACHINE_READABLE_SUMMARY='false'
 $cases=@()
 foreach($profile in $manifest){foreach($operation in @('encode','decode')){$cases+=@{profile=$profile.id;operation=$operation}}}
 foreach($operation in @('control_encode','control_decode')){$cases+=@{profile='text_1024/qsl_heap';operation=$operation}}
 for($pass=1;$pass -le $Passes+1;$pass++){
  $open=$pass -gt $Passes
  if($open -and $SkipOpen){break}
  $selected=if($open){@($cases|Where-Object {$_.operation -eq 'decode' -and $_.profile -notmatch 'dyn_' -and $_.profile -notmatch '^optionals/'})}else{@($cases)}
  if(-not $open -and $pass%2 -eq 0){[array]::Reverse($selected)}
  foreach($case in $selected){
   $model=if($open){'open'}else{'closed'}
   $name="$model-pass$pass-"+$case.operation+'-'+$case.profile.Replace('/','-')
   $env:MODEL=$model;$env:PROFILE=$case.profile;$env:OPERATION=$case.operation
   $env:SUMMARY=Join-Path $results "trials/$name.json"
   if(Test-Path -LiteralPath $env:SUMMARY){throw "Refusing to overwrite $name"}
   $before=[HttpBenchTimes]::Read();$serverBefore=[HttpBenchTimes]::CpuSeconds($serverHandle)
   $clock=[Diagnostics.Stopwatch]::StartNew()
   $client=Start-Process -FilePath $k6 -ArgumentList @('run','--quiet','--no-usage-report','k6/load.js') -WorkingDirectory $root -RedirectStandardOutput (Join-Path $results "trials/$name.stdout.log") -RedirectStandardError (Join-Path $results "trials/$name.stderr.log") -WindowStyle Hidden -PassThru
   $clientHandle=$client.Handle;$client.ProcessorAffinity=[IntPtr]11
   $client.WaitForExit();$code=[HttpBenchTimes]::ExitCode($clientHandle);$clientCpu=[HttpBenchTimes]::CpuSeconds($clientHandle)
   $clock.Stop();$after=[HttpBenchTimes]::Read();$serverCpu=[HttpBenchTimes]::CpuSeconds($serverHandle)-$serverBefore
   $active=((($after[1]-$before[1])+($after[2]-$before[2]))-($after[0]-$before[0]))/10000000.0
   $trials+=[ordered]@{name=$name;profile=$case.profile;operation=$case.operation;model=$model;pass=$pass;exitCode=$code;elapsedSeconds=$clock.Elapsed.TotalSeconds;clientCpuSeconds=$clientCpu;serverCpuSeconds=$serverCpu;otherCpuCoreEquivalent=[math]::Max([double]0,($active-$clientCpu-$serverCpu)/$clock.Elapsed.TotalSeconds)}
   $trials|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $results 'runs.json') -Encoding utf8
   Write-Output "Completed $name; exit=$code; trials=$($trials.Count)"
   if($code -ne 0){Get-Content -LiteralPath (Join-Path $results "trials/$name.stderr.log") -Tail 12;throw "k6 failed: $name"}
  }
 }
} finally {
 if(-not $server.HasExited){$server.Kill();$server.WaitForExit()}
}
