# Guide de Configuration du Stockage PostgreSQL

## 📦 Options de StorageClass

### Option 1 : StorageClass par défaut (Recommandé)

```yaml
postgresql:
  persistence:
    enabled: true
    storageClass: ""  # Utilise le StorageClass par défaut
    size: 20Gi
```

**Avantages** :
- Portable entre clusters
- Fonctionne partout (K3s, EKS, GKE, AKS, etc.)
- Pas de configuration spécifique nécessaire

**Identifier votre SC par défaut** :
```bash
kubectl get storageclass
# Cherchez "(default)" dans la sortie
```

---

### Option 2 : StorageClass spécifique

```yaml
postgresql:
  persistence:
    enabled: true
    storageClass: "fast-ssd"  # Nom de votre SC
    size: 20Gi
```

**Cas d'usage** :
- Performances élevées requises (SSD NVMe)
- Multi-zone / Haute disponibilité
- Backup automatique (AWS EBS snapshots, etc.)

**Exemples par provider** :

#### AWS EKS
```yaml
storageClass: "gp3"  # AWS EBS GP3 (SSD performant)
# ou
storageClass: "io2"  # AWS EBS IO2 (IOPS provisionnées)
```

#### Google GKE
```yaml
storageClass: "standard-rwo"  # Persistent Disk Standard
# ou
storageClass: "premium-rwo"   # SSD Persistent Disk
```

#### Azure AKS
```yaml
storageClass: "managed-premium"  # Azure Premium SSD
```

#### Bare Metal / On-Premise
```yaml
storageClass: "rook-ceph-block"  # Ceph RBD
# ou
storageClass: "nfs-client"       # NFS provisioner
```

---

### Option 3 : Persistence désactivée (Dev/Test uniquement)

```yaml
postgresql:
  persistence:
    enabled: false
```

**⚠️ Attention** : Les données sont perdues au redémarrage du pod !

**Cas d'usage** :
- Environnement de test éphémère
- CI/CD pipelines
- Démonstrations

---

## 🔧 Migration de StorageClass

Si vous devez changer de StorageClass après installation :

```bash
# 1. Backup de la base de données
kubectl exec -n klask klask-postgresql-0 -- \
  pg_dump -U klask klask > backup-$(date +%Y%m%d).sql

# 2. Supprimer le StatefulSet (garde le PVC)
kubectl delete statefulset klask-postgresql -n klask

# 3. Supprimer le PVC
kubectl delete pvc data-klask-postgresql-0 -n klask

# 4. Upgrade avec le nouveau StorageClass
helm upgrade klask . \
  --set postgresql.persistence.storageClass=nouveau-sc \
  --namespace klask

# 5. Restaurer les données
kubectl exec -i -n klask klask-postgresql-0 -- \
  psql -U klask klask < backup-$(date +%Y%m%d).sql
```

---

## 📊 Dimensionnement

### Sizing Guide

| Environnement | Taille Recommandée | IOPS | Raison |
|---------------|-------------------|------|---------|
| Dev/Test      | 8-10 Gi          | 100  | Petit jeu de données |
| Staging       | 20-50 Gi         | 500  | Données de test réalistes |
| Production Small | 50-100 Gi      | 1000 | < 1M documents indexés |
| Production Medium | 100-500 Gi    | 3000 | 1M-10M documents |
| Production Large | 500Gi-1Ti      | 5000+ | > 10M documents |

### Calcul de la taille

```
Taille estimée = (Nombre de documents × 1KB) × 1.5 (overhead)
```

**Exemple** : 5M documents → 5GB × 1.5 = **7.5GB minimum**

---

## 🚨 Troubleshooting

### Erreur: "change mount propagation through procfd"

**Cause** : StorageClass incompatible (ex: `kubernetes.io/no-provisioner`)

**Solution** :
```bash
# Vérifier le StorageClass du PVC
kubectl get pvc -n klask -o wide

# Changer pour un provisioner dynamique
helm upgrade klask . --set postgresql.persistence.storageClass=""
```

### PVC en état "Pending"

**Cause** : Pas de StorageClass disponible

**Diagnostic** :
```bash
kubectl describe pvc data-klask-postgresql-0 -n klask
kubectl get storageclass
```

**Solution** : Installer un provisioner (ex: Rancher Local Path, NFS, Rook)

### Performances lentes

**Diagnostic** :
```bash
# Vérifier les IOPS du volume
kubectl describe pvc -n klask | grep -A 5 "iops"

# Tester les performances dans le pod
kubectl exec -it -n klask klask-postgresql-0 -- \
  dd if=/dev/zero of=/var/lib/postgresql/data/test bs=1M count=1000
```

**Solution** : Utiliser un StorageClass avec plus d'IOPS (Premium SSD)

---

## ✅ Best Practices

1. **Toujours activer la persistence en production**
   ```yaml
   persistence:
     enabled: true  # JAMAIS false en prod
   ```

2. **Utiliser le StorageClass par défaut sauf besoin spécifique**
   ```yaml
   storageClass: ""  # Simple et portable
   ```

3. **Prévoir 2-3x la taille de vos données**
   - Croissance future
   - WAL logs PostgreSQL
   - Indexes et statistiques

4. **Tester la restauration régulièrement**
   ```bash
   # Automatiser les backups
   kubectl create cronjob postgres-backup --image=postgres:18-alpine \
     --schedule="0 2 * * *" -- \
     pg_dump -h klask-postgresql -U klask klask > /backup/klask-$(date +%Y%m%d).sql
   ```

5. **Monitoring du stockage**
   ```yaml
   # Ajouter des alertes Prometheus
   - alert: PostgreSQLDiskAlmostFull
     expr: kubelet_volume_stats_available_bytes / kubelet_volume_stats_capacity_bytes < 0.1
     for: 5m
   ```

---

## 🔗 Références

- [Kubernetes Storage Classes](https://kubernetes.io/docs/concepts/storage/storage-classes/)
- [PostgreSQL on Kubernetes Best Practices](https://www.postgresql.org/docs/current/kernel-resources.html)
- [StatefulSet Volume Management](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/#stable-storage)
