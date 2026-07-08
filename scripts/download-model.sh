#!/usr/bin/env bash
set -euo pipefail

echo "🚀 Downloading Llama 3.1 8B Instruct (Q4_K_M quantized)"
echo "📦 Size: ~4.5GB - This will take 5-15 minutes depending on your connection"
echo ""

# Create models directory
mkdir -p models

MODEL_URL="https://huggingface.co/Qwen/Qwen3-8B-GGUF/resolve/main/Qwen3-8B-Q4_K_M.gguf"
MODEL_PATH="models/Qwen3-8B-Q4_K_M.gguf"

# Check if already downloaded
if [ -f "$MODEL_PATH" ]; then
    echo "✅ Model already exists at $MODEL_PATH"
    exit 0
fi

# Download
echo "⬇️  Downloading..."
curl -L -o "$MODEL_PATH" "$MODEL_URL" --progress-bar

# Check file size (should be around 4.5GB)
SIZE=$(du -h "$MODEL_PATH" | cut -f1)
echo ""
echo "✅ Download complete!"
echo "📊 File size: $SIZE"
echo "📁 Location: $MODEL_PATH"
echo ""
echo "🎉 Ready to use! Run: cargo run --example test_llm"
