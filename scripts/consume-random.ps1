#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Generate passwords and UUIDs from QRNG Gateway random data

.DESCRIPTION
    This script continuously generates secure passwords and UUIDs using
    quantum random numbers from the QRNG Gateway.
    
    NOTE: This script now uses the optimized /api/uuid endpoint instead of
    manually constructing UUIDs from /api/random bytes. This is more efficient
    and ensures correct UUIDv4 formatting.

.EXAMPLE
    .\consume-random.ps1
    Run continuously generating passwords and UUIDs every 2 seconds

.NOTES
    Author: Valer BOCAN, PhD, CSSLP - www.bocan.ro
    Project: QRNG Data Diode
    Updated: 2025-11-17 - Optimized to use /api/uuid endpoint
#>

param(
    [string]$GatewayUrl = "http://localhost:7764",
    [string]$ApiKey = "test-key-1234567890"
)

# Configuration
$PasswordLength = 20
$PasswordsPerCycle = 3
$UUIDsPerCycle = 5
$IntervalSeconds = 2

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
    
    $endpoint = "$Url/api/random?bytes=$ByteCount&encoding=$Format"
    $headers = @{ "Authorization" = "Bearer $Key" }
    
    try {
        $response = Invoke-WebRequest -Uri $endpoint -Method Get -Headers $headers -UseBasicParsing -ErrorAction Stop
        
        # Decode hex data to bytes if format is hex
        if ($Format -eq "hex") {
            $hexString = $response.Content
            $bytes = [byte[]]::new($hexString.Length / 2)
            for ($i = 0; $i -lt $hexString.Length; $i += 2) {
                $bytes[$i / 2] = [Convert]::ToByte($hexString.Substring($i, 2), 16)
            }
            $data = $bytes
        } else {
            $data = $response.Content
        }
        
        return @{
            Success = $true
            Data = $data
            StatusCode = $response.StatusCode
            Length = $data.Length
        }
    }
    catch {
        return @{
            Success = $false
            Error = $_.Exception.Message
            StatusCode = if ($_.Exception.Response) { $_.Exception.Response.StatusCode.Value__ } else { 0 }
        }
    }
}

function New-SecurePassword {
    param(
        [byte[]]$RandomBytes,
        [int]$Length = 20
    )
    
    $charSets = @{
        Uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        Lowercase = "abcdefghijklmnopqrstuvwxyz"
        Digits = "0123456789"
        Special = "!@#$%^&*()-_=+[]{}|;:,.<>?"
    }
    
    $allChars = $charSets.Uppercase + $charSets.Lowercase + $charSets.Digits + $charSets.Special
    $password = ""
    
    $password += $charSets.Uppercase[$RandomBytes[0] % $charSets.Uppercase.Length]
    $password += $charSets.Lowercase[$RandomBytes[1] % $charSets.Lowercase.Length]
    $password += $charSets.Digits[$RandomBytes[2] % $charSets.Digits.Length]
    $password += $charSets.Special[$RandomBytes[3] % $charSets.Special.Length]
    
    for ($i = 4; $i -lt $Length; $i++) {
        $password += $allChars[$RandomBytes[$i] % $allChars.Length]
    }
    
    $chars = $password.ToCharArray()
    for ($i = $chars.Length - 1; $i -gt 0; $i--) {
        $j = $RandomBytes[($Length + $i) % $RandomBytes.Length] % ($i + 1)
        $temp = $chars[$i]
        $chars[$i] = $chars[$j]
        $chars[$j] = $temp
    }
    
    return -join $chars
}

function Get-UUIDs {
    param(
        [string]$Url,
        [string]$Key,
        [int]$Count = 1
    )
    
    $endpoint = "$Url/api/uuid?count=$Count"
    $headers = @{ "Authorization" = "Bearer $Key" }
    
    try {
        $response = Invoke-WebRequest -Uri $endpoint -Method Get -Headers $headers -UseBasicParsing -ErrorAction Stop
        
        if ($Count -eq 1) {
            # Single UUID returned as plain text
            return @{
                Success = $true
                Data = @($response.Content.Trim())
                StatusCode = $response.StatusCode
            }
        } else {
            # Multiple UUIDs returned as JSON array
            $uuids = $response.Content | ConvertFrom-Json
            return @{
                Success = $true
                Data = $uuids
                StatusCode = $response.StatusCode
            }
        }
    }
    catch {
        return @{
            Success = $false
            Error = $_.Exception.Message
            StatusCode = if ($_.Exception.Response) { $_.Exception.Response.StatusCode.Value__ } else { 0 }
        }
    }
}

# Main execution
Write-ColorOutput "`n=== QRNG Password & UUID Generator ===" -Color $Colors.Info
Write-ColorOutput "Gateway: $GatewayUrl" -Color $Colors.Info
Write-ColorOutput "Interval: $IntervalSeconds seconds | Passwords: $PasswordsPerCycle | UUIDs: $UUIDsPerCycle" -Color $Colors.Info
Write-ColorOutput "Press Ctrl+C to stop`n" -Color $Colors.Warning

$cycleCount = 0
$startTime = Get-Date

while ($true) {
    $cycleCount++
    $currentTime = Get-Date
    $elapsed = ($currentTime - $startTime).TotalSeconds
    
    [Console]::Clear()
    Write-ColorOutput "=== Cycle #$cycleCount | Runtime: $([math]::Round($elapsed, 1))s ===" -Color $Colors.Info
    Write-ColorOutput "Timestamp: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n" -Color $Colors.Data
    
    Write-ColorOutput "Quantum Random Passwords (length: $PasswordLength):" -Color $Colors.Success
    for ($i = 0; $i -lt $PasswordsPerCycle; $i++) {
        $result = Get-RandomData -Url $GatewayUrl -Key $ApiKey -ByteCount ($PasswordLength + 16) -Format "hex"
        
        if ($result.Success) {
            $password = New-SecurePassword -RandomBytes $result.Data -Length $PasswordLength
            Write-ColorOutput "  [$($i + 1)] $password" -Color $Colors.Data
        } else {
            Write-ColorOutput "  [$($i + 1)] Error: $($result.Error)" -Color $Colors.Error
        }
    }
    
    Write-ColorOutput "`nQuantum Random UUIDs (v4):" -Color $Colors.Success
    $result = Get-UUIDs -Url $GatewayUrl -Key $ApiKey -Count $UUIDsPerCycle
    
    if ($result.Success) {
        for ($i = 0; $i -lt $result.Data.Count; $i++) {
            Write-ColorOutput "  [$($i + 1)] $($result.Data[$i])" -Color $Colors.Data
        }
    } else {
        Write-ColorOutput "  Error fetching UUIDs: $($result.Error)" -Color $Colors.Error
    }
    
    Write-ColorOutput "`nNext update in $IntervalSeconds seconds..." -Color $Colors.Info
    Start-Sleep -Seconds $IntervalSeconds
}
