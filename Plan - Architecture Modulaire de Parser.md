# Plan: Architecture Modulaire de Parsers pour Klask

## Objectif
Créer une infrastructure extensible de parsers basée sur les MIME types pour supporter différents formats de fichiers (texte maintenant, PDF/DOCX plus tard).

## Décisions Clés

| Décision | Choix |
|----------|-------|
| Lib MIME | `mimetype-detector` v0.3.4 (450+ formats, zero deps, thread-safe, `MimeKind` enum) |
| Scope Phase 1 | Infrastructure seule (pas de PDF/DOCX) |
| Features Cargo | Opt-in pour les parsers binaires |
| Input parsers | `&[u8]` (bytes) pour supporter binaires |
| Async | Sync parsers + `spawn_blocking` au call site |
| Nom dispatcher | `ParserDispatcher` (pattern dispatcher) |

---

## Phase 1: Infrastructure Parser

### Étape 1.1 - Créer le module parser
**Nouveaux fichiers à créer:**
```
klask-rs/src/services/parser/
├── mod.rs           # Parser trait + exports
├── dispatcher.rs    # ParserDispatcher (sélectionne le bon parser)
├── text_parser.rs   # Parser texte/code
└── error.rs         # ParseError enum
```

### Étape 1.2 - Définir le trait Parser

```rust
// parser/mod.rs
use mimetype_detector::MimeKind;

pub trait Parser: Send + Sync {
    fn name(&self) -> &'static str;

    /// Catégories MIME supportées (MimeKind::Text, MimeKind::Document, etc.)
    fn supported_kinds(&self) -> &[MimeKind];

    /// Extensions supportées (fallback si MIME detection échoue)
    fn supported_extensions(&self) -> &[&'static str];

    fn parse(&self, content: &[u8], file_path: &str) -> Result<ParsedContent, ParseError>;

    fn priority(&self) -> i32 { 0 }
}

pub struct ParsedContent {
    pub text: String,
    pub mime_type: String,
    pub metadata: Option<FileMetadata>,
}
```

### Étape 1.3 - Implémenter le ParserDispatcher

```rust
// parser/dispatcher.rs
use mimetype_detector::{detect, MimeKind};

pub static PARSER_DISPATCHER: Lazy<ParserDispatcher> = Lazy::new(ParserDispatcher::new);

impl ParserDispatcher {
    /// Trouve le parser approprié basé sur MimeKind
    pub fn find_parser(&self, content: &[u8], extension: Option<&str>) -> Option<Arc<dyn Parser>> {
        // 1. Détecter le MimeKind via mimetype-detector
        if let Some(mime) = detect(content) {
            let kind = mime.kind();
            for parser in &self.parsers {
                if parser.supported_kinds().contains(&kind) {
                    return Some(parser.clone());
                }
            }
        }
        // 2. Fallback sur extension
        // ...
    }

    pub fn parse(&self, content: &[u8], file_path: &str) -> Result<ParsedContent, ParseError>;
    pub fn is_supported(&self, content: &[u8], extension: Option<&str>) -> bool;
}
```

### Étape 1.4 - Implémenter TextParser

Reprend la liste `SUPPORTED_EXTENSIONS` actuelle et gère tous les fichiers texte/code.

---

## Phase 2: Intégration au Crawler

### Étape 2.1 - Modifier `git_tree_walker.rs`

**Fichier:** `klask-rs/src/services/crawler/git_tree_walker.rs`

Ajouter:
```rust
/// Read blob as raw bytes (new)
pub fn read_blob_bytes(repo: &gix::Repository, oid: &ObjectId) -> Result<Vec<u8>>;

/// Read and parse blob content (new)
pub fn read_and_parse_blob(repo: &gix::Repository, oid: &ObjectId, file_path: &str)
    -> Result<Option<ParsedContent>>;
```

### Étape 2.2 - Modifier `FileProcessor`

**Fichier:** `klask-rs/src/services/crawler/file_processing.rs`

Changer signature:
```rust
// Avant
pub async fn process_single_file(..., provided_content: Option<String>) -> Result<()>

// Après
pub async fn process_single_file(..., provided_content: Option<ParsedContent>) -> Result<()>
```

### Étape 2.3 - Consolider le filtrage

**Supprimer la duplication de `is_supported_file_static()`:**
- `file_processing.rs` - Supprimer `SUPPORTED_EXTENSIONS` constant
- `branch_processor.rs` - Utiliser `PARSER_DISPATCHER.is_supported()`
- `service.rs` - Utiliser `PARSER_DISPATCHER.is_supported()`

---

## Phase 3: Cargo.toml

```toml
[dependencies]
mimetype-detector = "0.3"  # MIME detection avec MimeKind
once_cell = "1.19"         # Lazy static

# Future optional parsers
[dependencies.pdf_extract]
version = "0.9"
optional = true

[dependencies.docx-rs]
version = "0.4"
optional = true

[features]
default = []
pdf-parser = ["pdf_extract"]
docx-parser = ["docx-rs"]
```

---

## Fichiers à Modifier

| Fichier | Action |
|---------|--------|
| `klask-rs/src/services/mod.rs` | Ajouter `pub mod parser;` |
| `klask-rs/src/services/parser/mod.rs` | **NOUVEAU** - Trait + exports |
| `klask-rs/src/services/parser/dispatcher.rs` | **NOUVEAU** - ParserDispatcher |
| `klask-rs/src/services/parser/text_parser.rs` | **NOUVEAU** - TextParser |
| `klask-rs/src/services/parser/error.rs` | **NOUVEAU** - ParseError |
| `klask-rs/src/services/crawler/git_tree_walker.rs` | Ajouter `read_blob_bytes()` + `read_and_parse_blob()` |
| `klask-rs/src/services/crawler/file_processing.rs` | Changer signature, supprimer `SUPPORTED_EXTENSIONS` |
| `klask-rs/src/services/crawler/branch_processor.rs` | Utiliser dispatcher pour filtrage |
| `klask-rs/src/services/crawler/service.rs` | Utiliser dispatcher pour filtrage |
| `klask-rs/Cargo.toml` | Ajouter dépendances |

---

## Tests

1. **Unit tests** pour chaque parser
2. **Integration tests** pour le registry
3. **Tests existants** doivent continuer à passer (comportement identique pour fichiers texte)

---

## Ordre d'Implémentation

1. Ajouter `mimetype-detector` v0.3 à Cargo.toml
2. Créer le module `parser/` avec trait et dispatcher
3. Implémenter `TextParser` avec `MimeKind::Text` + extensions actuelles
4. Modifier `git_tree_walker.rs` pour retourner bytes
5. Modifier `FileProcessor` pour utiliser `ParsedContent`
6. Consolider le filtrage dans les 3 fichiers via `PARSER_DISPATCHER`
7. Ajouter tests
8. Vérifier que tous les tests existants passent

---

## Préparation Future (hors scope Phase 1)

Structure prête pour ajouter facilement:
```rust
#[cfg(feature = "pdf-parser")]
parsers.push(Arc::new(PdfParser::new()));

#[cfg(feature = "docx-parser")]
parsers.push(Arc::new(DocxParser::new()));
```
