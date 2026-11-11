# Search syntax

This page describes what the Klask search box accepts today. It is derived from
the search service (`klask-rs/src/services/search.rs`), the tokenizer
(`klask-rs/src/services/tokenizer.rs`) and the search API
(`klask-rs/src/api/search.rs`).

## The basics

- A query is split into **terms** on whitespace and on any character that is not
  a letter, a digit, `_` or `-`. Punctuation such as `.`, `(`, `:` or `/` acts as
  a separator, so `foo.bar(baz)` is searched as the three terms `foo`, `bar`
  and `baz`.
- **All terms must match** (implicit AND). There is no `OR`, `NOT`,
  `field:value` or `+`/`-` operator: those characters are not interpreted.
- Each term is looked up in the **file content, the file name and the file
  path**. A file matches when every term is found in at least one of those three
  fields (different terms may be found in different fields).
- Matching is on whole **tokens**, not on substrings: searching `conn` does not
  find `connection`. For substring-like matching, use a [regular
  expression](#regular-expressions).
- Search is **case-insensitive** by default.
- Results can be narrowed with filters: repository, project, branch or tag,
  file extension and file size. Filters are applied in addition to the query.

## How code is tokenized

Source files are indexed with a tokenizer made for code:

| In the file | Tokens indexed |
|---|---|
| `camelCase` | `camel`, `case`, and the whole word (`camelcase` and `camelCase`) |
| `HTMLParser` | `html`, `parser`, and the whole word |
| `getHTTPResponse` | `get`, `http`, `response`, and the whole word |
| `my_function_name` | `my_function_name` (underscores and hyphens are kept) |
| `snake-case` | `snake-case` |

Consequences for queries:

- Searching `case` finds `camelCase`, because the parts of a camelCase word are
  indexed too.
- Searching `camelCase` finds that identifier, because the whole word is indexed
  as well. Queries themselves are **not** split on camelCase: `camelCase` is a
  single term, lowercased to `camelcase` for a case-insensitive search.
- `snake_case` identifiers are **not** split on underscores: searching
  `function` does not find `my_function_name`. Search the full identifier, or use
  a regular expression.

## Case-sensitive search

Enable the **case sensitive** option to match case exactly: `readerTemplate`
then matches that spelling only, not `readertemplate`. Matching is still on whole
tokens. In the current backend, when this option is on the fuzzy and regex
options have no effect.

## Fuzzy search

Enable the **fuzzy** option to tolerate one character of difference (edit
distance 1) per term, for example a typo. It applies to the content, name and
path lookups of each term. Fuzzy search and regular expressions are mutually
exclusive: when both are requested, the regular expression is used.

## Regular expressions

Enable the **regex** option to search with a regular expression.

- The syntax is that of the Rust [`regex`](https://docs.rs/regex) crate (no
  look-around, no backreferences).
- Optional flags: `i` (case-insensitive), `m` (multiline), `s` (dot matches
  newline), alone or combined (`ims`).
- **The pattern is matched against one indexed token at a time**, not against
  lines or raw text. It cannot span several tokens: it cannot contain a space,
  and a pattern such as `foo.*bar` only matches inside a single token (for
  example `foo_and_bar`). Tokens come from the tokenizer described above, so
  `camelCase` is matched as `camel`, `case` or the whole word.
- The pattern is also tried against the file name and the file path.
- Content tokens are indexed in lowercase: write the pattern in lowercase (or
  use the `i` flag) when searching file content.
- Limits: a pattern is at most 500 characters long, with at most 3 levels of
  nested groups; empty patterns and unbalanced parentheses are rejected.
- Patterns that start with `.*` have to scan the whole index and are slow. A
  search is stopped after 30 seconds.

## Phrases

Searching for an exact sequence of words in quotes is **not supported** yet:
quote characters act as separators and the words are searched independently.

## Semantic and hybrid modes

If the server was built and configured for it, the search can also run in a
*semantic* mode (natural-language questions) or a *hybrid* mode that combines it
with the keyword search. See [Semantic search](../admin/semantic-search.md). The
default is keyword search; use it for regular expressions and exact identifiers.
