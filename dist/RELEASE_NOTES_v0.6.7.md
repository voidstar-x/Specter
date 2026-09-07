# Specter v0.6.7 — Sidebar active-domain selector

Adds a compact domain picker next to the Specter brand at the top
of the sidebar — always visible, default-initialised from the
user's `default_domain` setting (chosen at sign-up, editable in
Settings → Generale), and persisted server-side on every change.

## Why

The default domain drives a lot of domain-scoped UI:

* Workflow + template pickers in the chat composer filter by it
* "Nuovo progetto" / "Nuova revisione tabellare" pre-populate
  their domain dropdown to it
* The chat composer inherits it on standalone chats

Before v0.6.7 changing it required Settings → Generale → Dominio
predefinito → save (~6 clicks). Most real-world flows hit this
knob multiple times a day; making it one click on the sidebar
brand is a big quality-of-life win.

## How it looks

```
┌──────────────────────────────────────┐
│ ⬛ Specter v0.6.7  [Assicurazioni ▾] │
├──────────────────────────────────────┤
│ 💬 Assistente                    +   │
│ 📁 Progetti                          │
│ 📋 Revisioni tabellari               │
│ ...                                  │
└──────────────────────────────────────┘
```

The combo:

* Defaults to whatever the user picked at sign-up (or the last
  saved value).
* Shows only the **enabled domains** preference (so a user who
  hid "Compliance" in Settings → Generale won't see it in the
  sidebar either).
* Saves the new choice to `user_settings.default_domain` on every
  change — no separate "save" button.
* Survives restart / re-installation that preserves the data
  folder (the picker reads from the same store hydrated at
  unlock time).

## Tests

Frontend-only change; svelte-check 0 errors. Backend untouched.

## Downloads

Pre-built MSIs for Windows:

- `Specter_0.6.7_x64.msi` — Windows x86_64
- `Specter_0.6.7_arm64.msi` — Windows ARM64, Snapdragon X Elite

Drop-in replacement for v0.6.6.

## License

Specter is distributed under **AGPL-3.0-only**. The Specter
wordmark and logo are trademarks; see `NOTICE.md`. The full
licence text is available in-app under **Settings → Licenza**.
