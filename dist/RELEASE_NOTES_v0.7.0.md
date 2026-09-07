# Specter v0.7.0 — Nuovo settore «Fiscale» (tax/commercialista italiano)

Aggiunge il dodicesimo settore professionale, **Fiscale**, dedicato a
commercialisti e consulenti tributari italiani sul versante
adempimenti, imposte e contenzioso tributario. Complementare a
«Finanza»: Finanza copre bilanci/valutazione/crisi d'impresa, Fiscale
copre IVA, imposte dirette e indirette, dichiarazioni, ravvedimento,
accertamento e processo tributario.

## Cosa c'è di nuovo

### Settore Fiscale selezionabile ovunque

`Fiscale` compare nel selettore di settore in sidebar, nei filtri di
Workflow / Revisioni tabellari / Progetti, e come dominio di default
impostabile in Settings. Etichette: IT «Fiscale», EN «Tax»,
FR «Fiscalité», DE «Steuern», ES/PT «Fiscal».

### 8 workflow pronti all'uso

**Assistant (chat strutturata):**

- **Parere tributario strutturato** — fatti → norma → prassi/
  giurisprudenza → analisi → conclusione → disclaimer.
- **Ravvedimento operoso — calcolo guidato** — imposta + sanzione
  ridotta (nuovo regime D.Lgs. 87/2024) + interessi legali pro-rata,
  con codici tributo F24.
- **Analisi avviso di accertamento + strategia difensiva** — termini
  d'impugnazione, rilievi, vizi, confronto adesione / autotutela /
  ricorso (post-abrogazione del reclamo-mediazione).

**Tabular (estrazione multi-documento):**

- **Riconciliazione liquidazioni IVA periodiche** — registri vs LIPE
  vs F24.
- **Verifica requisiti regime forfettario** — L. 190/2014 art. 1
  c. 54-89.
- **Monitoraggio fiscale — Quadro RW (IVIE/IVAFE)** — attività estere.
- **Imposte indirette su atti** — registro/bollo/ipo-catastali.
- **Scadenzario versamenti F24 per cliente**.

### 9 colonne quick-insert Fiscale

Imponibile, Aliquota, Imposta dovuta, Ritenuta, Sanzione, Interessi,
Norma di riferimento, Scadenza, Codice tributo — per costruire
revisioni tabellari fiscali su misura.

### Allineamento alle riforme tributarie 2024

I prompt del settore recepiscono esplicitamente:

- **Reclamo-mediazione abrogato** dal 4.1.2024 (D.Lgs. 220/2023): il
  ricorso va direttamente alla Corte di Giustizia Tributaria di primo
  grado.
- **Nuovo regime sanzionatorio D.Lgs. 87/2024** dal 1.9.2024 (omesso
  versamento 30% → 25%, ravvedimento a 1/7): il modello verifica la
  data della violazione e dichiara il regime applicato.
- **Contraddittorio preventivo generalizzato** (art. 6-bis L. 212/2000,
  D.Lgs. 219/2023).

### Documentazione

Nuovo `docs/piano_settore_fiscale.md` con il piano descrittivo del
settore, incluso l'elenco delle **banche dati pubbliche e gratuite**
per norme e sentenze (Normattiva, def.finanze.it, Sentenze Web
Cassazione, Giustizia Tributaria DGT, EUR-Lex) e la roadmap di
arricchimento.

## Note di migrazione

- **Nessuna migrazione di schema.** La validazione del dominio è al
  confine API; aggiungere un settore è una modifica a `DOMAINS` +
  etichette i18n. Installazioni esistenti aggiornano in modo
  trasparente.
- I preset e i system prompt sono file in `config/` letti all'avvio:
  installando il nuovo MSI compaiono automaticamente.

## Download

MSI Windows precompilati:

- `Specter_0.7.0_x64.msi` — Windows x86_64
- `Specter_0.7.0_arm64.msi` — Windows ARM64, Snapdragon X Elite

Sostituzione drop-in per v0.6.7.

## Licenza

Specter è distribuito sotto **AGPL-3.0-only**. Il marchio e il logo
Specter sono marchi registrati; vedi `NOTICE.md`. Il testo completo
della licenza è disponibile in-app sotto **Impostazioni → Licenza**.
