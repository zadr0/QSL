param([int]$MaxHours=4,[string]$Python='python',[switch]$SkipIdleGate)
$ErrorActionPreference='Stop'
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'));$state=Join-Path $project 'results/background-state.json'
if(Test-Path -LiteralPath $state){$old=Get-Content $state -Raw|ConvertFrom-Json;if($old.phase -in 'waiting_or_measuring','diagnostics' -and (Get-Process -Id $old.pid -ErrorAction SilentlyContinue)){throw 'A background pass is already active'}}
if(Test-Path -LiteralPath (Join-Path $project 'results/STOP')){throw 'Remove results/STOP explicitly before starting'}
[IO.Directory]::CreateDirectory((Join-Path $project 'results'))|Out-Null
$log=Join-Path $project ('results/background-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
$args=@('-NoProfile','-File',('"'+(Join-Path $PSScriptRoot 'wait-and-run.ps1')+'"'),'-MaxHours',$MaxHours,'-Python',('"'+$Python+'"'))
if($SkipIdleGate){$args+='-SkipIdleGate'}
$p=Start-Process -FilePath (Get-Command pwsh).Source -ArgumentList $args -WorkingDirectory $project -WindowStyle Hidden -RedirectStandardOutput ($log+'.log') -RedirectStandardError ($log+'.error.log') -PassThru
@{pid=$p.Id;log=$log;stateFile=$state;stopFile=(Join-Path $project 'results/STOP');maxHours=$MaxHours}|ConvertTo-Json
