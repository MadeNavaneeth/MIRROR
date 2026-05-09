#!/bin/bash
set -e

# Mirror UI Build Script
# Standards-compliant with OpenCode design system

echo "🚀 Building Mirror Dashboard..."

cd "$(dirname "$0")/../dashboard"

if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

echo "🔨 Running Vite build..."
npm run build

echo "✅ Build complete. Files ready in dashboard/dist"
echo "👉 Start the gateway to serve the UI: mirror gateway"
