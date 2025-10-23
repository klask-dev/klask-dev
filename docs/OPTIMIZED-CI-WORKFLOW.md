# Workflow CI/CD Optimisé

## 🎯 Philosophie

Notre stratégie CI/CD optimise l'utilisation des ressources tout en maintenant la qualité :

- **Build uniquement quand nécessaire**
- **Security scan systématique pour le code qui sera mergé**
- **Images de production seulement pour du code validé**

## 🚀 Quand les images sont-elles construites ?

### ✅ **Pull Requests**
```bash
git checkout -b feature/new-search
git push origin feature/new-search
# Créer une PR → Images construites
```
**Pourquoi ?** Le code sera potentiellement mergé, il faut le valider.

### ✅ **Push sur main/master** 
```bash
git push origin main
```
**Pourquoi ?** Code de production, besoin du tag `latest`.

### ✅ **Tags de version**
```bash
git tag v1.2.3
git push origin v1.2.3
```
**Pourquoi ?** Release officielle.

### ❌ **Push sur branches features**
```bash
git push origin feature/new-search  # Sans PR
```
**Pourquoi pas ?** Développement privé, pas encore prêt à être validé.

## 💡 Avantages

### **Économies de ressources**
- Moins de builds inutiles
- Moins de storage utilisé dans GHCR
- CI/CD plus rapide pour tout le monde

### **Workflow naturel**
```bash
# 1. Développement privé (pas de build)
git checkout -b feature/auth
git commit -m "work in progress"
git push origin feature/auth

# 2. Code prêt → PR (build + test)
gh pr create --title "Add authentication"
# → Images pr-123 créées et testées

# 3. Merge → Production (latest)
gh pr merge 123
# → Images latest créées
```

### **Sécurité renforcée**
- Security scan seulement sur le code qui compte
- Pas d'images "expérimentales" en production
- Traçabilité claire : PR → Test → Merge → Production

## 🔧 Configuration technique

### **Triggers dans `.github/workflows/ci.yml`**
```yaml
on:
  push:
    branches: [ main, master ]  # Production uniquement
    tags: [ 'v*' ]              # Releases
  pull_request:
    branches: [ main, master ]  # Code candidat au merge
```

### **Images générées selon le contexte**

| Contexte | Tag généré | Cas d'usage |
|----------|------------|-------------|
| PR #123 | `pr-123` | Test/validation |
| Push main | `main`, `latest` | Production |
| Tag v1.2.3 | `v1.2.3`, `1.2`, `1` | Release stable |

## 🛠️ Cas d'usage

### **Développeur A - Feature branch**
```bash
# Travail en cours, pas besoin d'images
git checkout -b feature/search-filters
# ... développement ...
git push origin feature/search-filters  # Aucun build

# Prêt pour review
gh pr create  # → Images pr-456 créées pour tests
```

### **Développeur B - Hotfix urgent**
```bash
# Besoin de tester rapidement
git checkout -b hotfix/critical-bug
# ... fix ...
gh pr create  # → Images pr-789 créées immédiatement
```

### **Release Manager**
```bash
# Release en production
git tag v2.1.0
git push origin v2.1.0  # → Images v2.1.0 pour production
```

## 📊 Impact

### **Avant (build sur toutes les branches)**
- 50 branches × 2 images × 200MB = 20GB/mois
- 50 builds/jour
- CI saturé

### **Après (build sur PR + main + tags)**
- 10 PRs × 2 images × 200MB = 4GB/mois
- 15 builds/jour  
- CI fluide

## 🎮 Comment tester votre feature

### **Option 1: Pull Request (recommandé)**
```bash
gh pr create
# → Utiliser l'image pr-123 générée
helm install test oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=pr-123
```

### **Option 2: Build local**
```bash
# Pour tests rapides sans CI
docker build -t klask-backend:local klask-rs/
docker build -t klask-frontend:local klask-react/
```

## 🤝 Best Practices

1. **Créez des PRs tôt** pour bénéficier des images CI
2. **Utilisez des commits atomiques** pour des builds plus rapides
3. **Mergez rapidement** les PRs validées pour libérer les ressources
4. **Taggez régulièrement** pour des releases stables

Cette approche vous donne le contrôle total : build quand vous voulez tester/partager, pas de gaspillage pour le développement privé ! 🎯