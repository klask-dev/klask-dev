//! Semantic search evaluation probe (plan phase 6).
//!
//! Answers one question before any corpus-level eval is worth running: **does
//! the embedding model actually understand code, and does it survive a query
//! written in a language other than English?** Klask is a general-purpose,
//! open-source code search engine, so the indexed identifiers and the user's
//! query may be in any language, in any combination.
//!
//! The probe is a miniature retrieval benchmark: N code snippets, N natural
//! language queries describing them, every query ranked against every snippet.
//! Each concept exists in two variants that differ **only** by the language of
//! its identifiers and comments, and each query exists in two languages, which
//! gives a 2x2:
//!
//! | code \ query | English | French |
//! |---|---|---|
//! | English identifiers | baseline | cross-lingual query |
//! | French identifiers  | mixed    | fully non-English project |
//!
//! Queries deliberately avoid reusing the identifiers they should match, so the
//! probe measures *semantic* retrieval, not lexical overlap. Lexical overlap is
//! what BM25 already handles for free, and the whole premise of adding a vector
//! index is to retrieve what BM25 cannot.
//!
//! Run (downloads each model on first use):
//! ```sh
//! cargo run --release --features semantic-search --bin semantic-eval
//! cargo run --release --features semantic-search --bin semantic-eval -- \
//!     jinaai/jina-embeddings-v2-base-code intfloat/multilingual-e5-small
//! ```
//!
//! A model at chance scores P@1 = 1/N. Read the *margin* column too: it is how
//! far the correct snippet sits ahead of the best distractor, i.e. how much
//! room a similarity threshold would have in production.

use anyhow::Result;
use klask_rs::config::SemanticSearchConfig;
use klask_rs::services::semantic::embedder::{EmbeddingProvider, FastEmbedProvider};

/// One concept, expressed as code twice (identifiers in English, then in
/// French) and as a natural-language query twice (English, then French).
struct Concept {
    label: &'static str,
    code_en: &'static str,
    code_fr: &'static str,
    query_en: &'static str,
    query_fr: &'static str,
}

/// Eight everyday backend concepts. Short enough (5-12 lines) that no snippet
/// approaches the 512-token window, so truncation cannot distort the result.
const CONCEPTS: &[Concept] = &[
    Concept {
        label: "token expiry",
        code_en: r#"export function isSessionTokenValid(token: string): boolean {
  const payload = decodeBase64(token.split(".")[1]);
  const { exp } = JSON.parse(payload);
  return exp * 1000 > Date.now();
}"#,
        code_fr: r#"export function jetonSessionEstValide(jeton: string): boolean {
  const charge = decoderBase64(jeton.split(".")[1]);
  const { exp } = JSON.parse(charge);
  return exp * 1000 > Date.now();
}"#,
        query_en: "check whether a login credential has expired before granting access",
        query_fr: "vérifier si une authentification a expiré avant d'autoriser l'accès",
    },
    Concept {
        label: "backoff retry",
        code_en: r#"async function callWithRetry<T>(fn: () => Promise<T>, attempts = 5): Promise<T> {
  for (let i = 0; i < attempts; i++) {
    try {
      return await fn();
    } catch (e) {
      await sleep(2 ** i * 100);
    }
  }
  throw new Error("giving up");
}"#,
        code_fr: r#"async function appelerAvecNouvelEssai<T>(fn: () => Promise<T>, tentatives = 5): Promise<T> {
  for (let i = 0; i < tentatives; i++) {
    try {
      return await fn();
    } catch (e) {
      await dormir(2 ** i * 100);
    }
  }
  throw new Error("abandon");
}"#,
        query_en: "run an operation again after a failure, pausing longer each time",
        query_fr: "relancer une opération après un échec en patientant de plus en plus longtemps",
    },
    Concept {
        label: "pagination",
        code_en: r#"function slicePage<T>(items: T[], page: number, size: number): T[] {
  const start = (page - 1) * size;
  return items.slice(start, start + size);
}"#,
        code_fr: r#"function decouperPage<T>(elements: T[], numero: number, taille: number): T[] {
  const debut = (numero - 1) * taille;
  return elements.slice(debut, debut + taille);
}"#,
        query_en: "return only a subset of results for the current screen of a long list",
        query_fr: "ne renvoyer qu'une partie des résultats pour l'écran courant d'une longue liste",
    },
    Concept {
        label: "cache eviction",
        code_en: r#"function evictOldest(cache: Map<string, Entry>, limit: number): void {
  while (cache.size > limit) {
    const first = cache.keys().next().value;
    cache.delete(first);
  }
}"#,
        code_fr: r#"function evincerLesPlusAnciens(memoire: Map<string, Entree>, limite: number): void {
  while (memoire.size > limite) {
    const premier = memoire.keys().next().value;
    memoire.delete(premier);
  }
}"#,
        query_en: "drop the least recently used entries when memory grows too large",
        query_fr: "supprimer les données gardées de côté quand la mémoire devient trop grosse",
    },
    Concept {
        label: "csv parsing",
        code_en: r#"function readDelimitedFile(raw: string): string[][] {
  return raw
    .split("\n")
    .filter(Boolean)
    .map((line) => line.split(";"));
}"#,
        code_fr: r#"function lireFichierDelimite(brut: string): string[][] {
  return brut
    .split("\n")
    .filter(Boolean)
    .map((ligne) => ligne.split(";"));
}"#,
        query_en: "turn a spreadsheet export into rows and columns",
        query_fr: "transformer un export tableur en lignes et colonnes",
    },
    Concept {
        label: "password hashing",
        code_en: r#"async function protectSecret(plain: string): Promise<string> {
  const salt = randomBytes(16);
  return argon2.hash(plain, { salt, memoryCost: 65536 });
}"#,
        code_fr: r#"async function protegerSecret(clair: string): Promise<string> {
  const sel = octetsAleatoires(16);
  return argon2.hash(clair, { salt: sel, memoryCost: 65536 });
}"#,
        query_en: "store a user's login phrase so that nobody can read it back",
        query_fr: "stocker le mot de passe d'un utilisateur sans pouvoir le relire ensuite",
    },
    Concept {
        label: "outbound mail",
        code_en: r#"async function notifyByMail(to: string, subject: string, body: string) {
  await transport.deliver({ from: NO_REPLY, to, subject, text: body });
}"#,
        code_fr: r#"async function notifierParCourriel(destinataire: string, objet: string, corps: string) {
  await transport.livrer({ from: NE_PAS_REPONDRE, destinataire, objet, text: corps });
}"#,
        query_en: "let someone know something happened, using their inbox address",
        query_fr: "prévenir quelqu'un qu'il s'est passé quelque chose via son adresse de messagerie",
    },
    Concept {
        label: "upload size limit",
        code_en: r#"function rejectIfTooLarge(file: Upload, maxBytes: number): void {
  if (file.size > maxBytes) {
    throw new PayloadTooLarge(file.name);
  }
}"#,
        code_fr: r#"function refuserSiTropVolumineux(fichier: Depot, octetsMax: number): void {
  if (fichier.size > octetsMax) {
    throw new ChargeTropGrande(fichier.name);
  }
}"#,
        query_en: "refuse a document that exceeds the allowed weight before storing it",
        query_fr: "refuser un document qui dépasse le poids autorisé avant de l'enregistrer",
    },
];

/// Models compared when none are given on the command line: the current default
/// (code-specialized, English), the lighter English alternative, and the
/// multilingual challenger.
const DEFAULT_MODELS: &[&str] =
    &["jinaai/jina-embeddings-v2-base-code", "Xenova/bge-small-en-v1.5", "intfloat/multilingual-e5-small"];

/// Instruction prefixes a model expects, as `(query, document)`.
///
/// fastembed applies none of these itself, and Klask's query path currently
/// embeds raw text (`semantic/query.rs`). Skipping them is not neutral: E5
/// models are trained with `query:` / `passage:` and lose a lot without them,
/// so a probe that omitted them would under-report the multilingual option.
/// **If a prefixed model wins, the product has to learn the same prefixes.**
fn prefixes(model_code: &str) -> (&'static str, &'static str) {
    let code = model_code.to_ascii_lowercase();
    if code.contains("e5") {
        // intfloat E5 family, including the multilingual ones.
        ("query: ", "passage: ")
    } else if code.contains("bge") && !code.contains("m3") {
        // BGE v1.5 documents a query-side instruction for retrieval; documents
        // are embedded bare. bge-m3 needs no instruction.
        ("Represent this sentence for searching relevant passages: ", "")
    } else {
        // jina code/text models and the rest: symmetric, no instruction.
        ("", "")
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

struct Score {
    /// Share of queries whose own snippet ranks first.
    p_at_1: f32,
    /// Mean reciprocal rank of the correct snippet.
    mrr: f32,
    /// Mean cosine gap between the correct snippet and the best wrong one.
    /// Negative means the model prefers a distractor on average.
    margin: f32,
    /// Concepts the model got wrong, for a qualitative read.
    misses: Vec<&'static str>,
}

/// Rank every snippet against every query; `queries[i]` belongs to `docs[i]`.
fn evaluate(queries: &[Vec<f32>], docs: &[Vec<f32>]) -> Score {
    let n = queries.len();
    let mut hits = 0usize;
    let mut rr_sum = 0.0f32;
    let mut margin_sum = 0.0f32;
    let mut misses = Vec::new();

    for (i, q) in queries.iter().enumerate() {
        let sims: Vec<f32> = docs.iter().map(|d| cosine(q, d)).collect();
        let correct = sims[i];
        let best_other = sims.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, s)| *s).fold(f32::MIN, f32::max);
        margin_sum += correct - best_other;

        // Rank of the correct snippet: how many distractors beat it, plus one.
        let rank = 1 + sims.iter().enumerate().filter(|(j, s)| *j != i && **s > correct).count();
        rr_sum += 1.0 / rank as f32;
        if rank == 1 {
            hits += 1;
        } else {
            misses.push(CONCEPTS[i].label);
        }
    }

    Score { p_at_1: hits as f32 / n as f32, mrr: rr_sum / n as f32, margin: margin_sum / n as f32, misses }
}

fn embed_all(provider: &FastEmbedProvider, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    provider.embed(&texts)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let models: Vec<String> = if args.is_empty() {
        DEFAULT_MODELS.iter().map(|s| s.to_string()).collect()
    } else {
        args
    };

    let n = CONCEPTS.len();
    println!(
        "Semantic probe: {n} concepts, {n} queries, chance P@1 = {:.2}\n",
        1.0 / n as f32
    );

    for model in &models {
        let config = SemanticSearchConfig {
            enabled: true,
            model: model.clone(),
            cache_dir: "target/fastembed-cache".to_string(),
            vector_store_dir: "target/eval-vector-index".to_string(),
            chunk_max_lines: 45,
            chunk_overlap_lines: 10,
            batch_size: 32,
            queue_capacity: 32,
        };

        let started = std::time::Instant::now();
        let provider = match FastEmbedProvider::try_new(&config) {
            Ok(p) => p,
            Err(e) => {
                println!("{model}\n  SKIPPED: {e}\n");
                continue;
            }
        };
        let (q_prefix, d_prefix) = prefixes(model);

        let docs_en = embed_all(
            &provider,
            CONCEPTS.iter().map(|c| format!("{d_prefix}{}", c.code_en)).collect(),
        )?;
        let docs_fr = embed_all(
            &provider,
            CONCEPTS.iter().map(|c| format!("{d_prefix}{}", c.code_fr)).collect(),
        )?;
        let queries_en = embed_all(
            &provider,
            CONCEPTS.iter().map(|c| format!("{q_prefix}{}", c.query_en)).collect(),
        )?;
        let queries_fr = embed_all(
            &provider,
            CONCEPTS.iter().map(|c| format!("{q_prefix}{}", c.query_fr)).collect(),
        )?;

        println!(
            "{model}  (dim {}, loaded+embedded in {:.1}s{})",
            provider.dimension(),
            started.elapsed().as_secs_f32(),
            if q_prefix.is_empty() {
                String::new()
            } else {
                format!(", query prefix {q_prefix:?}")
            }
        );
        println!(
            "  {:<22} {:>6} {:>6} {:>8}   missed",
            "code / query", "P@1", "MRR", "margin"
        );

        for (label, queries, docs) in [
            ("EN code / EN query", &queries_en, &docs_en),
            ("EN code / FR query", &queries_fr, &docs_en),
            ("FR code / FR query", &queries_fr, &docs_fr),
            ("FR code / EN query", &queries_en, &docs_fr),
        ] {
            let s = evaluate(queries, docs);
            println!(
                "  {:<22} {:>6.2} {:>6.2} {:>+8.3}   {}",
                label,
                s.p_at_1,
                s.mrr,
                s.margin,
                s.misses.join(", ")
            );
        }
        println!();
    }

    Ok(())
}
