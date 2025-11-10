#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Consume random data from the QRNG Gateway API

.DESCRIPTION
    This script demonstrates how to consume quantum random numbers from the
    QRNG Gateway. It supports different output formats (hex, base64, binary)
    and can optionally save the data to a file.

.PARAMETER GatewayUrl
    The base URL of the QRNG Gateway (default: http://localhost:8080)

.PARAMETER ApiKey
    The API key for authentication (default: test-key-1234567890)

.PARAMETER Bytes
    Number of random bytes to request (default: 32, max: 65536)

.PARAMETER Encoding
    Output encoding format: hex, base64, or binary (default: hex)

.PARAMETER OutputFile
    Optional file path to save the random data

.PARAMETER Requests
    Number of requests to make (default: 1)

.PARAMETER Delay
    Delay between requests in milliseconds (default: 100)

.PARAMETER ShowStats
    Display statistics about the received data

.EXAMPLE
    .\consume-random.ps1
    Get 32 bytes in hex format

.EXAMPLE
    .\consume-random.ps1 -Bytes 1024 -Encoding base64
    Get 1024 bytes in base64 format

.EXAMPLE
    .\consume-random.ps1 -Bytes 256 -OutputFile random.bin -Encoding binary
    Get 256 bytes and save to file

.EXAMPLE
    .\consume-random.ps1 -Requests 10 -Delay 500 -ShowStats
    Make 10 requests with statistics

.NOTES
    Author: Valer BOCAN, PhD, CSSLP - www.bocan.ro
    Project: QRNG Data Diode
#>

param(
    [string]$GatewayUrl = "http://localhost:8080",
    [string]$ApiKey = "test-key-1234567890",
    [ValidateRange(1, 65536)]
    [int]$Bytes = 32,
    [ValidateSet("hex", "base64", "binary")]
    [string]$Encoding = "hex",
    [string]$OutputFile = "",
    [ValidateRange(1, 1000)]
    [int]$Requests = 1,
    [ValidateRange(0, 10000)]
    [int]$Delay = 100,
    [switch]$ShowStats
)

# Colors for output
$script:Colors = @{
    Success = "Green"
    Error   = "Red"
    Warning = "Yellow"
    Info    = "Cyan"
    Data    = "White"
}

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Get-RandomData {
    param(
        [string]$Url,
        [string]$Key,
        [int]$ByteCount,
        [string]$Format
    )
    
    $endpoint = "$Url/api/random?bytes=$ByteCount&encoding=$Format&api_key=$Key"
    
    try {
        $response = Invoke-WebRequest -Uri $endpoint -Method Get -UseBasicParsing -ErrorAction Stop
        
        return @{
            Success = $true
            Data = $response.Content
            StatusCode = $response.StatusCode
            Length = $response.Content.Length
        }
    }
    catch {
        return @{
            Success = $false
            Error = $_.Exception.Message
            StatusCode = $_.Exception.Response.StatusCode.Value__
        }
    }
}

function Get-DataStats {
    param([string]$Data)
    
    if ($Encoding -eq "hex") {
        # Convert hex to bytes for analysis
        $hexBytes = $Data -replace '[^0-9A-Fa-f]', ''
        $byteArray = [byte[]]::new($hexBytes.Length / 2)
        for ($i = 0; $i -lt $hexBytes.Length; $i += 2) {
            $byteArray[$i / 2] = [Convert]::ToByte($hexBytes.Substring($i, 2), 16)
        }
    }
    elseif ($Encoding -eq "base64") {
        $byteArray = [Convert]::FromBase64String($Data)
    }
    else {
        $byteArray = [System.Text.Encoding]::UTF8.GetBytes($Data)
    }
    
    # Calculate basic statistics
    $sum = 0
    $min = 255
    $max = 0
    
    foreach ($byte in $byteArray) {
        $sum += $byte
        if ($byte -lt $min) { $min = $byte }
        if ($byte -gt $max) { $max = $byte }
    }
    
    $mean = $sum / $byteArray.Length
    
    # Count unique bytes
    $unique = ($byteArray | Select-Object -Unique).Count
    
    return @{
        Length = $byteArray.Length
        Mean = [math]::Round($mean, 2)
        Min = $min
        Max = $max
        Unique = $unique
        Entropy = [math]::Round(($unique / 256.0) * 100, 2)
    }
}

function Save-ToFile {
    param(
        [string]$Data,
        [string]$Path,
        [string]$Format
    )
    
    try {
        if ($Format -eq "binary") {
            [System.IO.File]::WriteAllBytes($Path, [System.Text.Encoding]::UTF8.GetBytes($Data))
        }
        else {
            Set-Content -Path $Path -Value $Data -NoNewline
        }
        return $true
    }
    catch {
        Write-ColorOutput "Error saving file: $_" -Color $Colors.Error
        return $false
    }
}

# Main execution
Write-ColorOutput "`n=== QRNG Gateway Random Data Consumer ===" -Color $Colors.Info
Write-ColorOutput "Gateway: $GatewayUrl" -Color $Colors.Info
Write-ColorOutput "Requests: $Requests | Bytes per request: $Bytes | Encoding: $Encoding`n" -Color $Colors.Info

$totalBytes = 0
$successCount = 0
$failCount = 0
$startTime = Get-Date
$allData = @()

for ($i = 1; $i -le $Requests; $i++) {
    Write-ColorOutput "[$i/$Requests] Requesting $Bytes bytes..." -Color $Colors.Info
    
    $result = Get-RandomData -Url $GatewayUrl -Key $ApiKey -ByteCount $Bytes -Format $Encoding
    
    if ($result.Success) {
        $successCount++
        $totalBytes += $result.Length
        $allData += $result.Data
        
        Write-ColorOutput "  ✓ Success (HTTP $($result.StatusCode)) - Received $($result.Length) bytes" -Color $Colors.Success
        
        if ($Requests -eq 1 -or $ShowStats) {
            # Show preview of data
            $preview = if ($result.Data.Length -gt 100) {
                $result.Data.Substring(0, 100) + "..."
            } else {
                $result.Data
            }
            Write-ColorOutput "  Data: $preview" -Color $Colors.Data
        }
    }
    else {
        $failCount++
        Write-ColorOutput "  ✗ Failed (HTTP $($result.StatusCode)) - $($result.Error)" -Color $Colors.Error
    }
    
    if ($i -lt $Requests -and $Delay -gt 0) {
        Start-Sleep -Milliseconds $Delay
    }
}

$endTime = Get-Date
$duration = ($endTime - $startTime).TotalSeconds

# Display summary
Write-ColorOutput "`n=== Summary ===" -Color $Colors.Info
Write-ColorOutput "Total Requests: $Requests" -Color $Colors.Data
Write-ColorOutput "Successful: $successCount" -Color $Colors.Success
Write-ColorOutput "Failed: $failCount" -Color $(if ($failCount -gt 0) { $Colors.Error } else { $Colors.Data })
Write-ColorOutput "Total Bytes: $totalBytes" -Color $Colors.Data
Write-ColorOutput "Duration: $([math]::Round($duration, 2))s" -Color $Colors.Data
Write-ColorOutput "Throughput: $([math]::Round($totalBytes / $duration, 2)) bytes/sec" -Color $Colors.Data

# Show statistics if requested
if ($ShowStats -and $successCount -gt 0) {
    Write-ColorOutput "`n=== Data Statistics ===" -Color $Colors.Info
    $combinedData = $allData -join ""
    $stats = Get-DataStats -Data $combinedData
    
    Write-ColorOutput "Bytes analyzed: $($stats.Length)" -Color $Colors.Data
    Write-ColorOutput "Mean value: $($stats.Mean)" -Color $Colors.Data
    Write-ColorOutput "Min value: $($stats.Min)" -Color $Colors.Data
    Write-ColorOutput "Max value: $($stats.Max)" -Color $Colors.Data
    Write-ColorOutput "Unique bytes: $($stats.Unique)/256 ($($stats.Entropy)%)" -Color $Colors.Data
}

# Save to file if specified
if ($OutputFile -and $successCount -gt 0) {
    Write-ColorOutput "`n=== Saving to File ===" -Color $Colors.Info
    $combinedData = $allData -join ""
    
    if (Save-ToFile -Data $combinedData -Path $OutputFile -Format $Encoding) {
        Write-ColorOutput "✓ Saved to: $OutputFile" -Color $Colors.Success
        Write-ColorOutput "  Size: $(Get-Item $OutputFile | Select-Object -ExpandProperty Length) bytes" -Color $Colors.Data
    }
}

Write-ColorOutput ""

# Return exit code based on success
exit $(if ($failCount -eq 0) { 0 } else { 1 })
