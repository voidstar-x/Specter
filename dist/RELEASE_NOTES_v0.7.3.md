# Specter v0.7.3 — Workflow cross-dominio + analisi cespiti e libri contabili

Aggiunge due workflow del commercialista (analisi cespiti e controlli
libri contabili) e il meccanismo di **registrazione cross-dominio**:
un singolo preset può comparire nel picker di più settori, senza
duplicare il JSON.

## Registrazione cross-dominio (`also_applicable_to`)

I preset workflow guadagnano il campo opzionale `also_applicable_to`
(stesso meccanismo già usato dai template DOCX): elenca i domini
**aggiuntivi** oltre a quello primario. Un workflow utile a più
settori si registra **una volta sola** e appare in tutti i picker
applicabili.

* Backend: nuovo campo + metodo `matches_domain` (settore primario
  OPPURE qualsiasi voce di `also_applicable_to`). Il loader valida le
  voci contro l'elenco canonico dei settori (scarta quelle non valide
  o ridondanti). Il filtro `GET /workflow?domain=` usa `matches_domain`.
* I workflow creati dall'utente restano a settore singolo: è un
  meccanismo per i preset di sistema.

## Due nuovi workflow (entrambi `Fiscale` ⇄ `Finanza`)

Basati su fonti autorevoli (art. 16 DPR 600/73, art. 102/86 TUIR,
coefficienti DM 31/12/1988, art. 5 D.Lgs. 446/97), con norme
verificate aggiornate (super/iper-ammortamento storico, sostituito dal
credito d'imposta Transizione 4.0/5.0; plafond manutenzioni 5% ex art.
102 c.6):

- **Analisi cespiti / registro beni ammortizzabili** (tabular) —
  riconciliazione registro↔bilancio, deducibilità ammortamenti,
  eccedenza fiscale (→ variazione IRES), movimenti e plus/minusvalenze.
  Una riga per cespite o categoria.
- **Controlli di quadratura libri contabili** (tabular) — registro
  anomalie: partita doppia, progressività, saldi mastrini (cassa mai
  negativa, banca riconciliata), riconciliazione IVA (registri ↔ LIPE
  ↔ dichiarazione), ritenute ↔ F24, quadratura clienti/fornitori,
  scritture di assestamento. Una riga per anomalia.

Entrambi compaiono **sia** nel picker Fiscale **sia** in quello
Finanza grazie a `also_applicable_to`.

## Note

- Nessuna migrazione di schema. 42/42 test preset-loader verdi
  (inclusi 2 nuovi sul cross-dominio).
- `docs/WORKFLOWS.md` §2 documenta il campo `also_applicable_to`;
  `docs/piano_settore_fiscale.md` §3 annota la coppia cross-dominio.

## Download

- `Specter_0.7.3_x64.msi` — Windows x86_64
- `Specter_0.7.3_arm64.msi` — Windows ARM64, Snapdragon X Elite

Sostituzione drop-in per v0.7.2.

## Licenza

AGPL-3.0-only. Il marchio e il logo Specter sono marchi registrati.
