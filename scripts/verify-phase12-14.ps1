param(
    [string]$ApiBaseUrl = "http://localhost:8080",
    [switch]$IncludeDocker
)

$ErrorActionPreference = "Stop"

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )
    if (-not $Condition) {
        throw "ASSERTION FAILED: $Message"
    }
}

function Get-StatusCode {
    param(
        [string]$Method,
        [string]$Url,
        [hashtable]$Headers = @{},
        [string]$Body = ""
    )

    try {
        if ($Method -eq "GET") {
            $resp = Invoke-WebRequest -Method Get -Uri $Url -Headers $Headers -UseBasicParsing
        } elseif ($Method -eq "POST") {
            $resp = Invoke-WebRequest -Method Post -Uri $Url -Headers $Headers -Body $Body -ContentType "application/json" -UseBasicParsing
        } elseif ($Method -eq "OPTIONS") {
            $resp = Invoke-WebRequest -Method Options -Uri $Url -Headers $Headers -UseBasicParsing
        } else {
            throw "Unsupported method $Method"
        }
        return [int]$resp.StatusCode
    } catch {
        if ($_.Exception.Response -and $_.Exception.Response.StatusCode) {
            return [int]$_.Exception.Response.StatusCode.value__
        }
        throw
    }
}

Write-Host "==> Running cargo tests"
cargo test

Write-Host "==> Probing API at $ApiBaseUrl"
try {
    $probe = Invoke-WebRequest -UseBasicParsing "$ApiBaseUrl/api/health" -TimeoutSec 2
    Assert-True ($probe.StatusCode -eq 200) "API health probe failed at $ApiBaseUrl/api/health"
} catch {
    throw "API is not reachable at $ApiBaseUrl. Start the API first (e.g. in another terminal) and re-run this script."
}

try {
    Write-Host "==> Checking security headers"
    $headersResp = Invoke-WebRequest -UseBasicParsing "$ApiBaseUrl/api/health"
    Assert-True ($headersResp.Headers["X-Content-Type-Options"] -eq "nosniff") "Missing X-Content-Type-Options"
    Assert-True ($headersResp.Headers["X-Frame-Options"] -eq "DENY") "Missing X-Frame-Options"
    Assert-True ($headersResp.Headers["Referrer-Policy"] -eq "strict-origin-when-cross-origin") "Missing Referrer-Policy"

    Write-Host "==> Checking CORS allowlist behavior"
    $allowedHeaders = @{ Origin = "http://allowed.test" }
    $allowed = Invoke-WebRequest -UseBasicParsing -Uri "$ApiBaseUrl/api/health" -Headers $allowedHeaders
    Assert-True ($allowed.Headers["Access-Control-Allow-Origin"] -eq "http://allowed.test") "Allowed origin not echoed"

    $deniedHeaders = @{ Origin = "http://evil.test" }
    $denied = Invoke-WebRequest -UseBasicParsing -Uri "$ApiBaseUrl/api/health" -Headers $deniedHeaders
    Assert-True ([string]::IsNullOrWhiteSpace($denied.Headers["Access-Control-Allow-Origin"])) "Disallowed origin unexpectedly allowed"

    Write-Host "==> Checking websocket route sanity"
    $wsStatus = Get-StatusCode -Method GET -Url "$ApiBaseUrl/api/solve/stream?runId=dummy"
    Assert-True (($wsStatus -eq 400) -or ($wsStatus -eq 426)) "Unexpected WS route status without upgrade: $wsStatus"

    Write-Host "==> Checking baseline route limiter returns 429 under burst"
    $dailyStatuses = @()
    for ($i = 0; $i -lt 12; $i++) {
        $dailyStatuses += Get-StatusCode -Method GET -Url "$ApiBaseUrl/api/daily"
    }
    Assert-True ($dailyStatuses -contains 429) "Baseline limiter did not return 429"

    Write-Host "==> Checking health route remains available under load"
    $healthAfterLoad = Get-StatusCode -Method GET -Url "$ApiBaseUrl/api/health"
    Assert-True ($healthAfterLoad -eq 200) "Health endpoint unavailable under load"

    Write-Host "==> Checking expensive routes limit earlier than baseline"
    Start-Sleep -Seconds 2
    $expensiveStatuses = @()
    for ($i = 0; $i -lt 6; $i++) {
        $body = '{"mazeId":"missing","solver":"BFS"}'
        $expensiveStatuses += Get-StatusCode -Method POST -Url "$ApiBaseUrl/api/solve" -Body $body
    }
    Assert-True ($expensiveStatuses -contains 429) "Expensive route limiter did not return 429"

    Write-Host "All API verification checks passed."
}
finally {}

if ($IncludeDocker) {
    Write-Host "==> Running Docker checks (optional)"
    docker build -t ctf-maze-api .
    Assert-True ($LASTEXITCODE -eq 0) "docker build failed"
    $whoamiOut = docker run --rm ctf-maze-api whoami
    Assert-True ($LASTEXITCODE -eq 0) "docker run whoami failed"
    Assert-True ($whoamiOut -ne "root") "Container user should not be root"

    docker compose up --build -d
    Assert-True ($LASTEXITCODE -eq 0) "docker compose up failed"
    try {
        Start-Sleep -Seconds 3
        $composeHealth = Invoke-WebRequest -UseBasicParsing "http://localhost:8080/api/health"
        Assert-True ($composeHealth.StatusCode -eq 200) "Compose health check failed"
    }
    finally {
        docker compose down
    }

    Write-Host "Docker verification checks passed."
}

Write-Host "Phase 12-14 verification script completed successfully."
