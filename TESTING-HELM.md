# Guide de test pour le Helm chart Klask

Ce guide vous explique comment tester votre Helm chart localement avant de pousser sur GitHub.

## 🚀 Tests rapides (recommandé)

### Option 1: Script automatisé
```bash
# Exécuter tous les tests
./scripts/test-helm.sh
```

### Option 2: Makefile
```bash
# Voir toutes les commandes disponibles
make -f Makefile.helm help

# Lancer tous les tests
make -f Makefile.helm test-all

# Tests individuels
make -f Makefile.helm lint
make -f Makefile.helm dry-run
make -f Makefile.helm template
```

## 🔍 Tests détaillés étape par étape

### 1. Validation de la syntaxe
```bash
cd charts/klask
helm lint .
```

### 2. Génération des templates (sans déploiement)
```bash
# Template de base
helm template klask-test . --debug

# Avec ingress activé
helm template klask-test . --set ingress.enabled=true

# Avec PostgreSQL externe
helm template klask-test . --set postgresql.enabled=false
```

### 3. Test d'installation (dry-run)
```bash
# Test complet sans déployer
helm install klask-test . --dry-run --debug

# Avec configuration personnalisée
helm install klask-test . -f values-local.yaml --dry-run
```

## 🐳 Tests avec images locales

### 1. Builder les images localement
```bash
# Backend
cd klask-rs
docker build -t klask-backend:local .

# Frontend
cd ../klask-react
docker build -t klask-frontend:local .
```

### 2. Tester avec les images locales
```bash
# Utiliser la config locale
helm install klask-test ./charts/klask -f charts/klask/values-local.yaml --dry-run

# Installation réelle sur cluster local
helm install klask-test ./charts/klask -f charts/klask/values-local.yaml
```

## ☸️ Tests sur clusters locaux

### Kind (Kubernetes in Docker)
```bash
# Créer un cluster kind
kind create cluster --name klask-test

# Charger les images dans kind
kind load docker-image klask-backend:local --name klask-test
kind load docker-image klask-frontend:local --name klask-test

# Déployer
make -f Makefile.helm kind-install

# Nettoyer
helm uninstall klask-test
kind delete cluster --name klask-test
```

### Minikube
```bash
# Démarrer minikube
minikube start

# Utiliser le Docker de minikube
eval $(minikube docker-env)

# Builder les images dans minikube
cd klask-rs && docker build -t klask-backend:local .
cd ../klask-react && docker build -t klask-frontend:local .

# Déployer
make -f Makefile.helm minikube-install

# Accéder à l'application
minikube service klask-test-frontend
```

## 🧪 Tests de configuration

### Test des différentes configurations
```bash
# Test 1: Configuration minimale
helm template test1 charts/klask --set postgresql.enabled=false

# Test 2: Avec ingress
helm template test2 charts/klask --set ingress.enabled=true

# Test 3: Ressources personnalisées
helm template test3 charts/klask \
  --set backend.resources.requests.cpu=200m \
  --set frontend.resources.limits.memory=1Gi

# Test 4: Configuration production
helm template test4 charts/klask \
  --set ingress.enabled=true \
  --set postgresql.persistence.enabled=true \
  --set postgresql.persistence.size=20Gi
```

### Validation avec kubeval (optionnel)
```bash
# Installer kubeval
go install github.com/instrumenta/kubeval@latest

# Valider les manifests générés
helm template klask-test charts/klask | kubeval
```

## 🐛 Débogage

### Voir les manifests générés
```bash
# Sauvegarder les templates
helm template klask-test charts/klask --debug > klask-manifests.yaml

# Examiner un composant specifique
helm template klask-test charts/klask | grep -A 20 "kind: Deployment"
```

### Vérifier les valeurs utilisées
```bash
# Voir les valeurs par défaut
helm show values charts/klask

# Voir les valeurs après merge avec votre config
helm template klask-test charts/klask -f charts/klask/values-local.yaml --debug
```

### Tester les health checks
```bash
# Après installation
kubectl get pods -l "app.kubernetes.io/instance=klask-test"
kubectl describe pod <pod-name>
kubectl logs <pod-name>
```

## ✅ Checklist avant push

- [ ] `helm lint` passe sans erreur
- [ ] `helm template` génère des manifests valides
- [ ] `helm install --dry-run` réussit
- [ ] Test avec ingress activé/désactivé
- [ ] Test avec PostgreSQL interne/externe
- [ ] Test avec différentes tailles de ressources
- [ ] Validation kubeval (si disponible)
- [ ] Test d'installation réelle sur cluster local
- [ ] Health checks fonctionnent
- [ ] Application accessible via port-forward ou ingress

## 🔄 Workflow recommandé

1. **Modifications** → Éditer le chart
2. **Lint** → `make -f Makefile.helm lint`
3. **Template** → `make -f Makefile.helm template`
4. **Dry-run** → `make -f Makefile.helm dry-run`
5. **Test local** → Installation sur kind/minikube
6. **Validation** → Vérifier que l'app fonctionne
7. **Push** → `git push` (déclenche le workflow CI)

Une fois que tous ces tests passent, vous pouvez pousser en confiance ! 🚀