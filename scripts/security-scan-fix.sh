#!/bin/bash

echo "🔒 Corrections Security Scan"
echo "============================"

echo ""
echo "❌ Problèmes identifiés :"
echo "  1. Multiple SARIF runs avec même catégorie"
echo "  2. Git repository checkout manquant"
echo ""

echo "✅ Corrections appliquées :"
echo ""

echo "1️⃣ Ajout du checkout repository :"
echo "   - name: Checkout repository"
echo "     uses: actions/checkout@v4"
echo ""

echo "2️⃣ Séparation des uploads SARIF :"
echo ""
echo "   Avant (problématique) :"
echo "   - sarif_file: '.'  # Upload tous les .sarif → conflit de catégorie"
echo ""
echo "   Après (corrigé) :"
echo "   - sarif_file: 'trivy-frontend.sarif'"
echo "     category: 'trivy-frontend'"
echo "   - sarif_file: 'trivy-backend.sarif'"  
echo "     category: 'trivy-backend'"
echo ""

echo "🎯 Résultat attendu :"
echo "  ✅ Frontend security scan dans l'onglet 'trivy-frontend'"
echo "  ✅ Backend security scan dans l'onglet 'trivy-backend'"
echo "  ✅ Pas de conflit de catégorie"
echo "  ✅ Git repository accessible pour l'upload"
echo ""

echo "📊 Dans l'interface GitHub :"
echo "  → Security tab"
echo "  → Code scanning alerts"
echo "  → Voir les résultats séparés frontend/backend"