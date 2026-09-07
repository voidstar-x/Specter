# Specter v0.6.1 — Mistral 429 retry-with-backoff

Single-fix release on top of v0.6.0. The dedicated Mistral provider
that landed in v0.6.0 surfaces an immediate error when Mistral
returns HTTP 429 — and on the free **Experiment** tier (default
1 req/s) it's easy to trip when the chat composer fires several
Mistral calls in quick succession (main chat + title generation +
HyDE + tabular extraction).

## What's new

### Automatic retry on 429

`POST /v1/chat/completions` is now wrapped in a 3-attempt
retry-with-backoff:

* **Honour Mistral's `Retry-After` header** when present, parsed as
  integer seconds. Capped at 30s so the chat composer doesn't
  freeze for half an hour on a monthly-quota reset.
* **Exponential fallback** when the header is absent: 1s → 2s → 4s.
  Total worst-case wait before surfacing the error: ~7s.
* **Only 429 retries.** Other failure codes (401 invalid key, 403
  quota / guardrail, 422 validation, 5xx server) surface
  immediately — they're authoritative refusals or distinct enough
  to be worth user attention without a hidden retry loop.

The user-perceptible behaviour: a transient 429 (RPS spike, hot
workspace) now resolves itself in 1-7 seconds with a warm progress
indicator instead of failing the chat turn outright. Persistent
429 (genuinely exhausted quota) still surfaces the same Italian
error message after the retry budget is exhausted, so the user
isn't lied to about the state of their account.

## Tests

Five new unit tests pin the backoff calculation:
* `next_backoff_honours_retry_after_seconds`
* `next_backoff_caps_retry_after_at_thirty_seconds`
* `next_backoff_exponential_when_header_missing`
* `next_backoff_ignores_bogus_retry_after_format` (HTTP-date
  format falls through to exponential)
* `next_backoff_handles_whitespace_in_header`

22/22 `llm::mistral` tests green. 110/110 across the `llm::`
tree. No schema migration. No API contract change. No UI change.

## Downloads

Pre-built MSIs for Windows:

- `Specter_0.6.1_x64.msi` — Windows x86_64
- `Specter_0.6.1_arm64.msi` — Windows ARM64, Snapdragon X Elite

Drop-in replacement for v0.6.0.

## License

Specter is distributed under **AGPL-3.0-only**. The Specter
wordmark and logo are trademarks; see `NOTICE.md`. The full
licence text is available in-app under **Settings → Licenza**.
