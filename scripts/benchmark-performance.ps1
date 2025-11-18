#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Comprehensive performance benchmark for QRNG-DD Gateway
    
.DESCRIPTION
    Runs various performance tests and collects metrics:
    - Throughput testing (requests/second)
    - Latency distribution (P50, P75, P90, P95, P99)
    - Concurrent client testing
    - Sustained load testing
    
.PARAMETER GatewayUrl
    Gateway URL (default: http://localhost:7764)
    
.PARAMETER ApiKey
    API key for authentication (default: test-key-1234567890)
    
.PARAMETER Duration
    Test duration in seconds (default: 600 for 10 minutes)
    
.PARAMETER Clients
    Number of concurrent clients (default: 10)
    
.PARAMETER RequestSize
    Size of random data to request in bytes (default: 1024)
    
.EXAMPLE
    .\benchmark-performance.ps1
    
.EXAMPLE
    .\benchmark-performance.ps1 -Duration 3600 -Clients 20
#>

param(
    [string]$GatewayUrl = "http://localhost:7764",
    [string]$ApiKey = "test-key-1234567890",
    [int]$Duration = 600,
    [int]$Clients = 10,
    [int]$RequestSize = 1024
)

$ErrorActionPreference = "Stop"

Write-Host "╔════════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║         QRNG-DD Gateway Performance Benchmark                      ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "  Gateway URL:      $GatewayUrl"
Write-Host "  API Key:          $($ApiKey.Substring(0, 10))..."
Write-Host "  Test Duration:    $Duration seconds"
Write-Host "  Concurrent Clients: $Clients"
Write-Host "  Request Size:     $RequestSize bytes"
Write-Host ""

# Test connectivity
Write-Host "Testing connectivity..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$GatewayUrl/health" -TimeoutSec 5
    Write-Host "✓ Gateway is reachable" -ForegroundColor Green
} catch {
    Write-Host "✗ Cannot reach gateway: $_" -ForegroundColor Red
    exit 1
}

# Check buffer status
Write-Host "`nChecking buffer status..." -ForegroundColor Yellow
$metrics = Invoke-RestMethod -Uri "$GatewayUrl/metrics" -TimeoutSec 5
$uptime = $metrics -split "`n" | Where-Object { $_ -match "qrng_uptime_seconds" } | ForEach-Object { ($_ -split " ")[1] }
Write-Host "  Gateway uptime: $uptime seconds" -ForegroundColor Cyan

# Wait for buffer to fill (at least 30%)
Write-Host "`nWaiting for buffer to reach 30%..." -ForegroundColor Yellow
$maxWait = 120
$waited = 0
while ($waited -lt $maxWait) {
    try {
        $response = Invoke-RestMethod -Uri "$GatewayUrl/api/random?bytes=1&encoding=hex" `
            -Headers @{ "Authorization" = "Bearer $ApiKey" } `
            -TimeoutSec 5
        Write-Host "✓ Buffer has sufficient data" -ForegroundColor Green
        break
    } catch {
        Write-Host "  Waiting... ($waited/$maxWait seconds)" -ForegroundColor Gray
        Start-Sleep -Seconds 5
        $waited += 5
    }
}

if ($waited -ge $maxWait) {
    Write-Host "✗ Buffer did not fill in time. Proceeding anyway..." -ForegroundColor Yellow
}

# Prepare results directory
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$resultsDir = ".\benchmark_results_$timestamp"
New-Item -ItemType Directory -Path $resultsDir -Force | Out-Null
Write-Host "`nResults will be saved to: $resultsDir" -ForegroundColor Cyan

# Benchmark function
function Invoke-BenchmarkRequest {
    param(
        [string]$Url,
        [string]$Key,
        [int]$Size
    )
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $response = Invoke-RestMethod -Uri "$Url/api/random?bytes=$Size&encoding=hex" `
            -Headers @{ "Authorization" = "Bearer $Key" } `
            -TimeoutSec 30
        $sw.Stop()
        return @{
            Success = $true
            LatencyMs = $sw.Elapsed.TotalMilliseconds
            Bytes = $Size
        }
    } catch {
        $sw.Stop()
        return @{
            Success = $false
            LatencyMs = $sw.Elapsed.TotalMilliseconds
            Error = $_.Exception.Message
        }
    }
}

# ============================================================================
# Test 1: Single-client throughput baseline
# ============================================================================
Write-Host "`n" + "="*70 -ForegroundColor Cyan
Write-Host "Test 1: Single-client baseline (60 seconds)" -ForegroundColor Cyan
Write-Host "="*70 -ForegroundColor Cyan

$results = @()
$testDuration = 60
$startTime = Get-Date
$endTime = $startTime.AddSeconds($testDuration)

Write-Host "Running..." -ForegroundColor Yellow
while ((Get-Date) -lt $endTime) {
    $result = Invoke-BenchmarkRequest -Url $GatewayUrl -Key $ApiKey -Size $RequestSize
    $results += $result
}

$successCount = ($results | Where-Object { $_.Success }).Count
$failCount = $results.Count - $successCount
$totalBytes = ($results | Where-Object { $_.Success } | Measure-Object -Property Bytes -Sum).Sum
$latencies = $results | Where-Object { $_.Success } | ForEach-Object { $_.LatencyMs }
$throughput = $successCount / $testDuration

Write-Host "`nResults:" -ForegroundColor Green
Write-Host "  Total requests:    $($results.Count)"
Write-Host "  Successful:        $successCount"
Write-Host "  Failed:            $failCount"
Write-Host "  Throughput:        $([math]::Round($throughput, 2)) req/s"
Write-Host "  Total bytes:       $totalBytes"
Write-Host "  Data rate:         $([math]::Round($totalBytes / $testDuration / 1024, 2)) KB/s"

if ($latencies.Count -gt 0) {
    $sortedLatencies = $latencies | Sort-Object
    $p50 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.50)]
    $p75 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.75)]
    $p90 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.90)]
    $p95 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.95)]
    $p99 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.99)]
    
    Write-Host "`nLatency distribution:" -ForegroundColor Yellow
    Write-Host "  P50:               $([math]::Round($p50, 2)) ms"
    Write-Host "  P75:               $([math]::Round($p75, 2)) ms"
    Write-Host "  P90:               $([math]::Round($p90, 2)) ms"
    Write-Host "  P95:               $([math]::Round($p95, 2)) ms"
    Write-Host "  P99:               $([math]::Round($p99, 2)) ms"
}

# Save results
$results | ConvertTo-Json | Out-File "$resultsDir\test1_single_client.json"

# ============================================================================
# Test 2: Concurrent clients
# ============================================================================
Write-Host "`n" + "="*70 -ForegroundColor Cyan
Write-Host "Test 2: Concurrent clients ($Clients clients, $Duration seconds)" -ForegroundColor Cyan
Write-Host "="*70 -ForegroundColor Cyan

$script:concurrentResults = [System.Collections.Concurrent.ConcurrentBag[object]]::new()
$script:running = $true

$jobs = 1..$Clients | ForEach-Object {
    Start-Job -ScriptBlock {
        param($Url, $Key, $Size, $Duration)
        
        $results = @()
        $endTime = (Get-Date).AddSeconds($Duration)
        
        while ((Get-Date) -lt $endTime) {
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            try {
                $response = Invoke-RestMethod -Uri "$Url/api/random?bytes=$Size&encoding=hex" `
                    -Headers @{ "Authorization" = "Bearer $Key" } `
                    -TimeoutSec 30
                $sw.Stop()
                $results += @{
                    Success = $true
                    LatencyMs = $sw.Elapsed.TotalMilliseconds
                    Bytes = $Size
                }
            } catch {
                $sw.Stop()
                $results += @{
                    Success = $false
                    LatencyMs = $sw.Elapsed.TotalMilliseconds
                    Error = $_.Exception.Message
                }
            }
        }
        
        return $results
    } -ArgumentList $GatewayUrl, $ApiKey, $RequestSize, $Duration
}

Write-Host "Running $Clients concurrent clients for $Duration seconds..." -ForegroundColor Yellow
Write-Host "This may take a while..." -ForegroundColor Gray

# Progress indicator
$progressInterval = [math]::Max([int]($Duration / 20), 1)
for ($i = 0; $i -lt $Duration; $i += $progressInterval) {
    Start-Sleep -Seconds $progressInterval
    $pct = [math]::Round(($i / $Duration) * 100, 0)
    Write-Host "  Progress: $pct% ($i/$Duration seconds)" -ForegroundColor Gray
}

# Wait for all jobs to complete
Write-Host "Waiting for jobs to complete..." -ForegroundColor Yellow
$allResults = $jobs | Wait-Job | Receive-Job
$jobs | Remove-Job

# Flatten results
$flatResults = @()
foreach ($clientResults in $allResults) {
    $flatResults += $clientResults
}

$successCount = ($flatResults | Where-Object { $_.Success }).Count
$failCount = $flatResults.Count - $successCount
$totalBytes = ($flatResults | Where-Object { $_.Success } | Measure-Object -Property Bytes -Sum).Sum
$latencies = $flatResults | Where-Object { $_.Success } | ForEach-Object { $_.LatencyMs }
$throughput = $successCount / $Duration

Write-Host "`nResults:" -ForegroundColor Green
Write-Host "  Total requests:    $($flatResults.Count)"
Write-Host "  Successful:        $successCount"
Write-Host "  Failed:            $failCount"
Write-Host "  Throughput:        $([math]::Round($throughput, 2)) req/s"
Write-Host "  Total bytes:       $totalBytes"
Write-Host "  Data rate:         $([math]::Round($totalBytes / $Duration / 1024, 2)) KB/s"

if ($latencies.Count -gt 0) {
    $sortedLatencies = $latencies | Sort-Object
    $p50 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.50)]
    $p75 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.75)]
    $p90 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.90)]
    $p95 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.95)]
    $p99 = $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.99)]
    
    Write-Host "`nLatency distribution:" -ForegroundColor Yellow
    Write-Host "  P50:               $([math]::Round($p50, 2)) ms"
    Write-Host "  P75:               $([math]::Round($p75, 2)) ms"
    Write-Host "  P90:               $([math]::Round($p90, 2)) ms"
    Write-Host "  P95:               $([math]::Round($p95, 2)) ms"
    Write-Host "  P99:               $([math]::Round($p99, 2)) ms"
    Write-Host "  Min:               $([math]::Round(($sortedLatencies | Measure-Object -Minimum).Minimum, 2)) ms"
    Write-Host "  Max:               $([math]::Round(($sortedLatencies | Measure-Object -Maximum).Maximum, 2)) ms"
    Write-Host "  Mean:              $([math]::Round(($sortedLatencies | Measure-Object -Average).Average, 2)) ms"
}

# Save results
$flatResults | ConvertTo-Json | Out-File "$resultsDir\test2_concurrent_clients.json"

# ============================================================================
# Test 3: Get final metrics
# ============================================================================
Write-Host "`n" + "="*70 -ForegroundColor Cyan
Write-Host "Final Prometheus Metrics" -ForegroundColor Cyan
Write-Host "="*70 -ForegroundColor Cyan

$finalMetrics = Invoke-RestMethod -Uri "$GatewayUrl/metrics"
$finalMetrics | Out-File "$resultsDir\final_metrics.txt"
Write-Host $finalMetrics

# ============================================================================
# Summary Report
# ============================================================================
Write-Host "`n" + "="*70 -ForegroundColor Cyan
Write-Host "BENCHMARK SUMMARY" -ForegroundColor Cyan
Write-Host "="*70 -ForegroundColor Cyan

$summary = @{
    Timestamp = $timestamp
    Configuration = @{
        GatewayUrl = $GatewayUrl
        Duration = $Duration
        Clients = $Clients
        RequestSize = $RequestSize
    }
    Test1_SingleClient = @{
        Throughput = [math]::Round($throughput, 2)
        LatencyP50 = [math]::Round($p50, 2)
        LatencyP95 = [math]::Round($p95, 2)
        LatencyP99 = [math]::Round($p99, 2)
    }
    Test2_ConcurrentClients = @{
        Throughput = [math]::Round($throughput, 2)
        LatencyP50 = [math]::Round($p50, 2)
        LatencyP95 = [math]::Round($p95, 2)
        LatencyP99 = [math]::Round($p99, 2)
    }
}

$summary | ConvertTo-Json -Depth 10 | Out-File "$resultsDir\summary.json"
Write-Host ($summary | ConvertTo-Json -Depth 10) -ForegroundColor Cyan

Write-Host "`n✓ Benchmark complete!" -ForegroundColor Green
Write-Host "  Results saved to: $resultsDir" -ForegroundColor Cyan
