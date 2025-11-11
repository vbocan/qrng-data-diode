#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Direct MCP bridge for QRNG Gateway - No proxy needed!
    
.DESCRIPTION
    This script bridges stdio (for Claude Desktop) to HTTP (for remote QRNG gateway)
    using PowerShell's built-in HTTP capabilities.
    
.PARAMETER GatewayUrl
    The QRNG Gateway URL (e.g., https://qrng.yourdomain.com)
    
.EXAMPLE
    # Claude Desktop configuration:
    {
      "mcpServers": {
        "qrng": {
          "command": "pwershell",
          "args": [
            "-File",
            "D:\\path\\to\\mcp_bridge.ps1",
            "-GatewayUrl",
            "https://qrng.yourdomain.com"
          ]
        }
      }
    }
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$GatewayUrl
)

# Ensure URL ends without trailing slash
$mcpUrl = $GatewayUrl.TrimEnd('/') + '/mcp'

# Process stdin line by line
try {
    while ($line = [Console]::ReadLine()) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        
        try {
            # Parse JSON request
            $request = $line | ConvertFrom-Json
            
            # Forward to gateway
            $response = Invoke-RestMethod -Uri $mcpUrl `
                -Method Post `
                -ContentType 'application/json' `
                -Body $line `
                -TimeoutSec 30 `
                -ErrorAction Stop
            
            # Send response to stdout
            $response | ConvertTo-Json -Depth 10 -Compress | Write-Host
            
        } catch {
            # Return error response
            $errorResponse = @{
                jsonrpc = "2.0"
                id = if ($request) { $request.id } else { $null }
                error = @{
                    code = -32000
                    message = $_.Exception.Message
                }
            }
            $errorResponse | ConvertTo-Json -Depth 10 -Compress | Write-Host
        }
    }
} catch {
    # Silently exit on pipe close
}
