#!/usr/bin/env pwsh
<#
.SYNOPSIS
    MCP bridge for QRNG MCP Server - Connects Claude Desktop to remote MCP server
    
.DESCRIPTION
    This script bridges stdio (for Claude Desktop) to HTTP (for remote QRNG MCP server)
    using the Streamable HTTP protocol (MCP 2025-06-18 spec).
    
.PARAMETER McpServerUrl
    The QRNG MCP Server URL (e.g., http://qrng-mcp:8080)
    
.EXAMPLE
    # Claude Desktop configuration:
    {
      "mcpServers": {
        "qrng": {
          "command": "pwsh",
          "args": [
            "-File",
            "D:\\path\\to\\mcp_bridge.ps1",
            "-McpServerUrl",
            "http://localhost:8080"
          ]
        }
      }
    }
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$McpServerUrl
)

# Ensure URL ends without trailing slash
$baseUrl = $McpServerUrl.TrimEnd('/')

# Create persistent HTTP client for session management
$sessionId = $null

# Process stdin line by line
try {
    while ($line = [Console]::ReadLine()) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        
        try {
            # Parse JSON request
            $request = $line | ConvertFrom-Json
            
            # Determine HTTP method based on request
            $method = 'POST'
            $uri = $baseUrl
            
            # Check if this is a session termination
            if ($request.method -eq 'close' -and $sessionId) {
                $method = 'DELETE'
                $uri = "$baseUrl/?sessionId=$sessionId"
            }
            
            # Forward to MCP server
            $headers = @{
                'Content-Type' = 'application/json'
            }
            
            if ($sessionId) {
                $headers['X-Session-Id'] = $sessionId
            }
            
            $response = Invoke-RestMethod -Uri $uri `
                -Method $method `
                -Headers $headers `
                -Body $line `
                -TimeoutSec 30 `
                -ErrorAction Stop
            
            # Extract session ID from response if present
            if ($response.sessionId) {
                $sessionId = $response.sessionId
            }
            
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
