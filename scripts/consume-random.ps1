#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Generate passwords and UUIDs from QRNG Gateway random data

.DESCRIPTION
    This script continuously generates secure passwords and UUIDs using
    quantum random numbers from the QRNG Gateway.

.EXAMPLE
    .\consume-random.ps1
    Run continuously generating passwords and UUIDs every 2 seconds

.NOTES
    Author: Valer BOCAN, PhD, CSSLP - www.bocan.ro
    Project: QRNG Data Diode
#>

# Hardcoded configuration
$GatewayUrl = "http://localhost:7764"
$ApiKey = "test-key-1234567890"
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

function New-UUID {
    param([byte[]]$RandomBytes)
    
    # Create a copy to avoid modifying the original array
    $bytes = [byte[]]::new(16)
    [Array]::Copy($RandomBytes, 0, $bytes, 0, 16)
    
    $bytes[6] = ($bytes[6] -band 0x0F) -bor 0x40
    $bytes[8] = ($bytes[8] -band 0x3F) -bor 0x80
    
    $uuid = "{0:x2}{1:x2}{2:x2}{3:x2}-{4:x2}{5:x2}-{6:x2}{7:x2}-{8:x2}{9:x2}-{10:x2}{11:x2}{12:x2}{13:x2}{14:x2}{15:x2}" -f `
        $bytes[0], $bytes[1], $bytes[2], $bytes[3],
        $bytes[4], $bytes[5], $bytes[6], $bytes[7],
        $bytes[8], $bytes[9], $bytes[10], $bytes[11],
        $bytes[12], $bytes[13], $bytes[14], $bytes[15]
    
    return $uuid
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
    for ($i = 0; $i -lt $UUIDsPerCycle; $i++) {
        $result = Get-RandomData -Url $GatewayUrl -Key $ApiKey -ByteCount 16 -Format "hex"
        
        if ($result.Success) {
            $uuid = New-UUID -RandomBytes $result.Data
            Write-ColorOutput "  [$($i + 1)] $uuid" -Color $Colors.Data
        } else {
            Write-ColorOutput "  [$($i + 1)] Error: $($result.Error)" -Color $Colors.Error
        }
    }
    
    Write-ColorOutput "`nNext update in $IntervalSeconds seconds..." -Color $Colors.Info
    Start-Sleep -Seconds $IntervalSeconds
}
