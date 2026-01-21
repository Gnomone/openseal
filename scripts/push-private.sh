#!/bin/bash
# OpenSeal Private Backup Push Script
# Safely pushes to private repository with appropriate gitignore

set -e

REPO_DIR="/root/highpass/hackerton/openseal"
PRIVATE_REMOTE="https://github.com/Gnomone/openseal_private.git"

cd "$REPO_DIR"

echo "🔐 OpenSeal Private Backup Push"
echo "================================"

# 1. Backup current gitignore
echo "📋 Backing up current .gitignore..."
cp .gitignore .gitignore.backup

# 2. Switch to private gitignore
echo "🔄 Switching to private .gitignore..."
cp .gitignore.private .gitignore

# 3. Add private remote if not exists
if ! git remote | grep -q "private"; then
    echo "🔗 Adding private remote..."
    git remote add private "$PRIVATE_REMOTE"
fi

# 4. Commit and push to private
echo "📦 Committing changes..."
git add .
git commit -m "backup: sync to private repo $(date '+%Y-%m-%d %H:%M:%S')" || echo "No changes to commit"

echo "⬆️  Pushing to private repository..."
git push private main --force

# 5. Restore public gitignore
echo "🔄 Restoring public .gitignore..."
cp .gitignore.public .gitignore

# 6. Clean up backup
rm .gitignore.backup

echo "✅ Private backup complete!"
echo "📝 .gitignore restored to public version"
