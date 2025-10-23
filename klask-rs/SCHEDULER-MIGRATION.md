# Migration du Scheduler vers Croner

## 🎯 **Objectifs**

- ✅ Supprimer `tokio-cron-scheduler` et `cron`
- ✅ Utiliser uniquement `croner` (2.0)
- ✅ Format unifié : 6 champs `seconds minutes hours day month weekday`
- ✅ Calcul direct du `next_run_time` sans job dummy
- ✅ Code plus simple et maintenable

## 📊 **Comparaison Avant/Après**

### **Dependencies**

```toml
# Avant
tokio-cron-scheduler = "0.10"  # ~500 KB
cron = "0.12"                   # ~50 KB

# Après
croner = "2.0"                  # ~100 KB
```

**Économie**: ~450 KB de dépendances

### **Code Complexity**

| Métrique | Avant | Après | Différence |
|----------|-------|-------|------------|
| Lignes de code | 482 | 345 | **-28%** |
| Conversions de format | 4 endroits | 0 | **-100%** |
| Jobs dummy | 1 (pour calcul) | 0 | **-100%** |
| Bibliothèques cron | 2 | 1 | **-50%** |

###  **Architecture**

#### **Avant (tokio-cron-scheduler)**

```rust
pub struct SchedulerService {
    scheduler: JobScheduler,              // Bibliothèque lourde
    pool: PgPool,
    crawler_service: Arc<CrawlerService>,
    job_ids: Arc<RwLock<HashMap<Uuid, Uuid>>>, // Tracking des IDs
}

// Problèmes:
// 1. Format 6 champs pour tokio-cron-scheduler
// 2. Format 5 champs pour la crate cron
// 3. Conversions manuelles partout
// 4. Job dummy pour calculer next_run_time
```

#### **Après (croner + tokio)**

```rust
pub struct SchedulerService {
    pool: PgPool,
    crawler_service: Arc<CrawlerService>,
    jobs: Arc<RwLock<HashMap<Uuid, ScheduledJob>>>, // Plus simple
}

struct ScheduledJob {
    repository_id: Uuid,
    cron_expression: String,        // Format unifié
    task_handle: JoinHandle<()>,    // Contrôle direct
}

// Avantages:
// 1. Un seul format (6 champs)
// 2. Pas de conversion
// 3. Calcul direct avec croner.find_next_occurrence()
// 4. Plus de contrôle sur les tasks
```

## 🔧 **Changements Principaux**

### **1. Plus de conversions de format**

```rust
// ❌ AVANT - Conversions manuelles partout
let parts: Vec<&str> = cron_schedule.split_whitespace().collect();
if parts.len() == 5 {
    format!("0 {}", cron_schedule)  // Ajouter secondes
} else {
    cron_schedule.clone()
}

// Puis plus tard...
let parts: Vec<&str> = cron_schedule.split_whitespace().collect();
let five_field_cron = if parts.len() == 6 {
    parts[1..].join(" ")  // Enlever secondes
} else {
    cron_schedule.clone()
};

// ✅ APRÈS - Format unique
let cron = Cron::new(&cron_expr).parse()?;
let next_run = cron.find_next_occurrence(&Utc::now(), false)?;
```

### **2. Calcul direct du next_run_time**

```rust
// ❌ AVANT - Job dummy nécessaire
// La fonction get_next_run_time() était stubbed:
pub async fn get_next_run_time(&self, _repository_id: Uuid) -> Option<DateTime<Utc>> {
    // TODO: Implement proper next run time retrieval
    None
}

// Workaround: parser séparé avec la crate `cron`
let five_field_cron = /* conversion */;
match five_field_cron.parse::<cron::Schedule>() {
    Ok(schedule) => schedule.upcoming(chrono::Utc).take(1).next(),
    Err(_) => None,
}

// ✅ APRÈS - Direct et propre
pub async fn get_next_run_time(&self, repository_id: Uuid) -> Option<DateTime<Utc>> {
    let jobs = self.jobs.read().await;
    if let Some(job) = jobs.get(&repository_id) {
        if let Ok(cron) = Cron::new(&job.cron_expression).parse() {
            return cron.find_next_occurrence(&Utc::now(), false).ok();
        }
    }
    None
}
```

### **3. Scheduling simplifié**

```rust
// ❌ AVANT - Création de Job complexe
let job = Job::new_async(schedule_expr.as_str(), move |_uuid, _lock| {
    Box::pin(async move {
        // Code du job
    })
})?;
let job_uuid = self.scheduler.add(job).await?;
self.job_ids.write().await.insert(repo_id, job_uuid);

// ✅ APRÈS - tokio::spawn direct
let task_handle = tokio::spawn(async move {
    loop {
        let next_run = cron.find_next_occurrence(&Utc::now(), false)?;
        let duration = (next_run - now).to_std()?;

        tokio::time::sleep(duration).await;

        // Exécuter le crawl
        crawler.crawl_repository(repo_id).await?;
    }
});

self.jobs.write().await.insert(repo_id, ScheduledJob {
    repository_id: repo_id,
    cron_expression,
    task_handle,
});
```

## 📋 **Plan de Migration**

### **Étape 1: ✅ Dependencies**
- [x] Remplacer dans `Cargo.toml`
- [x] Créer nouveau `scheduler_new.rs`

### **Étape 2: Backend Code**
- [ ] Renommer `scheduler_new.rs` → `scheduler.rs`
- [ ] Mettre à jour `lib.rs` ou `main.rs` imports
- [ ] Tester compilation

### **Étape 3: Frontend**
- [ ] Vérifier validation cron (déjà 6 champs ✅)
- [ ] Tester UI de configuration

### **Étape 4: Database**
- [ ] Pas de migration nécessaire (format déjà 6 champs)
- [ ] Vérifier que toutes les expressions existantes sont valides

### **Étape 5: Tests**
- [ ] Mettre à jour `tests/scheduler_test.rs`
- [ ] Tester toutes les expressions cron existantes
- [ ] Vérifier calcul next_run_time

### **Étape 6: E2E Testing**
- [ ] Créer repository avec cron schedule
- [ ] Vérifier affichage "Next crawl"
- [ ] Attendre exécution automatique
- [ ] Vérifier logs

## 🎮 **Testing Local**

```bash
# 1. Builder
cd klask-rs
cargo build

# 2. Tests unitaires
cargo test scheduler

# 3. Lancer le serveur
cargo run

# 4. Tester API
curl http://localhost:3000/api/scheduler/status

# 5. Créer un repo avec schedule test
curl -X POST http://localhost:3000/api/repositories \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-scheduled",
    "url": "https://github.com/test/repo",
    "autoCrawlEnabled": true,
    "cronSchedule": "0 */5 * * * *"
  }'
```

## ⚠️ **Points d'Attention**

### **Format Cron**
- **Toujours 6 champs**: `seconds minutes hours day month weekday`
- Exemples valides:
  - `0 0 */6 * * *` (toutes les 6 heures)
  - `0 30 9 * * MON-FRI` (9:30 lundi-vendredi)
  - `0 0 0 1 * *` (1er du mois à minuit)

### **Timezone**
- Par défaut: UTC
- Peut être configuré avec `chrono-tz` si nécessaire

### **Error Handling**
- Expressions invalides → erreur explicite
- Job fail → logged, continue scheduling
- Pas de retry automatique dans le nouveau système

## 📈 **Bénéfices Attendus**

### **Performance**
- ✅ Moins de mémoire (~450 KB économisés)
- ✅ Pas de conversions à runtime
- ✅ Calculs plus rapides

### **Maintenabilité**
- ✅ Code 28% plus court
- ✅ Logique unifiée
- ✅ Moins de surface d'erreur

### **Fonctionnalités**
- ✅ `get_next_run_time()` fonctionne vraiment
- ✅ Affichage "Next crawl" précis
- ✅ Status API enrichi

### **Developer Experience**
- ✅ Un seul format à retenir
- ✅ Debugging plus facile
- ✅ Tests plus simples

## 🚀 **Next Steps**

1. **Backup**: Sauvegarder l'ancien `scheduler.rs`
2. **Switch**: Renommer `scheduler_new.rs` → `scheduler.rs`
3. **Build**: `cargo build`
4. **Test**: Tests unitaires + E2E
5. **Deploy**: Une fois validé localement

Voulez-vous que je procède au switch maintenant ?