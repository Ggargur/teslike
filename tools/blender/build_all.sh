#!/usr/bin/env bash
# Gera todos os modelos (.glb + .png de previa) em assets/models/.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="$ROOT/assets/models"
mkdir -p "$OUT"
for m in sword humanoid dummy; do
  blender --background --python "$ROOT/tools/blender/$m.py" -- "$OUT/$m.glb"
done
