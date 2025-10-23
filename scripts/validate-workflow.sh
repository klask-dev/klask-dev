#!/bin/bash

echo "🔍 Validation des changements workflow CI/CD"
echo "==========================================="

WORKFLOW_FILE=".github/workflows/ci.yml"

if [ ! -f "$WORKFLOW_FILE" ]; then
    echo "❌ Fichier $WORKFLOW_FILE non trouvé"
    exit 1
fi

echo ""
echo "✅ Changements appliqués :"
echo ""

# Vérifier que rust-modernization est dans les triggers
if grep -q "rust-modernization" "$WORKFLOW_FILE"; then
    echo "1. ✅ Branche 'rust-modernization' ajoutée aux triggers"
else
    echo "1. ❌ Branche 'rust-modernization' manquante"
fi

# Vérifier les conditions des jobs
if grep -q "rust-modernization.*build-and-push" "$WORKFLOW_FILE"; then
    echo "2. ✅ Job 'build-and-push' mis à jour pour rust-modernization"
else
    echo "2. ❌ Job 'build-and-push' non mis à jour"
fi

if grep -q "rust-modernization.*security-scan" "$WORKFLOW_FILE"; then
    echo "3. ✅ Job 'security-scan' mis à jour pour rust-modernization"
else
    echo "3. ❌ Job 'security-scan' non mis à jour"
fi

# Vérifier la logique de tag dynamique
if grep -q "steps.tag.outputs.tag" "$WORKFLOW_FILE"; then
    echo "4. ✅ Tag dynamique implémenté pour security scan"
else
    echo "4. ❌ Tag dynamique manquant"
fi

echo ""
echo "📋 Résumé des corrections :"
echo ""
echo "Problème original :"
echo "  ❌ Security scan cherchait 'latest' pour branche rust-modernization"
echo "  ❌ Images n'étaient pas buildées pour rust-modernization"
echo ""
echo "Corrections appliquées :"
echo "  ✅ Branche rust-modernization ajoutée aux triggers"
echo "  ✅ Jobs build-and-push + security-scan activés pour rust-modernization"  
echo "  ✅ Tag dynamique : 'latest' pour main/master, nom de branche sinon"
echo ""
echo "Comportement attendu maintenant :"
echo "  📦 main/master → Images avec tag 'latest'"
echo "  📦 rust-modernization → Images avec tag 'rust-modernization'"
echo "  🔍 Security scan utilise le bon tag selon la branche"
echo ""
echo "🚀 Vous pouvez maintenant pousser et tester !"