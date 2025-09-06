#!/bin/bash

# Symor Build Script
# Builds release binaries for all platforms and creates distribution packages

set -e

echo "🔨 Building symor for all platforms..."

# Create releases directory
rm -rf web/src/releases
echo "rm -rf web/src/releases"
mkdir -p web/src/releases

# Build targets
TARGETS=(
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "x86_64-pc-windows-gnu"
    "i686-pc-windows-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

# Build each target
for target in "${TARGETS[@]}"; do
    echo "📦 Building for $target..."
    
    if [[ "$target" == "x86_64-pc-windows-gnu" ]]; then
        cargo build --release --target "$target" --jobs 16
        binary_name="sym.exe"
    elif [[ "$target" == "i686-pc-windows-gnu" ]]; then
        cargo build --release --target "$target" --jobs 16
        binary_name="sym.exe"    
    else
        cargo build --release --target "$target" --jobs 16
        binary_name="sym"
    fi
    
    if [ $? -ne 0 ]; then
        echo "❌ Build failed for $target" 
        exit 1
    fi
    
    # Copy binary to releases with platform suffix
    platform_name=$(echo "$target" | sed 's/-/_/g')
    cp -f "target/$target/release/$binary_name" "web/src/releases/sym_$platform_name"
done

echo "✅ All builds successful!"

echo "Waiting..."

sleep 3

# Create tar.gz with all binaries and install script
echo "📦 Creating distribution tar.gz..."
cd web/src
tar -czf ../get/latest.tar.gz -C releases . -C .. install.sh

echo "✅ Distribution created: web/src/releases/latest.tar.gz"
echo "📊 File size: $(du -h releases/latest.tar.gz | cut -f1)"
echo "🎉 Build complete!"
