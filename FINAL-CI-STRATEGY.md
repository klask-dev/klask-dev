# Stratégie CI/CD finale

## 🎯 **Philosophie adoptée**

**Build sur toutes les branches** - Maximum de flexibilité pour le développement

## 🚀 **Quand les images sont construites**

### ✅ **Push sur n'importe quelle branche**
```bash
git push origin feature/auth-system
```
**Images créées :**
- `ghcr.io/klask-dev/klask-backend:feature-auth-system`
- `ghcr.io/klask-dev/klask-frontend:feature-auth-system`
- `ghcr.io/klask-dev/klask-backend:sha-abc1234`
- `ghcr.io/klask-dev/klask-frontend:sha-abc1234`

### ✅ **Push sur main/master**
```bash
git push origin main
```
**Images créées :**
- `ghcr.io/klask-dev/klask-backend:main`
- `ghcr.io/klask-dev/klask-frontend:main`
- `ghcr.io/klask-dev/klask-backend:latest` ⭐
- `ghcr.io/klask-dev/klask-frontend:latest` ⭐
- `ghcr.io/klask-dev/klask-backend:sha-abc1234`
- `ghcr.io/klask-dev/klask-frontend:sha-abc1234`

### ✅ **Tags de version**
```bash
git tag v1.2.3
git push origin v1.2.3
```
**Images créées :**
- `ghcr.io/klask-dev/klask-backend:v1.2.3` ⭐
- `ghcr.io/klask-dev/klask-frontend:v1.2.3` ⭐
- `ghcr.io/klask-dev/klask-backend:1.2.3`
- `ghcr.io/klask-dev/klask-frontend:1.2.3`
- `ghcr.io/klask-dev/klask-backend:1.2`
- `ghcr.io/klask-dev/klask-frontend:1.2`
- `ghcr.io/klask-dev/klask-backend:1`
- `ghcr.io/klask-dev/klask-frontend:1`

### ✅ **Pull Requests**
```bash
# Créer une PR
gh pr create
```
**Images créées :**
- `ghcr.io/klask-dev/klask-backend:pr-123`
- `ghcr.io/klask-dev/klask-frontend:pr-123`
- `ghcr.io/klask-dev/klask-backend:sha-abc1234`
- `ghcr.io/klask-dev/klask-frontend:sha-abc1234`

## 🔍 **Security Scan adaptatif**

| Contexte | Tag scanné |
|----------|------------|
| Branche feature | `feature-auth-system` |
| Branche main/master | `latest` |
| Tag v1.2.3 | `v1.2.3` |
| Pull Request | `pr-123` |

## 📦 **Helm Chart**

Publié automatiquement après chaque build réussi (toutes branches).

## 🎮 **Cas d'usage pratiques**

### **Développement avec test immédiat**
```bash
git checkout -b feature/new-ui
git push origin feature/new-ui
# → Images feature-new-ui disponibles immédiatement

helm install test oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=feature-new-ui \
  --set frontend.image.tag=feature-new-ui
```

### **Pull Request avec validation**
```bash
gh pr create
# → Images pr-123 + test sur branche
# → Double validation (branche + PR)
```

### **Production stable**
```bash
git push origin main
# → Images latest pour production

helm install klask oci://ghcr.io/klask-dev/klask
# Utilise automatiquement latest
```

### **Release officielle**
```bash
git tag v1.0.0
git push origin v1.0.0
# → Images v1.0.0 pour release

helm install klask oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=v1.0.0
```

## 🔧 **Configuration technique**

### **Triggers**
```yaml
on:
  push:
    branches: [ '**' ]    # Toutes les branches
    tags: [ 'v*' ]        # Tags de version
  pull_request:
    branches: [ main, master ]  # PRs vers main/master
```

### **Tags générés**
```yaml
tags: |
  type=ref,event=branch          # nom-de-branche
  type=ref,event=pr              # pr-123
  type=sha,prefix=sha-           # sha-abc1234 (fixé!)
  type=semver,pattern={{version}} # v1.2.3
  type=semver,pattern={{major}}.{{minor}} # 1.2
  type=semver,pattern={{major}}   # 1
  type=raw,value=latest,enable={{is_default_branch}} # latest
```

## 💡 **Avantages de cette approche**

✅ **Flexibilité maximale** : Test direct de n'importe quelle branche  
✅ **Feedback rapide** : Pas besoin de créer une PR pour tester  
✅ **Traçabilité** : Tags SHA pour identifier le code exact  
✅ **Standardisation** : `latest` pour main, versions pour releases  
✅ **CI robuste** : Build systématique = détection précoce des problèmes  

## ⚠️ **Considérations**

📊 **Builds fréquents** : Plus d'utilisation de CI, mais flexibilité max  
🏷️ **Beaucoup de tags** : Nettoyage périodique recommandé  
💾 **Storage GHCR** : Surveiller l'usage, nettoyer les anciens tags  

## 🎯 **Philosophie**

> "Build early, build often, test everywhere"

Cette stratégie privilégie la **vitesse de développement** et la **facilité de test** au détriment d'un peu plus d'utilisation des ressources CI/CD. C'est un excellent compromis pour une équipe qui veut pouvoir tester rapidement ses branches sans friction.