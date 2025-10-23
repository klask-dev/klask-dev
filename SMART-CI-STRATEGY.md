# Stratégie CI/CD intelligente

## 🎯 **Philosophie**

**Tests partout, builds seulement quand nécessaire**

## 🔄 **Comportement selon le contexte**

### ✅ **Branches sans PR** - Tests seulement
```bash
git checkout -b feature/auth-system
git push origin feature/auth-system  # Sans créer de PR
```
**Actions :**
- ✅ Tests frontend (npm test, typecheck)
- ✅ Tests backend (cargo test, clippy, format)
- ❌ Pas de build d'images
- ❌ Pas de security scan
- ❌ Pas de publication Helm

**Résultat :** Feedback rapide sur la qualité du code sans consommer de ressources

### ✅ **Pull Requests** - Tests + Build complet
```bash
gh pr create --title "Add authentication"
```
**Actions :**
- ✅ Tests frontend + backend
- ✅ Build et push des images Docker
- ✅ Security scan (Trivy)
- ❌ Pas de publication Helm (seulement pour main/master)

**Images créées :**
- `ghcr.io/klask-dev/klask-backend:pr-123`
- `ghcr.io/klask-dev/klask-frontend:pr-123`
- `ghcr.io/klask-dev/klask-backend:sha-abc1234`
- `ghcr.io/klask-dev/klask-frontend:sha-abc1234`

### ✅ **Push sur main/master** - Tout + Latest
```bash
git push origin main
```
**Actions :**
- ✅ Tests frontend + backend
- ✅ Build et push des images Docker
- ✅ Security scan (Trivy)
- ✅ Publication Helm chart

**Images créées :**
- `ghcr.io/klask-dev/klask-backend:main`
- `ghcr.io/klask-dev/klask-frontend:main`
- `ghcr.io/klask-dev/klask-backend:latest` ⭐
- `ghcr.io/klask-dev/klask-frontend:latest` ⭐
- `ghcr.io/klask-dev/klask-backend:sha-abc1234`
- `ghcr.io/klask-dev/klask-frontend:sha-abc1234`

### ✅ **Tags de version** - Release complète
```bash
git tag v1.2.3
git push origin v1.2.3
```
**Actions :**
- ✅ Tests frontend + backend
- ✅ Build et push des images Docker
- ✅ Security scan (Trivy)
- ❌ Pas de publication Helm (chart versionné manuellement)

**Images créées :**
- `ghcr.io/klask-dev/klask-backend:v1.2.3` ⭐
- `ghcr.io/klask-dev/klask-frontend:v1.2.3` ⭐
- `ghcr.io/klask-dev/klask-backend:1.2.3`
- `ghcr.io/klask-dev/klask-frontend:1.2.3`
- `ghcr.io/klask-dev/klask-backend:1.2`
- `ghcr.io/klask-dev/klask-frontend:1.2`
- `ghcr.io/klask-dev/klask-backend:1`
- `ghcr.io/klask-dev/klask-frontend:1`

## 🚀 **Workflow pratique**

### **Développement itératif**
```bash
# 1. Développement avec feedback rapide
git checkout -b feature/new-search
git commit -m "initial implementation"
git push origin feature/new-search
# → Tests seulement, pas d'images (économie de ressources)

# 2. Plusieurs itérations avec tests
git commit -m "fix tests"
git push origin feature/new-search
# → Tests seulement, validation continue

# 3. Prêt pour review → Build complet
gh pr create --title "Add new search functionality"
# → Tests + Images pr-XXX pour validation complète

# 4. Merge → Production
gh pr merge
# → Tests + Images latest + Helm chart
```

### **Hotfix rapide**
```bash
git checkout -b hotfix/critical-bug
gh pr create --title "Fix critical security issue"
# → Build immédiat avec images pr-XXX pour tests
```

## 🎮 **Comment tester selon le contexte**

### **Branch en développement (tests seulement)**
```bash
# Option 1: Build local pour test rapide
docker build -t klask-backend:local klask-rs/
docker build -t klask-frontend:local klask-react/

# Option 2: Créer une PR pour avoir les images CI
gh pr create --draft
```

### **Pull Request (images disponibles)**
```bash
# Utiliser les images de la PR
helm install test-pr oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=pr-123 \
  --set frontend.image.tag=pr-123
```

### **Production (latest)**
```bash
# Utiliser les images latest de main
helm install klask oci://ghcr.io/klask-dev/klask
```

### **Release (version)**
```bash
# Utiliser une version spécifique
helm install klask oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=v1.2.3 \
  --set frontend.image.tag=v1.2.3
```

## 🔧 **Configuration technique**

### **Conditions de build**
```yaml
# Build seulement pour:
if: |
  github.event_name == 'pull_request' ||
  (github.event_name == 'push' && (
    github.ref == 'refs/heads/main' ||
    github.ref == 'refs/heads/master' ||
    startsWith(github.ref, 'refs/tags/v')
  ))
```

### **Jobs exécutés selon le contexte**

| Contexte | Tests | Build | Security | Helm |
|----------|-------|-------|----------|------|
| Branch sans PR | ✅ | ❌ | ❌ | ❌ |
| Pull Request | ✅ | ✅ | ✅ | ❌ |
| Push main/master | ✅ | ✅ | ✅ | ✅ |
| Tag version | ✅ | ✅ | ✅ | ❌ |

## 💡 **Avantages**

✅ **Feedback rapide** : Tests immédiats sur toute branche  
✅ **Économie de ressources** : Images seulement quand nécessaire  
✅ **Pas de double build** : Logique claire et sans ambiguïté  
✅ **Security appropriée** : Scan seulement sur le code qui compte  
✅ **Helm intelligent** : Publication seulement pour main/master  
✅ **Developer-friendly** : Tests partout, builds quand utile  

## 📊 **Comparaison avec les autres stratégies**

| Stratégie | Builds/jour | Storage utilisé | Feedback |
|-----------|-------------|-----------------|----------|
| Build sur tout | 50+ | Élevé | Excellent |
| Build sur PR seulement | 10 | Faible | Moyen |
| **Smart (cette approche)** | **15** | **Optimal** | **Excellent** |

## 🎯 **Résumé**

Cette approche donne le meilleur des deux mondes :
- **Tests rapides** pour le développement quotidien
- **Builds complets** seulement quand on veut vraiment déployer/tester
- **Pas de gaspillage** de ressources CI/CD
- **Flexibilité** pour créer une PR quand on veut des images