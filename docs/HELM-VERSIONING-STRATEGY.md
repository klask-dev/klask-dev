# Stratégie de versioning Helm Charts

## 🎯 **Versioning automatique des charts**

Le workflow Helm génère automatiquement des versions selon le contexte.

## 📦 **Versions générées**

### **Pull Requests**
```bash
# Base version dans Chart.yaml: 0.1.0
# PR #123 → Chart version: 0.1.0-pr123

gh pr create
# → oci://ghcr.io/klask-dev/klask:0.1.0-pr123
```

### **Push sur main/master**
```bash
# Base version dans Chart.yaml: 0.1.0
# Main → Chart version: 0.1.0 (stable)

git push origin main
# → oci://ghcr.io/klask-dev/klask:0.1.0
```

### **Tags de version**
```bash
# Tag v1.2.3 → Chart version: 1.2.3

git tag v1.2.3
git push origin v1.2.3
# → oci://ghcr.io/klask-dev/klask:1.2.3
```

### **Déclenchement manuel**
```bash
# Via GitHub Actions interface
# → oci://ghcr.io/klask-dev/klask:0.1.0-dev
```

## 🔧 **Logique de versioning**

### **Configuration dans Chart.yaml**
```yaml
# Base version (à incrémenter manuellement pour releases)
version: 0.1.0
```

### **Versions automatiques**
| Contexte | Version générée | Exemple |
|----------|----------------|---------|
| PR #123 | `base-pr123` | `0.1.0-pr123` |
| Main/Master | `base` | `0.1.0` |
| Tag v1.2.3 | `1.2.3` | `1.2.3` |
| Manuel | `base-dev` | `0.1.0-dev` |

## 🚀 **Utilisation**

### **Tester votre PR**
```bash
# Après création de PR #123
helm install test-pr oci://ghcr.io/klask-dev/klask:0.1.0-pr123
```

### **Déploiement staging**
```bash
# Dernière version stable de main
helm install staging oci://ghcr.io/klask-dev/klask:0.1.0
```

### **Déploiement production**
```bash
# Version release spécifique
helm install prod oci://ghcr.io/klask-dev/klask:1.2.3
```

### **Développement local**
```bash
# Test manuel sans CI
helm install local ./charts/klask
```

## 🔄 **Workflow de release**

### **1. Développement avec PR**
```bash
git checkout -b feature/awesome-feature
gh pr create
# → Chart 0.1.0-pr456 créé pour tests
```

### **2. Merge vers main**
```bash
gh pr merge
# → Chart 0.1.0 stable créé
```

### **3. Release officielle**
```bash
# Incrémenter manuellement dans Chart.yaml
sed -i 's/version: 0.1.0/version: 0.2.0/' charts/klask/Chart.yaml
git commit -m "bump: chart version to 0.2.0"
git push origin main

# Créer le tag de release
git tag v0.2.0
git push origin v0.2.0
# → Chart 0.2.0 de release créé
```

## 🎮 **Commandes utiles**

### **Lister les versions disponibles**
```bash
# Via GHCR API
curl -H "Authorization: Bearer $GITHUB_TOKEN" \
  https://ghcr.io/v2/klask-dev/klask/tags/list
```

### **Voir les détails d'une version**
```bash
helm show chart oci://ghcr.io/klask-dev/klask:0.1.0-pr123
```

### **Télécharger un chart**
```bash
helm pull oci://ghcr.io/klask-dev/klask:0.1.0-pr123
```

### **Déclenchement manuel du workflow Helm**
```bash
# Via l'interface GitHub:
# Actions → Publish Helm Chart → Run workflow
```

## 💡 **Avantages**

✅ **Versioning automatique** : Pas de gestion manuelle  
✅ **Traçabilité** : Chaque PR a sa version  
✅ **Tests faciles** : Chart disponible immédiatement après PR  
✅ **Releases propres** : Versions sémantiques pour production  
✅ **Flexibilité** : Déclenchement manuel possible  

## 📋 **Bonnes pratiques**

### **Versioning de Chart.yaml**
- Incrémenter la version manuellement pour les releases majeures
- Garder la version base stable entre les releases
- Utiliser semantic versioning (0.1.0 → 0.2.0 → 1.0.0)

### **Tests de charts**
```bash
# Toujours tester le chart de votre PR avant merge
helm install test-pr oci://ghcr.io/klask-dev/klask:0.1.0-pr123

# Vérifier que ça fonctionne
kubectl get pods
kubectl get services

# Nettoyer après test
helm uninstall test-pr
```

### **Déploiements**
- **Staging** : Utiliser les versions stables de main
- **Production** : Utiliser uniquement les versions taggées
- **Tests** : Utiliser les versions PR

Cette stratégie vous donne un contrôle total sur les versions Helm tout en automatisant le processus ! 🚀