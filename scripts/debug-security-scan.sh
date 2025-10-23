#!/bin/bash

echo "🔍 Debug Security Scan Tags"
echo "============================"

echo ""
echo "📋 Problème résolu :"
echo "  Avant: ghcr.io/klask-dev/klask-frontend:68/merge ❌"
echo "  Après: ghcr.io/klask-dev/klask-frontend:pr-68    ✅"
echo ""

echo "🏷️ Logique de tags pour security scan :"
echo ""

echo "🔹 Pull Request #68 :"
echo "  github.event_name = 'pull_request'"
echo "  github.event.number = '68'"  
echo "  → Tag utilisé: pr-68"
echo ""

echo "🔹 Push sur main :"
echo "  github.ref = 'refs/heads/main'"
echo "  → Tag utilisé: latest"
echo ""

echo "🔹 Tag de version v1.2.3 :"
echo "  github.ref_type = 'tag'"
echo "  github.ref_name = 'v1.2.3'"
echo "  → Tag utilisé: v1.2.3"
echo ""

echo "🔹 Branch feature/auth-system :"
echo "  github.ref_name = 'feature/auth-system'"
echo "  → Tag utilisé: feature-auth-system (nettoyé)"
echo ""

echo "✅ Plus de tags invalides avec des '/' !"
echo ""
echo "📖 Note: Les tags Docker doivent respecter le format :"
echo "  [a-zA-Z0-9][a-zA-Z0-9._-]*"
echo "  Donc '68/merge' → invalide, 'pr-68' → valide"