#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Simple sustained throughput test for QRNG-DD Gateway
    
.DESCRIPTION
    Tests sustained throughput by making continuous requests
    and measuring success rate and latencies.
#>

param(
    [string]$GatewayUrl = "http://localhost:7764",
    [string]$ApiKey = "test-key-1234567890",
    [int]$DurationSeconds = 600,
    [int]$RequestSize = 1024,
    [int]$ThinkTimeMs = 50  # Delay between requests to avoid exhausting buffer
)

$ErrorActionPreference = "Stop"

Write-Host "╔════════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║      QRNG-DD Sustained Throughput Test                             ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "  Gateway URL:      $GatewayUrl"
Write-Host "  Duration:         $DurationSeconds seconds"
Write-Host "  Request Size:     $RequestSize bytes"
Write-Host "  Think Time:       $ThinkTimeMs ms"
Write-Host ""

# Wait for buffer
Write-Host "Waiting 60 seconds for buffer to fill..." -ForegroundColor Yellow
Start-Sleep -Seconds 60

$results = @{
    Latencies = @()
    Successes = 0
    Failures = 0
    Errors = @{}
}

$startTime = Get-Date
$endTime = $startTime.AddSeconds($DurationSeconds)
$lastReport = $startTime
$requestCount = 0

Write-Host "Starting test..." -ForegroundColor Green
Write-Host ""

while ((Get-Date) -lt $endTime) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $response = Invoke-RestMethod -Uri "$GatewayUrl/api/random?bytes=$RequestSize&encoding=hex" `
            -Headers @{ "Authorization" = "Bearer $ApiKey" } `
            -TimeoutSec 30
        $sw.Stop()
        $results.Successes++
        $results.Latencies += $sw.Elapsed.TotalMilliseconds
    } catch {
        $sw.Stop()
        $results.Failures++
        $errorMsg = $_.Exception.Message
        if ($results.Errors.ContainsKey($errorMsg)) {
            $results.Errors[$errorMsg]++
        } else {
            $results.Errors[$errorMsg] = 1
        }
    }
    
    $requestCount++
    
    # Progress report every 30 seconds
    $now = Get-Date
    if (($now - $lastReport).TotalSeconds -ge 30) {
        $elapsed = ($now - $startTime).TotalSeconds
        $throughput = [math]::Round($results.Successes / $elapsed, 2)
        $successRate = [math]::Round(($results.Successes / $requestCount) * 100, 2)
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Elapsed: $([math]::Round($elapsed))s | Requests: $requestCount | Success: $successRate% | Throughput: $throughput req/s" -ForegroundColor Cyan
        $lastReport = $now
    }
    
    if ($ThinkTimeMs -gt 0) {
        Start-Sleep -Milliseconds $ThinkTimeMs
    }
}

# Final results
$totalTime = ((Get-Date) - $startTime).TotalSeconds
Write-Host ""
Write-Host "="*70 -ForegroundColor Green
Write-Host "RESULTS" -ForegroundColor Green
Write-Host "="*70 -ForegroundColor Green
Write-Host ""
Write-Host "Test Duration:     $([math]::Round($totalTime, 2)) seconds" -ForegroundColor Yellow
Write-Host "Total Requests:    $requestCount"
Write-Host "Successful:        $($results.Successes) ($([math]::Round(($results.Successes/$requestCount)*100, 2))%)"
Write-Host "Failed:            $($results.Failures) ($([math]::Round(($results.Failures/$requestCount)*100, 2))%)"
Write-Host ""
Write-Host "Throughput:        $([math]::Round($results.Successes / $totalTime, 2)) req/s" -ForegroundColor Cyan
Write-Host "Data Rate:         $([math]::Round($results.Successes * $RequestSize / $totalTime / 1024, 2)) KB/s" -ForegroundColor Cyan
Write-Host ""

if ($results.Latencies.Count -gt 0) {
    $sorted = $results.Latencies | Sort-Object
    $p50 = $sorted[[math]::Floor($sorted.Count * 0.50)]
    $p75 = $sorted[[math]::Floor($sorted.Count * 0.75)]
    $p90 = $sorted[[math]::Floor($sorted.Count * 0.90)]
    $p95 = $sorted[[math]::Floor($sorted.Count * 0.95)]
    $p99 = $sorted[[math]::Floor($sorted.Count * 0.99)]
    $min = ($sorted | Measure-Object -Minimum).Minimum
    $max = ($sorted | Measure-Object -Maximum).Maximum
    $avg = ($sorted | Measure-Object -Average).Average
    
    Write-Host "Latency Distribution:" -ForegroundColor Yellow
    Write-Host "  Min:    $([math]::Round($min, 2)) ms"
    Write-Host "  P50:    $([math]::Round($p50, 2)) ms"
    Write-Host "  P75:    $([math]::Round($p75, 2)) ms"
    Write-Host "  P90:    $([math]::Round($p90, 2)) ms"
    Write-Host "  P95:    $([math]::Round($p95, 2)) ms"
    Write-Host "  P99:    $([math]::Round($p99, 2)) ms"
    Write-Host "  Max:    $([math]::Round($max, 2)) ms"
    Write-Host "  Mean:   $([math]::Round($avg, 2)) ms"
}

if ($results.Errors.Count -gt 0) {
    Write-Host ""
    Write-Host "Error Summary:" -ForegroundColor Red
    foreach ($error in $results.Errors.GetEnumerator()) {
        Write-Host "  $($error.Value)x: $($error.Key)"
    }
}

# Get final metrics
Write-Host ""
Write-Host "="*70 -ForegroundColor Yellow
Write-Host "PROMETHEUS METRICS" -ForegroundColor Yellow
Write-Host "="*70 -ForegroundColor Yellow
$metrics = Invoke-RestMethod -Uri "$GatewayUrl/metrics"
Write-Host $metrics

Write-Host ""
Write-Host "✓ Test complete!" -ForegroundColor Green
