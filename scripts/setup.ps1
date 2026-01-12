# PowerShell Setup Script for MollyCache
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path "$PSScriptRoot\.."
$repoRoot = $repoRoot.Path
$profilePath = $PROFILE

Write-Host "🔮 Setting up MollyCache environment..."

# 1. Configure Git Hooks
Write-Host "1️⃣  Configuring Git Hooks..."
git config core.hooksPath .githooks
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Git hooks enabled."
} else {
    Write-Host "   ❌ Failed to configure git hooks." -ForegroundColor Red
}

# 2. Install 'molly' alias/function
Write-Host "2️⃣  Installing 'molly' shell function..."

$mollyFunc = @"

# --- MollyCache Dev Tools ---
function molly {
    param(
        [Alias("t")]
        [string]`$target
    )

    `$projectRoot = "$repoRoot"
    
    if (-not `$target) {
        Set-Location `$projectRoot
        return
    }

    # Handle Worktrees
    `$wtDir = Join-Path `$projectRoot "worktrees" `$target
    
    if (Test-Path `$wtDir) {
        Write-Host "📂 Switching to worktree: `$target" -ForegroundColor Cyan
        Set-Location `$wtDir
    } else {
        Write-Host "🌿 Creating new worktree: `$target" -ForegroundColor Green
        
        # Capture current location to return if git fails
        `$oldLoc = Get-Location
        Set-Location `$projectRoot
        
        # Create worktree (detached or new branch)
        # Try to verify if branch exists or create new one
        try {
            git worktree add "worktrees/`$target" -b "ai/`$target"
        } catch {
            Write-Warning "Branch might already exist or name is invalid. Trying checkout without -b..."
            git worktree add "worktrees/`$target" `$target
        }

        if ($?) {
            Set-Location `$wtDir
            # Copy config if needed (optional)
        } else {
            Set-Location `$oldLoc
            Write-Error "Failed to create worktree."
        }
    }
}
# ----------------------------
"@

# Check if already installed
if (Test-Path $profilePath) {
    $currentProfile = Get-Content $profilePath -Raw
    if ($currentProfile -match "# --- MollyCache Dev Tools ---") {
        Write-Host "   ⚠️  'molly' function already exists in profile. Skipping." -ForegroundColor Yellow
    } else {
        Add-Content -Path $profilePath -Value $mollyFunc
        Write-Host "   ✅ 'molly' function added to $profilePath"
    }
} else {
    New-Item -Path $profilePath -ItemType File -Force | Out-Null
    Add-Content -Path $profilePath -Value $mollyFunc
    Write-Host "   ✅ Created profile and added 'molly' function."
}

Write-Host "`n🎉 Setup complete! Restart your terminal or run '. `$profilePath' to use the 'molly' command."
