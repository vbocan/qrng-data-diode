#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Quick burst test (no think time) to measure maximum gateway throughput
#>

param(
    [string]$GatewayUrl = "http://localhost:7764",
    [string]$ApiKey = "test-key-1234567890",
    [int]$DurationSeconds = 30,
    [int]$RequestSize = 1024
)

Write-Host "Burst Test (30 seconds, no throttling)" -ForegroundColor Cyan
Write-Host "Waiting 30s for buffer to fill..."
Start-Sleep -Seconds 30

$results = @{
    Latencies = @()
    Successes = 0
    Failures = 0
}

$startTime = Get-Date
$endTime = $startTime.AddSeconds($DurationSeconds)

Write-Host "Starting burst test..."

while ((Get-Date) -lt $endTime) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $response = Invoke-RestMethod -Uri "$GatewayUrl/api/random?bytes=$RequestSize&encoding=hex" `
            -Headers @{ "Authorization" = "Bearer $ApiKey" } `
            -TimeoutSec 5
        $sw.Stop()
        $results.Successes++
        $results.Latencies += $sw.Elapsed.TotalMilliseconds
    } catch {
        $sw.Stop()
        $results.Failures++
    }
}

$totalTime = ((Get-Date) - $startTime).TotalSeconds
$total = $results.Successes + $results.Failures

Write-Host ""
Write-Host "Results:" -ForegroundColor Green
Write-Host "  Duration:    $([math]::Round($totalTime, 2))s"
Write-Host "  Total:       $total"
Write-Host "  Successful:  $($results.Successes) ($([math]::Round(($results.Successes/$total)*100, 1))%)"
Write-Host "  Failed:      $($results.Failures) ($([math]::Round(($results.Failures/$total)*100, 1))%)"
Write-Host "  Throughput:  $([math]::Round($results.Successes / $totalTime, 2)) req/s"

if ($results.Latencies.Count -gt 10) {
    $sorted = $results.Latencies | Sort-Object
    Write-Host ""
    Write-Host "Latency:" -ForegroundColor Yellow
    Write-Host "  P50: $([math]::Round($sorted[[math]::Floor($sorted.Count * 0.50)], 2)) ms"
    Write-Host "  P95: $([math]::Round($sorted[[math]::Floor($sorted.Count * 0.95)], 2)) ms"
    Write-Host "  P99: $([math]::Round($sorted[[math]::Floor($sorted.Count * 0.99)], 2)) ms"
}
