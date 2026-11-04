param([int]$MaxHours=4,[string]$Python='python')
$ErrorActionPreference='Stop'
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'));Set-Location -LiteralPath $project
$start=[Diagnostics.Stopwatch]::StartNew();$limit=$MaxHours*3600;$stateFile=Join-Path $project 'results/background-state.json';$stop=Join-Path $project 'results/STOP';$pass=0
function State($phase,$message){@{phase=$phase;message=$message;pid=$PID;updated=(Get-Date).ToUniversalTime().ToString('o');elapsedSeconds=$start.Elapsed.TotalSeconds;maxHours=$MaxHours}|ConvertTo-Json|Set-Content -LiteralPath $stateFile}
try{
 [IO.Directory]::CreateDirectory((Join-Path $project 'results'))|Out-Null
 $initial=@{};foreach($file in Get-ChildItem '../../qsl/src','../../qsl/macros/src','src','schemas','benches' -Recurse -File){$initial[$file.FullName]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash}
 $initialFiles=@('Cargo.toml','Cargo.lock','matrix.json','../../qsl/Cargo.toml','../../qsl/macros/Cargo.toml');foreach($file in Get-Item $initialFiles){$initial[$file.FullName]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash}
 $matrix=Get-Content 'matrix.json' -Raw|ConvertFrom-Json
 while($start.Elapsed.TotalSeconds -lt $limit -and -not(Test-Path -LiteralPath $stop)){
  foreach($path in $initial.Keys){if((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $initial[$path]){throw 'Sources changed while waiting; rebuild/restart explicitly'}}
  & $Python -B tools/build-report.py
  if($LASTEXITCODE -ne 0){throw 'Report generation failed'}
  $report=Get-Content report/data.json -Raw|ConvertFrom-Json
  $done=@($report.rows|Where-Object{$_.eligible}|ForEach-Object{$_.case})
  $remaining=@($matrix.cases|Where-Object{$_ -notin $done})
  if($remaining.Count -eq 0){break}
  $pass++;$output=Join-Path $project ('results/strict-wait-'+(Get-Date -Format 'yyyyMMdd-HHmmss')+'-p'+$pass)
  $filter='^('+((@($remaining|ForEach-Object{[Regex]::Escape($_)})) -join '|')+')$'
  State 'waiting_or_measuring' ("Pass $pass; $($done.Count)/100 accepted. Idle gate and native benchmark run without GUI updates.")
  $seconds=[Math]::Max(1,[int]($limit-$start.Elapsed.TotalSeconds))
  & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'run.ps1') -Output $output -Filter $filter -IdleTimeoutSeconds 600 -MaxDurationSeconds $seconds -StopFile $stop
  # A failed idle gate retains evidence and retries until the global deadline. No failed observation is silently accepted.
  & $Python -B tools/build-report.py
  if($LASTEXITCODE -ne 0){throw 'Report generation failed'}
  if((Test-Path -LiteralPath $stop) -or $start.Elapsed.TotalSeconds -ge $limit){break}
  Start-Sleep -Seconds 10
 }
 $report=Get-Content report/data.json -Raw|ConvertFrom-Json
 if($report.accepted -eq $report.expected){
  State 'diagnostics' '100/100 qualified; collecting separate RTT and collector controls.'
  & pwsh -NoProfile -File tools/probe.ps1 -Program latency_probe -Output ('results/latency-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
  if(-not(Test-Path -LiteralPath $stop) -and $start.Elapsed.TotalSeconds -lt $limit){& pwsh -NoProfile -File tools/collector-control.ps1}
 }
 & $Python -B tools/build-report.py
 if($LASTEXITCODE -ne 0){throw 'Final report failed'}
 $report=Get-Content report/data.json -Raw|ConvertFrom-Json
 State $(if($report.accepted -eq $report.expected){'complete'}elseif(Test-Path -LiteralPath $stop){'stopped'}else{'idle_or_noise_deadline'}) ("$($report.accepted)/100 qualified. Local report refreshed; rejected data retained. No automatic Git publication.")
}catch{State 'error' $_.Exception.Message;throw}
