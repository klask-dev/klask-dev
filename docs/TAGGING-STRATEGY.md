# Stratégie de build et tagging des images Docker

Ce document explique quand et comment les images Docker sont construites et taggées.

## 🚀 Déclencheurs de build

### **Pull Requests (développement)**
```bash
git checkout -b feature/auth-system
git push origin feature/auth-system
# Créer une PR vers main
```
**Images créées :**
- `ghcr.io/klask-dev/klask-backend:pr-123`
- `ghcr.io/klask-dev/klask-frontend:pr-123`
- `ghcr.io/klask-dev/klask-backend:sha-abc1234`
- `ghcr.io/klask-dev/klask-frontend:sha-abc1234`
- Security scan exécuté

### **❌ Push sur branches features (aucun build)**
```bash
git push origin feature/auth-system  # Sans PR
```
**Aucune image créée** - Économise les ressources CI/CD

### **Pour les branches principales (main/master)**
```bash
git push origin main
```
**Images créées :**
- `ghcr.io/klask-dev/klask-backend:main`
- `ghcr.io/klask-dev/klask-frontend:main`
- `ghcr.io/klask-dev/klask-backend:latest` ⭐
- `ghcr.io/klask-dev/klask-frontend:latest` ⭐
- `ghcr.io/klask-dev/klask-backend:main-sha1234567`
- `ghcr.io/klask-dev/klask-frontend:main-sha1234567`

### **Pour les tags de version (releases)**
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

### **Pour les Pull Requests**
```bash
# Automatique lors de la création d'une PR
```
**Images créées :**
- `ghcr.io/klask-dev/klask-backend:pr-123`
- `ghcr.io/klask-dev/klask-frontend:pr-123`

## 🔍 Security Scan

Le security scan (Trivy) utilise automatiquement le bon tag :

| Contexte | Tag scanné | Exemple |
|----------|------------|---------|
| Branche feature | `feature-auth-system` | `klask-backend:feature-auth-system` |
| Branche main/master | `latest` | `klask-backend:latest` |
| Tag de version | `v1.2.3` | `klask-backend:v1.2.3` |
| Pull Request | `pr-123` | `klask-backend:pr-123` |

## 📦 Helm Chart

Le Helm chart est publié automatiquement après chaque build réussi, quelle que soit la branche.

| Contexte | Chart publié |
|----------|--------------|
| Toute branche | `oci://ghcr.io/klask-dev/klask:0.1.0` |
| Tag de version | `oci://ghcr.io/klask-dev/klask:1.2.3` |

## 🚀 Utilisation

### **Pour le développement**
```bash
# Utiliser l'image de votre branche
helm install klask-dev oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=feature-auth-system \
  --set frontend.image.tag=feature-auth-system
```

### **Pour la production**
```bash
# Utiliser la version stable
helm install klask oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=v1.2.3 \
  --set frontend.image.tag=v1.2.3
```

### **Pour les tests avec latest**
```bash
# Utiliser la dernière version de main
helm install klask-staging oci://ghcr.io/klask-dev/klask \
  --set backend.image.tag=latest \
  --set frontend.image.tag=latest
```

## 🔧 Configuration technique

Cette stratégie est configurée via `docker/metadata-action` dans `.github/workflows/ci.yml` :

```yaml
tags: |
  type=ref,event=branch          # nom-de-branche
  type=ref,event=pr              # pr-123
  type=sha,prefix={{branch}}-    # branche-sha1234567
  type=semver,pattern={{version}} # v1.2.3
  type=semver,pattern={{major}}.{{minor}} # 1.2
  type=semver,pattern={{major}}  # 1
  type=raw,value=latest,enable={{is_default_branch}} # latest pour main/master
```

## 📋 Avantages

✅ **Flexibilité** : Chaque branche a ses propres images  
✅ **Traçabilité** : Tags avec SHA pour identifier précisément le code  
✅ **Stabilité** : `latest` seulement pour les branches stables  
✅ **Releases** : Versioning sémantique pour les releases  
✅ **Sécurité** : Scan adaptatif selon le contexte  
✅ **CI/CD simple** : Pas de configuration manuelle des branches