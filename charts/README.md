# Klask Helm Chart - OCI Installation

Ce répertoire contient le Helm chart pour déployer Klask sur Kubernetes. Le chart est automatiquement publié dans GitHub Container Registry (GHCR) via l'infrastructure CI/CD existante.

## 🏗️ Infrastructure CI/CD existante

Le projet dispose déjà d'une infrastructure complète :
- **`ci.yml`**: Build et test des images Docker (frontend/backend)
- **`deploy.yml`**: Déploiement staging/production 
- **`release.yml`**: Publication des releases avec SBOM
- **`build-and-publish.yml`**: Publication du Helm chart OCI *(nouveau)*

## 🚀 Installation via OCI Registry

### Installation simple
```bash
helm install klask oci://ghcr.io/klask-dev/klask --version 0.1.0
```

### Installation avec configuration personnalisée
```bash
# Créer un fichier de valeurs personnalisées
cat > my-values.yaml << EOF
ingress:
  enabled: true
  hosts:
    - host: klask.mondomaine.com
      paths:
        - path: /
          pathType: Prefix

backend:
  resources:
    requests:
      cpu: 200m
      memory: 256Mi
    limits:
      cpu: 1000m
      memory: 1Gi

frontend:
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 500m
      memory: 512Mi
EOF

# Installer avec les valeurs personnalisées
helm install klask oci://ghcr.io/klask-dev/klask --version 0.1.0 -f my-values.yaml
```

## 📦 Ce qui est déployé

Le chart déploie automatiquement :

- **Backend Rust** (`klask-backend`) sur le port 3000
- **Frontend React** (`klask-frontend`) sur le port 8080  
- **PostgreSQL** sur le port 5432
- **Services Kubernetes** pour la communication interne
- **Ingress** (optionnel) pour l'accès externe

## 🔧 Configuration

### Variables principales

| Paramètre | Description | Défaut |
|-----------|-------------|--------|
| `backend.image.repository` | Image Docker du backend | `ghcr.io/klask-dev/klask-backend` |
| `frontend.image.repository` | Image Docker du frontend | `ghcr.io/klask-dev/klask-frontend` |
| `postgresql.enabled` | Activer PostgreSQL intégré | `true` |
| `ingress.enabled` | Activer l'ingress | `false` |

### Base de données externe

Pour utiliser une base de données externe (AWS RDS, Google Cloud SQL, etc.) :

```yaml
# Désactiver PostgreSQL intégré
postgresql:
  enabled: false

# Configurer la connexion externe
backend:
  env:
    - name: DATABASE_URL
      value: "postgresql://user:pass@external-db:5432/klask"
```

## 🔄 Processus CI/CD (intégré à l'existant)

### Workflows existants
1. **`ci.yml`**: Build, test et publication des images Docker dans GHCR
2. **`deploy.yml`**: Déploiement automatique staging → production
3. **`release.yml`**: Publication des releases avec SBOM

### Nouveau workflow Helm
4. **`build-and-publish.yml`**: Publication du chart OCI
   - Se déclenche après succès de `ci.yml` 
   - Package et publie le chart dans GHCR
   - Met à jour automatiquement les références d'images

### Images Docker (via ci.yml existant)
Les images sont publiées automatiquement :
- `ghcr.io/klask-dev/klask-backend:latest`
- `ghcr.io/klask-dev/klask-frontend:latest`
- Tags sémantiques pour les releases

## 📋 Commandes utiles

### Vérifier le statut du déploiement
```bash
kubectl get pods -l "app.kubernetes.io/instance=klask"
kubectl get services -l "app.kubernetes.io/instance=klask"
```

### Accéder à l'application localement
```bash
kubectl port-forward service/klask-frontend 8080:8080
# Ouvrir http://localhost:8080
```

### Voir les logs
```bash
# Backend
kubectl logs -l "app.kubernetes.io/component=backend" -f

# Frontend  
kubectl logs -l "app.kubernetes.io/component=frontend" -f

# PostgreSQL
kubectl logs -l "app.kubernetes.io/name=postgresql" -f
```

### Mettre à jour le déploiement
```bash
helm upgrade klask oci://ghcr.io/klask-dev/klask --version 0.2.0
```

### Désinstaller
```bash
helm uninstall klask
```

## 🐛 Dépannage

### Le backend ne démarre pas
1. Vérifier que PostgreSQL est prêt :
   ```bash
   kubectl get pods -l "app.kubernetes.io/name=postgresql"
   ```

2. Vérifier la configuration de la base de données :
   ```bash
   kubectl logs -l "app.kubernetes.io/component=backend" --tail=50
   ```

### Le frontend ne charge pas
1. Vérifier que le backend est accessible :
   ```bash
   kubectl get svc klask-backend
   ```

2. Tester la connectivité :
   ```bash
   kubectl exec -it deployment/klask-frontend -- wget -qO- http://klask-backend:3000/api/status
   ```

### Problèmes d'ingress
1. Vérifier la classe d'ingress :
   ```bash
   kubectl get ingressclass
   ```

2. Vérifier les événements :
   ```bash
   kubectl describe ingress klask
   ```

## 🔐 Sécurité

- Les mots de passe PostgreSQL par défaut sont pour le développement uniquement
- En production, utilisez des secrets Kubernetes :
  ```yaml
  postgresql:
    auth:
      existingSecret: "my-postgres-secret"
  ```

## 📈 Monitoring

Pour activer le monitoring (Prometheus/Grafana) :

```yaml
backend:
  env:
    - name: METRICS_ENABLED
      value: "true"
  
  service:
    annotations:
      prometheus.io/scrape: "true"
      prometheus.io/port: "3000"
      prometheus.io/path: "/metrics"
```