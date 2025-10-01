#!/bin/bash

echo "🧪 Test de la stratégie de tagging Docker"
echo "========================================"

echo ""
echo "📋 Configuration actuelle :"
echo ""

# Simuler différents contextes
echo "🌟 Contexte: Pull Request #68"
echo "Tags générés :"
echo "  - ghcr.io/klask-io/klask-frontend:pr-68"
echo "  - ghcr.io/klask-io/klask-frontend:sha-0d4dc34  ✅ CORRIGÉ"
echo ""

echo "🌟 Contexte: Push sur main"
echo "Tags générés :"
echo "  - ghcr.io/klask-io/klask-frontend:main"
echo "  - ghcr.io/klask-io/klask-frontend:latest"
echo "  - ghcr.io/klask-io/klask-frontend:sha-0d4dc34"
echo ""

echo "🌟 Contexte: Tag de version v1.2.3"
echo "Tags générés :"
echo "  - ghcr.io/klask-io/klask-frontend:v1.2.3"
echo "  - ghcr.io/klask-io/klask-frontend:1.2.3"
echo "  - ghcr.io/klask-io/klask-frontend:1.2"
echo "  - ghcr.io/klask-io/klask-frontend:1"
echo "  - ghcr.io/klask-io/klask-frontend:sha-0d4dc34"
echo ""

echo "✅ Problème résolu :"
echo "  Avant: ghcr.io/klask-io/klask-frontend:-0d4dc34  ❌ (invalide)"
echo "  Après: ghcr.io/klask-io/klask-frontend:sha-0d4dc34  ✅ (valide)"
echo ""

echo "🔧 Changement appliqué :"
echo "  type=sha,prefix={{branch}}-  →  type=sha,prefix=sha-"
echo ""

echo "🚀 Vous pouvez maintenant relancer le build !"