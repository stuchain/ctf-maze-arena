param(
    [ValidateSet('all', 'rust', 'web')]
    [string]$Scope = 'all'
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot

function Invoke-Checked {
    param([scriptblock]$Command, [string]$Description)
    Write-Host "==> $Description"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit code $LASTEXITCODE"
    }
}

if ($Scope -in @('all', 'rust')) {
    Push-Location $projectRoot
    try {
        Invoke-Checked { cargo fmt --all -- --check } 'Rust formatting'
        Invoke-Checked { cargo clippy --workspace --all-targets -- -D warnings } 'Rust Clippy'
        Invoke-Checked { cargo test --workspace } 'Rust tests'
    }
    finally {
        Pop-Location
    }
}

if ($Scope -in @('all', 'web')) {
    Push-Location (Join-Path $projectRoot 'web')
    try {
        Invoke-Checked { npm run lint } 'Frontend lint'
        Invoke-Checked { npm run typecheck } 'Frontend typecheck'
        Invoke-Checked { npm run test:unit } 'Frontend unit tests'
        Invoke-Checked { npm run build } 'Frontend production build'
    }
    finally {
        Pop-Location
    }
}
