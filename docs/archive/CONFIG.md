# Configuration (`ownpurse.json`)

## Where the config is found

1. `$OWNPURSE_CONFIG`, if set (it must point to an existing file).
2. `ownpurse.json` in the current folder or any parent folder (a project config).
3. `$XDG_CONFIG_HOME/ownpurse/config.json` (default `~/.config/ownpurse/config.json`).

If none of these exist, a single `default` profile is built from `$OWNPURSE_CLIENT_ID`. `ownpurse config` shows every
effective setting and where it came from. `ownpurse config path` shows the files.

Settings are layered: built-in defaults, then the top level of the file, then the active profile. Any section below can
also be set inside a profile to override it for that profile only.

## Smallest valid config

```json
{
  "default_profile": "demo",
  "profiles": {
    "demo": {
      "client_id": "YOUR_XERO_APP_CLIENT_ID",
      "store": "record.sqlite",
      "orgs": { "roast": "c0ffee00-0000-4000-8000-0000000000a1" }
    }
  }
}
```

## Top level

| Key | Type | Default | Meaning |
|---|---|---|---|
| `default_profile` | string | `"default"` | Profile used when `--profile` and `$OWNPURSE_PROFILE` are not set |
| `profiles` | object | — | Profile name → profile (see below). Required |
| `store` | string | — | Record file for profiles that don't set their own |
| `groups` | object | `{}` | Named scopes: `{"ops": {"orgs": ["roast", "design", "lab"]}}`. Members are aliases, tenant ids or names. Use them as `group:ops` anywhere an org is accepted |
| `parties` | object | `{}` | Intercompany parties: name → regexes matched against `contact` and `narration`, optional `cross_match` (see below) |
| `intercompany.pairs` | array | `[]` | Intercompany pairs for `ownpurse ic` and the Intercompany tab (see below) |
| `account_map.categories` | object | `{}` | Categories for Compare's `align: map` (presentation only; see below) |
| `limits.*`, `sync.*`, `tui.*`, `xero.*` | object | see below | Tunables |
| `links`, `extra_links` | array | built-in list / `[]` | The TUI's Links: `{name, url, description, per?, source?, verified?}` |
| `instructions` | string | built-in text | Text of the TUI help overlay |

## Profile (`profiles.<name>`)

A profile is one Xero app, one token file and one local record.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `client_id` | string | — | The Xero app's client id (a PKCE "mobile or desktop" app; there is no secret). Required |
| `orgs` | object | `{}` | Alias → tenant id, e.g. `{"roast": "<tenant id>"}`. Aliases are what you type |
| `store` | string | `~/.local/share/ownpurse/<profile>.sqlite` | The local record. A relative path resolves against the config file's folder. `$OWNPURSE_STORE` overrides it |
| `token_file` | string | `~/.config/ownpurse/tokens/<profile>.json` | OAuth tokens, written with mode 600. Keep it outside the project. A relative path is relative to the current folder, unlike `store` |
| `require_tenant_name` | string | — | Refuse to work unless the connected org has exactly this name (useful for test profiles) |
| `forbid_tenants` | array | `[]` | Tenant ids this profile must never call |
| `allow_writes` | bool | `false` | Reserved for the approved write protocol. The Rust CLI and the TUI ignore it: they have no write path |
| `extra_links` | array | `[]` | Profile-specific links (added to the top-level ones) |

## `limits`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `limits.calls_per_day` | int | `1000` | Xero's daily call limit per org: quota display and budget checks |
| `limits.calls_per_minute` | int | `60` | Xero's per-minute limit |
| `limits.minute_soft_limit` | int | `55` | Calls in the last minute after which requests pause briefly |
| `limits.warn_day_remaining` | int | `100` | Calls left below which the header warns |
| `limits.refresh_token_days` | int | `60` | Xero's refresh-token lifetime when unused |
| `limits.refresh_token_warn_days` | int | `45` | Idle days after which a re-login reminder appears |

## `sync`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `sync.page_size` | int | `1000` | Records per page on paged endpoints |
| `sync.ims_overlap_seconds` | int | `5` | Overlap on incremental (If-Modified-Since) syncs |
| `sync.full_sweep_days` | int | `7` | Days between full sweeps (catches deletions that incremental syncs miss) |
| `sync.ttl_seconds.default` | int | `86400` | Record age after which an entity counts as stale (the TUI header shows `stale`) |
| `sync.ttl_seconds.<Entity>` | int | `900` for transaction entities | Per-entity override (BankTransactions, ManualJournals, Invoices, …) |

## `tui`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `tui.as_of` | string | `""` | As-of date the TUI opens at: `YYYY-MM-DD`, or `today` (the clock's date, for live books such as the Demo Company). Empty means the scoped org's last fiscal year-end |
| `tui.refresh_seconds` | int | `5` | How often the TUI re-reads the record |
| `tui.log_rows` | int | `500` | Rows shown in Activity |
| `tui.editor` | string | `""` | Editor for opening the config (else `$VISUAL` / `$EDITOR`) |
| `tui.open_command` | string | `""` | Command that opens a link ("Open in Xero"), split on whitespace, the URL appended. Empty: `open` on macOS, else `xdg-open` |
| `tui.narrow_columns` | int | `100` | Below this width: compact header and narrow column sets |
| `tui.stack_columns` | int | `80` | Below this width, document panes stack |
| `tui.detail_lines` | int | `8` | Height of the detail pane |
| `tui.pinned_orgs` | array | `[]` | Aliases or `group:<name>` listed first in the scope picker |
| `tui.recent_scopes` | int | `3` | Recent scopes listed in the picker |
| `tui.compare_default_align` | string | `"code"` | Compare alignment when it opens: `code` or `map` |
| `tui.keys_file` | string | (unset) | Key bindings file other than `~/.config/ownpurse/keys.toml`; relative to this config's folder (docs/TUI.md, *Your own key bindings*) |
| `tui.checklist` | string | (unset) | The close worklist the palette's "Explain: the close worklist" opens (a JSON file: `deadlines`, `questions`, `agent_items`, `postings`); relative to this config's folder (docs/TUI.md, *Explainer panels*) |

## `xero`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `xero.redirect_uri` | string | `http://localhost:8765/callback` | OAuth redirect. Register exactly this on the Xero app |
| `xero.login_timeout_seconds` | int | `900` | How long login waits for the browser |
| `xero.scopes.read` | array | see [scopes](#scopes) | Scopes login requests |
| `xero.scopes.write` | array | see [scopes](#scopes) | The write scopes `ownpurse login --write` adds, only for a profile with `allow_writes` (the owner's browser consent). Plain `login` and the TUI never request them |
| `xero.auth_url`, `token_url`, `revoke_url`, `connections_url`, `api_base` | string | Xero's endpoints | Endpoint overrides (tests) |

## Scopes

Login requests the read scopes below, plus any scope already granted on the token. Consent is additive, so logging in
again never silently drops a permission you granted earlier. Plain `ownpurse login` never adds a write scope; `login --write` (only for an `allow_writes` profile) adds `xero.scopes.write`. `ownpurse doctor` lists the granted scope names (never a token value) and, for an `allow_writes` profile, whether the write scopes are granted.

Read: `openid`, `profile`, `email`, `offline_access`, `accounting.settings.read`, `accounting.contacts.read`,
`accounting.attachments.read`, `accounting.banktransactions.read`, `accounting.manualjournals.read`,
`accounting.invoices.read`, `accounting.payments.read`, `accounting.reports.trialbalance.read`,
`accounting.reports.balancesheet.read`, `accounting.reports.profitandloss.read`, `accounting.reports.banksummary.read`.

## Intercompany (`parties`, `intercompany.pairs`)

A **pair** names the accounts on each side that should mirror each other.

```json
{
  "parties": {
    "hold": {"contact": ["\\bbrackenridge\\b"], "narration": ["\\bbrackenridge\\b"]}
  },
  "intercompany": {
    "pairs": [
      {"id": "hold-roast", "label": "Brackenridge ↔ Tallowmere",
       "a": {"org": "hold", "accounts": ["1460"]},
       "b": {"org": "roast", "accounts": ["2410"], "party": "hold"},
       "movement": "b_unexplained", "match_days": 5}
    ]
  }
}
```

| Key | Meaning |
|---|---|
| `id`, `label` | Pair id (for `ownpurse ic <id>`) and display label |
| `a`, `b` | Each side: `org` (alias) and `accounts` (codes, or `{"name": …}` for code-less bank accounts) |
| `b.party` | The party whose share of `b`'s accounts is compared, attributed by contact, narration or cross-org match |
| `b.other_parties` | Other parties posting to the same accounts, excluded from this party's residual |
| `b.component` | `residual` (account total less other parties' attributed postings) or `attributed` (only this party's postings) |
| `movement` | How the year's change in the difference is explained: `b_unexplained` or `match_both` (opposite amounts within `match_days`) |
| `match_days` | Matching window in days |
| `known_items` | Expected historical items. The engine reports whether the record contains each one |
| `parties.<name>.contact` / `.narration` | Case-insensitive regexes (Rust `regex` syntax; lookarounds and backreferences are a config error) |
| `parties.<name>.cross_match` | `{org, accounts, days}`: attribute by a matching posting in another org |

The output shows both sides, the difference, the bridge (prior-year opening difference plus this year's movement), the
explaining items, and an explicit unexplained residual. Nothing is ever filled in to force a match.

## Account map (`account_map.categories`)

Presentation grouping for Compare's `align: map`. It never changes a balance.

```json
{"account_map": {"categories": {
  "cash": {"default": "type:BANK", "label": "Cash and cards"},
  "loan_balances": {"hold": ["1460"], "roast": ["2410"], "label": "Loan balances"},
  "revenue": {"default": "class:REVENUE", "label": "Revenue"}
}}}
```

A category lists codes per org alias, or sets a `default` rule (`type:<XERO TYPE>` or `class:<CLASS>`), plus an
optional `label`. Rows merge only within one category.

## `classify`

Where the classification files live (top level or per profile; relative paths are relative to this file's folder):

| Key | Default | What |
|---|---|---|
| `classify.config` | `classify.toml` | the catalog, rules and queue settings (TOML, hand-editable; see ARCHITECTURE.md, Classification) |
| `classify.store` | `db/classify.sqlite` | the local label store (append-only; never sent to Xero) |

Rule conditions (`[[rules]] when = { … }`, all must hold): `description_has_alias`, `payee`, `payee_has`,
`description_has`, `account_in`, `org_in`, `amount_between`, `date_between`, `sign` (`in`/`out`),
`payee_is_org` (an org alias, or `any`: the payee is a configured org by its organisation or legal name, case,
punctuation and suffixes such as Ltd/LLC/LLP/Inc ignored; never the line's own org) and `bank_transfer`
(`true`: a `SPEND-TRANSFER`/`RECEIVE-TRANSFER` transaction or a line posted to a bank account).

Check the config with `ownpurse classify rules check`.

## Environment variables

| Variable | Effect |
|---|---|
| `OWNPURSE_CONFIG` | Config file to use |
| `OWNPURSE_PROFILE` | Profile (below `--profile`) |
| `OWNPURSE_STORE` | Record file (overrides `store`) |
| `OWNPURSE_CLIENT_ID` | Client id when there is no config file |
| `XDG_CONFIG_HOME`, `XDG_DATA_HOME` | Base folders for the user config, default token file and default record |
| `NO_COLOR` | The TUI uses no colour |
