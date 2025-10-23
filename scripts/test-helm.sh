#!/bin/bash
set -e

echo "🧪 Tests locaux du Helm chart Klask"
echo "=================================="

CHART_DIR="charts/klask"

# Vérifier que le chart existe
if [ ! -d "$CHART_DIR" ]; then
    echo "❌ Répertoire $CHART_DIR non trouvé"
    exit 1
fi

cd "$CHART_DIR"

echo ""
echo "1️⃣ Validation de la syntaxe (helm lint)"
echo "----------------------------------------"
helm lint .

echo ""
echo "2️⃣ Génération des templates (dry-run)"
echo "------------------------------------"
helm template klask-test . --debug --dry-run > /tmp/klask-templates.yaml
echo "✅ Templates générés dans /tmp/klask-templates.yaml"

echo ""
echo "3️⃣ Validation Kubernetes (kubeval si disponible)"
echo "-----------------------------------------------"
if command -v kubeval &> /dev/null; then
    kubeval /tmp/klask-templates.yaml
    echo "✅ Validation Kubernetes réussie"
else
    echo "⚠️  kubeval non installé, validation Kubernetes ignorée"
    echo "   Installation: go install github.com/instrumenta/kubeval@latest"
fi

echo ""
echo "4️⃣ Test avec différentes configurations"
echo "--------------------------------------"

# Test avec ingress activé
echo "📋 Test avec ingress activé..."
helm template klask-test . --set ingress.enabled=true --dry-run > /dev/null
echo "✅ Configuration ingress OK"

# Test avec PostgreSQL désactivé
echo "📋 Test avec PostgreSQL externe..."
helm template klask-test . --set postgresql.enabled=false --dry-run > /dev/null
echo "✅ Configuration PostgreSQL externe OK"

# Test avec ressources personnalisées
echo "📋 Test avec ressources personnalisées..."
helm template klask-test . \
    --set backend.resources.requests.cpu=200m \
    --set frontend.resources.limits.memory=1Gi \
    --dry-run > /dev/null
echo "✅ Configuration ressources OK"

echo ""
echo "6️⃣ Packaging du chart"
echo "--------------------"
cd ..
helm package klask
echo "✅ Chart packagé: $(ls klask-*.tgz)"

echo ""
echo "🎉 Tous les tests sont passés !"
echo ""
echo "Prochaines étapes:"
echo "- helm install klask-test ./klask --dry-run --debug (test d'installation)"
echo "- helm install klask-test ./klask (installation réelle sur cluster de test)"
echo "- git push (une fois validé)"