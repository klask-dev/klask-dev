#!/bin/bash

echo "🎭 Activation du workflow de test MOCK"
echo "====================================="

echo ""
echo "📋 Actions à faire :"
echo ""

echo "1️⃣ Désactiver temporairement le workflow principal :"
echo "   mv .github/workflows/ci.yml .github/workflows/ci.yml.backup"
echo ""

echo "2️⃣ Activer le workflow mock :"
echo "   mv .github/workflows/ci-mock.yml .github/workflows/ci.yml"
echo ""

echo "3️⃣ Commit et push pour tester :"
echo "   git add ."
echo "   git commit -m 'test: enable mock workflow for security scan testing'"
echo "   git push"
echo ""

echo "4️⃣ Une fois le test terminé, restaurer le workflow original :"
echo "   mv .github/workflows/ci.yml .github/workflows/ci-mock.yml"
echo "   mv .github/workflows/ci.yml.backup .github/workflows/ci.yml"
echo "   git add ."
echo "   git commit -m 'restore: original CI workflow'"
echo "   git push"
echo ""

echo "🎯 Ce que le workflow mock fait :"
echo "  ✅ Tests mockés (echo + sleep 5s)"
echo "  ✅ Build mocké (echo + sleep 3s)"  
echo "  ✅ Security scan RÉEL sur images existantes"
echo "  ✅ Upload SARIF avec catégories séparées"
echo ""

echo "⚡ Avantages :"
echo "  🚀 Workflow complet en ~15 secondes"
echo "  🔍 Test du security scan rapidement"
echo "  💰 Économie de ressources CI/CD"
echo ""

echo "👉 Prêt à activer le mock ? Lancez les commandes ci-dessus !"