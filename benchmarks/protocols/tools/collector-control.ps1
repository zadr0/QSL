param([string]$Output='',[switch]$SkipIdleGate)
$ErrorActionPreference='Stop'
if(-not $Output){$Output=Join-Path $PSScriptRoot ('../results/collector-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))}
$filter='^codec/mixed/read/b16/qsl_optimized$'
foreach($mode in @('off-before','on','off-after')){
 $args=@('-NoProfile','-File',(Join-Path $PSScriptRoot 'run.ps1'),'-Filter',$filter,'-Output',(Join-Path $Output $mode))
 if($SkipIdleGate){$args+='-SkipIdleGate'}
 if($mode -eq 'on'){$args+='-Sampler'}
 & pwsh @args
 if($LASTEXITCODE -ne 0){throw "Collector control stopped at $mode"}
}
Write-Output 'A/B/A finished. Diagnostic only: inspect drift and marginal intervals; do not subtract overhead from main results.'
