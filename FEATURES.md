# Rumor feature list

Status: proposed scope, not implementation status. Accepted decisions: Rune UI, fully keyboard-operable Vim-style interaction, an independent background daemon, and daily browser-assisted collection with manual intervention.

## Must: first usable local release

| ID | Feature | Required behavior |
|---|---|---|
| F01 | Source subscriptions | Add, disable, remove and configure sources and sections independently. Removal stops fetching; saved articles and labels remain unless explicitly deleted. |
| F02 | Source adapters | Public host/protocol with separately installed adapters, each in its own private repository. Discovery/content implementations stay private; public builds/tests need no private checkout or credentials. Show adapter/version/access capabilities and a no-adapters-installed state. |
| F03 | Chronological inbox | Publication-time sorting, newest or oldest first; never silently reorder by ML score. Stable paging and visible missing/estimated dates. |
| F04 | Seen tracking | A visible headline can become seen without becoming read or becoming a training label. Explicit mark unseen and undo. |
| F05 | Three-way feedback | **Hide / don't show this headline**, **Headline only**, **Read later**. Editable explicit labels, with unlabelled as a separate state. |
| F06 | Read-later queue | Read later records the preference and queues the article atomically. Removing from queue or finishing reading does not silently relabel it. |
| F07 | In-app reader | Sanitized article body, source/byline/publication attribution, links, readable typography, tables/code where present; original URL always available. |
| F08 | Offline saving | Queue full-text retrieval when saved; clearly distinguish complete offline copy, partial copy, pending, blocked, login required and unavailable. Never call a summary a full article. |
| F09 | Reading state | Unread/in progress/read, scroll resume, archive and reopen; independent of headline exposure and interest labels. |
| F10 | Local preference model | Train on explicit feedback, keep data/model locally, version and roll back models, show insufficient-data state. |
| F11 | Likelihood filter | Show predicted headline interest and read-later interest; filter by minimum headline interest while keeping time order. All-items view and hidden-items review remain available. |
| F12 | Deduplication | Same article discovered in multiple sections appears once; preserve discovery provenance. Group related coverage reversibly; allow split/merge corrections. |
| F13 | Search and filters | Search titles and available bodies; filter source, section, date, seen, saved, read and label. |
| F14 | Refresh and health | Manual refresh plus daemon-owned scheduling while the reader is closed; per-source last success, stale/blocked/needs-attention status, bounded retries and catch-up after resume. |
| F15 | Data ownership | SQLite persistence, transactional writes, backup/restore and export of subscriptions, URLs, states and labels. Cache cleanup cannot erase saved items or feedback. |
| F16 | Entirely keyboard-operable, Vim-style UI | Every application action, setting, dialog and reader operation works without a mouse. Vim-style navigation, searchable command palette, shortcut help, visible focus, selectable text, adjustable type and themes. Text fields preserve normal typing; Tab/Shift-Tab and Escape remain available. |
| F17 | Manual URL intake | Paste an article URL; retain a useful queued link even if extraction fails. Access restrictions still apply. |
| F24 | Independent background daemon | A separate user process collects sources, retrieves queued articles and runs local training with the Rune UI closed. Provide status, start/stop/pause controls, singleton ownership, durable jobs and recovery. |
| F25 | Daily browser-assisted collection | Open a dedicated visible browser session once daily for configured sources. Pause on login, consent or checks; let the owner resolve them manually, then resume without duplicate imports. Persist partial progress and expose pending intervention even when the reader is closed. |

## Should: after the workflow works

| ID | Feature | Purpose |
|---|---|---|
| F18 | Semantic grouping | Add a local embedding model if lexical grouping misses enough cross-source stories to justify it. |
| F19 | Source-specific refinements | Better article selectors, consented browser capture where permitted, media handling and content quality checks. |
| F20 | Model diagnostics | Per-class metrics, recent errors, class counts, reasons, hidden-item audits and measured calibration. |
| F21 | OPML interchange | Import/export feed subscriptions; use separate JSON export for labels and reading state. |
| F22 | Advisory changes | Highlight revised guidance, due-date changes and structured-record updates without duplicating unchanged records. |
| F23 | Browser save action | Optional extension to save the currently viewed URL or permitted article snapshot. No universal background scraping. |

## Later: future personal deployment

- Headless Rust collector/API running on a personal server.
- Website and mobile app with offline reading and synchronization.
- Single-owner device authentication, encrypted transport and revocable device tokens.
- Conflict handling for labels, read state, queue changes and deletions.

## Deliberately outside the first release

No multi-user account system, social feed, public article redistribution, generative summaries/chat, custom RAG stack, training a foundation model, mandatory GPU, paid scraping infrastructure, or plugin marketplace. Extensible source code is required; downloadable untrusted code is not required initially.
