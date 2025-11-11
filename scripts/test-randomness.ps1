#!/usr/bin/env pwsh
# QRNG Data Diode - Randomness Quality Test Script
#
# This script waits for the buffer to fill to 100% capacity, then consumes the entire
# buffer in one burst to perform maximum iterations of Monte Carlo π estimation.
#
# This provides the most rigorous randomness quality test possible.

param(
    [string]$GatewayUrl = "http://localhost:8080",
    [string]$ApiKey = "test-key-1234567890",
    [int]$PollIntervalSeconds = 5,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Info { param([string]$Message) Write-Host $Message -ForegroundColor Cyan }
function Write-Success { param([string]$Message) Write-Host "✓ $Message" -ForegroundColor Green }
function Write-Error { param([string]$Message) Write-Host "✗ $Message" -ForegroundColor Red }
function Write-Metric { param([string]$Name, [string]$Value) Write-Host "  ${Name}: " -NoNewline -ForegroundColor Yellow; Write-Host $Value }

Write-Info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Info "  QRNG Data Diode - Randomness Quality Test"
Write-Info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host ""

# Check if gateway is accessible
Write-Info "Checking gateway connectivity..."
try {
    $response = Invoke-WebRequest -Uri "$GatewayUrl/health" -Method Get -UseBasicParsing -ErrorAction Stop
    Write-Success "Gateway is online"
} catch {
    if ($_.Exception.Response.StatusCode -eq 503) {
        Write-Host "  Gateway is online but buffer is not ready (< 5% full)" -ForegroundColor Yellow
        Write-Host "  Continuing anyway - some endpoints may still work..." -ForegroundColor Gray
    } else {
        Write-Error "Gateway is not accessible at $GatewayUrl"
        Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

# Get system status
Write-Host ""
Write-Info "Fetching system status..."
try {
    $headers = @{ "Authorization" = "Bearer $ApiKey" }
    $status = Invoke-RestMethod -Uri "$GatewayUrl/api/status" -Headers $headers -Method Get

    Write-Success "System Status:"
    Write-Metric "Status" $status.status
    Write-Metric "Buffer Fill" "$([math]::Round($status.buffer_fill_percent, 2))%"
    Write-Metric "Buffer Available" "$($status.buffer_bytes_available) bytes"
    Write-Metric "Uptime" "$($status.uptime_seconds) seconds"
    Write-Metric "Requests Served" $status.total_requests_served
    Write-Metric "Bytes Served" "$($status.total_bytes_served) bytes"
    Write-Metric "Requests/sec" "$([math]::Round($status.requests_per_second, 2))"

    if ($status.warnings -and $status.warnings.Count -gt 0) {
        Write-Host "  Warnings:" -ForegroundColor Yellow
        foreach ($warning in $status.warnings) {
            Write-Host "    - $warning" -ForegroundColor Yellow
        }
    }

    # Wait for buffer to fill to maximum capacity
    Write-Host ""
    Write-Info "Waiting for buffer to reach 100% capacity..."
    $startTime = Get-Date
    while ($status.buffer_fill_percent -lt 99.9) {
        $elapsed = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 0)
        Write-Host "`r  Buffer: $([math]::Round($status.buffer_fill_percent, 2))% | Waiting ${elapsed}s..." -NoNewline -ForegroundColor Cyan
        Start-Sleep -Seconds $PollIntervalSeconds
        $status = Invoke-RestMethod -Uri "$GatewayUrl/api/status" -Headers $headers -Method Get
    }
    Write-Host ""
    Write-Success "Buffer is full! Available: $($status.buffer_bytes_available) bytes"

    # Calculate maximum iterations based on full buffer capacity
    $bytesPerIteration = 8
    $Iterations = [math]::Floor($status.buffer_bytes_available / $bytesPerIteration)
    Write-Metric "Maximum Iterations" "$Iterations (consuming entire buffer in one burst)"

} catch {
    Write-Error "Failed to fetch system status"
    Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Fetch sample random data
Write-Host ""
Write-Info "Fetching sample random data (32 bytes)..."
try {
    $url = "$GatewayUrl/api/random?bytes=32&encoding=hex&api_key=$ApiKey"
    $response = Invoke-WebRequest -Uri $url -Method Get -UseBasicParsing
    $randomData = $response.Content
    Write-Success "Random data retrieved"
    Write-Host "  Sample: $($randomData.Substring(0, [Math]::Min(64, $randomData.Length)))..." -ForegroundColor Gray
} catch {
    Write-Error "Failed to fetch random data"
    Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Run Monte Carlo π estimation test
Write-Host ""
Write-Info "Running Monte Carlo π estimation test with MAXIMUM iterations..."
Write-Host "  Iterations: $Iterations" -ForegroundColor Gray
Write-Host "  Consuming: $([math]::Round($Iterations * 8 / 1024 / 1024, 2)) MB from the entropy buffer" -ForegroundColor Gray
Write-Host ""

try {
    $url = "$GatewayUrl/api/test/monte-carlo?iterations=$Iterations&api_key=$ApiKey"
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-RestMethod -Uri $url -Method Post
    $stopwatch.Stop()

    Write-Success "Monte Carlo test completed in $($stopwatch.ElapsedMilliseconds)ms"
    Write-Host ""

    # Display results
    $piActual = [Math]::PI
    $errorPercent = [math]::Round($result.error_percent, 4)

    Write-Info "Results:"
    Write-Metric "Estimated π" "$([math]::Round($result.estimated_pi, 10))"
    Write-Metric "Actual π" "$([math]::Round($piActual, 10))"
    Write-Metric "Error" "$([math]::Round($result.error, 10))"
    Write-Metric "Error %" "$errorPercent%"
    Write-Metric "Convergence" $result.convergence_rate.ToUpper()

    # Color-code convergence
    if ($result.convergence_rate -eq "excellent") {
        Write-Host "  Quality: " -NoNewline -ForegroundColor Yellow
        Write-Host "EXCELLENT ★★★★★" -ForegroundColor Green
    } elseif ($result.convergence_rate -eq "good") {
        Write-Host "  Quality: " -NoNewline -ForegroundColor Yellow
        Write-Host "GOOD ★★★★☆" -ForegroundColor Green
    } elseif ($result.convergence_rate -eq "fair") {
        Write-Host "  Quality: " -NoNewline -ForegroundColor Yellow
        Write-Host "FAIR ★★★☆☆" -ForegroundColor Yellow
    } else {
        Write-Host "  Quality: " -NoNewline -ForegroundColor Yellow
        Write-Host "POOR ★★☆☆☆" -ForegroundColor Red
    }

    # Show comparison with pseudo-random if available
    if ($result.quantum_vs_pseudo) {
        Write-Host ""
        Write-Info "Quantum vs Pseudo-Random Comparison:"
        $qError = [math]::Round($result.quantum_vs_pseudo.quantum_error, 10)
        $pError = [math]::Round($result.quantum_vs_pseudo.pseudo_error, 10)
        $improvement = [math]::Round($result.quantum_vs_pseudo.improvement_factor, 2)

        Write-Metric "Quantum Error" $qError
        Write-Metric "Pseudo Error" $pError
        Write-Metric "Improvement Factor" "${improvement}x"

        if ($improvement -gt 1) {
            Write-Host "  Quantum randomness is " -NoNewline -ForegroundColor Gray
            Write-Host "${improvement}x better " -NoNewline -ForegroundColor Green
            Write-Host "than pseudo-random!" -ForegroundColor Gray
        } elseif ($improvement -lt 1) {
            Write-Host "  Pseudo-random performed better in this test" -ForegroundColor Yellow
        } else {
            Write-Host "  Similar performance to pseudo-random" -ForegroundColor Gray
        }
    }

    # Interpretation
    Write-Host ""
    Write-Info "Interpretation:"
    if ($errorPercent -lt 0.01) {
        Write-Host "  The quantum entropy source is producing high-quality random data" -ForegroundColor Green
        Write-Host "  with excellent statistical properties. Error is less than 0.01%." -ForegroundColor Green
    } elseif ($errorPercent -lt 0.1) {
        Write-Host "  The quantum entropy source is producing good random data" -ForegroundColor Green
        Write-Host "  suitable for most cryptographic applications." -ForegroundColor Green
    } elseif ($errorPercent -lt 1.0) {
        Write-Host "  The quantum entropy source is producing acceptable random data" -ForegroundColor Yellow
        Write-Host "  but quality could be improved. Consider running more iterations." -ForegroundColor Yellow
    } else {
        Write-Host "  The random data quality is below expected thresholds." -ForegroundColor Red
        Write-Host "  This may indicate issues with the entropy source or mixing." -ForegroundColor Red
    }

} catch {
    Write-Error "Monte Carlo test failed"
    Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Red

    if ($Verbose) {
        Write-Host "  Response: $($_.ErrorDetails)" -ForegroundColor Gray
    }

    exit 1
}

# Summary
Write-Host ""
Write-Info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Success "Test completed successfully!"
Write-Info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host ""
