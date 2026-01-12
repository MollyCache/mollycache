#!/bin/bash
# Bash/Zsh Setup Script for MollyCache

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🔮 Setting up MollyCache environment..."

# 1. Configure Git Hooks
echo "1️⃣  Configuring Git Hooks..."
git config core.hooksPath .githooks
echo "   ✅ Git hooks enabled."

# 2. Install 'molly' alias/function
echo "2️⃣  Installing 'molly' shell function..."

# Detect profile
SHELL_PROFILE="$HOME/.bashrc"
if [ -n "$ZSH_VERSION" ]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_PROFILE="$HOME/.bashrc"
fi

MOLLY_FUNC="
# --- MollyCache Dev Tools ---
molly() {
    local target=$1
    local project_root="$REPO_ROOT"
    
    if [ -z "$target" ]; then
        cd "$project_root"
        return
    fi

    if [ "$target" = "-t" ]; then
        target=$2
    fi
    
    # Handle -t flag if passed as first arg (simple parsing)
    if [[ "$1" == "-t" ]]; then
        target=$2
    fi

    if [ -z "$target" ]; then
        echo "Usage: molly [-t name]"
        return 1
    fi

    local wt_dir="$project_root/worktrees/$target"
    
    if [ -d "$wt_dir" ]; then
        echo "📂 Switching to worktree: $target"
        cd "$wt_dir"
    else
        echo "🌿 Creating new worktree: $target"
        
        # Save current dir
        local old_loc=$(pwd)
        cd "$project_root"
        
        # Create worktree
        if git worktree add "worktrees/$target" -b "ai/$target"; then
            cd "worktrees/$target"
        else
            echo "⚠️  Failed to create branch 'ai/$target'. Trying existing branch..."
            if git worktree add "worktrees/$target" "$target"; then
                cd "worktrees/$target"
            else
                echo "❌ Failed to create worktree."
                cd "$old_loc"
                return 1
            fi
        fi
    fi
}
# ----------------------------
"

if grep -q "# --- MollyCache Dev Tools ---" "$SHELL_PROFILE"; then
    echo "   ⚠️  'molly' function already exists in $SHELL_PROFILE. Skipping."
else
    echo "$MOLLY_FUNC" >> "$SHELL_PROFILE"
    echo "   ✅ 'molly' function added to $SHELL_PROFILE"
fi

echo ""
echo "🎉 Setup complete! Restart your terminal or run 'source $SHELL_PROFILE' to use the 'molly' command."
