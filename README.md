# VerdictEdge (Rust + Dioxus + Microsoft Phi-3.5 Mini)

> **Air-Gapped, On-Device Legal Risk & Statutory Intelligence Platform written in Rust with Dioxus and Microsoft Phi-3.5 Mini**

VerdictEdge is a privacy-first, 100% offline legal risk analyzer built with **Rust**, **Dioxus Desktop**, and **Microsoft Phi-3.5 Mini**. It analyzes contracts, master service agreements, and employment agreements without sending a single byte to external servers.

---

## ⚡ Core Features

* **100% On-Device & Air-Gapped Privacy:** Zero network calls or external cloud dependencies.
* **Microsoft Phi-3.5 Mini (Instruct) Integration:** Runs local instruct inference with special token formatting (`<|system|>`, `<|user|>`, `<|assistant|>`), providing executive risk synthesis, reciprocal fairness analysis, and strategic leverage.
* **Indian Statutory Voidability Engine:** Automatically flags provisions void under the **Indian Contract Act, 1872**:
  * **Section 27:** Restraint of Trade / post-employment non-compete clauses (based on *Niranjan Shankar Golikari* and *Percept D'Mark* precedents).
  * **Section 28:** Restraint of legal proceedings, premature rights extinguishment, or unreasonable exclusive jurisdiction clauses.
* **Automatic Balanced Counter-Offer Drafter:** Generates proposed counter-clause drafts to restore bilateral balance for:
  * Uncapped Indemnification & Unlimited Liability $\rightarrow$ **Mutual Liability Cap**
  * Immediate Unilateral Termination $\rightarrow$ **30-Day Notice + 15-Day Cure Period**
  * Broad Work-for-Hire / Perpetual IP Loss $\rightarrow$ **Background IP Retention & Payment Milestone Transfer**
  * Unilateral "from time to time" changes $\rightarrow$ **Mutual Written Consent**
  * Auto-Renewal Traps $\rightarrow$ **Explicit 30-Day Opt-in Confirmation**
* **Multilingual Audio Narrator & Speech Normalization:**
  * Generates audio summaries in **English**, **Hindi (हिंदी)**, and **Kannada (ಕನ್ನಡ)**.
  * Formats currency (`₹ 1,50,000` $\rightarrow$ `1,50,000 Rupees` in EN, `रुपये` in HI, `ರೂಪಾಯಿ` in KN), percentages, and pause punctuation.
  * Regional voice routing for Indian English (`Aman`), Hindi (`Lekha`), and Kannada (`Soumya`).
* **Document Importer (PDF & Multi-Doc OCR):**
  * Native offline multi-page PDF parsing via `lopdf`.
  * Multi-document batch processing (up to 10 agreements at once).
  * Apple Vision OCR integration for scanned contracts and images.
* **Interactive Pre-Signing Checklist:**
  * Interactive resolution items with state toggles before executing agreements.
* **Dynamic Font Scaling (`A-` / `A+`):**
  * Dynamic visual font sizing between 12px and 22px.
* **Local Persistent Scan History:**
  * Encrypted / local JSON archival (`~/.verdictedge/history.json`) with reload and deletion controls.
* **Cross-Platform Dioxus Desktop UI:**
  * High-performance reactive UI with a dark slate and emerald legal theme.

---

## 🏗️ Architecture

```
                                  +------------------------------------+
                                  |         VerdictEdge Dioxus         |
                                  |       (Cross-Platform Desktop)     |
                                  +-----------------+------------------+
                                                    |
                                                    v
+---------------------------------------------------------------------------------------------------+
|                                            Dioxus UI                                              |
|                       (InputScreen, ResultsScreen, HistoryDrawer, Scaler)                         |
+---------------------------------------------------------------------------------------------------+
          |                                         |                                     |
          v                                         v                                     v
+-------------------+                   +-----------------------+               +-------------------+
| DocumentImporter  |                   |    ContractEngine     |               |    VoiceEngine    |
| (lopdf / Apple    |                   |   (Rule Evaluator +   |               |  (TTS Normalizer, |
|  Vision OCR)      |                   |    Phi-3.5 Mini LLM)  |               |  Multilingual)    |
+---------+---------+                   +-----------+-----------+               +---------+---------+
          |                                         |                                     |
          v                                         v                                     v
+-------------------+                   +-----------------------+               +-------------------+
| Multi-Doc Text    |                   | Statutory Voidability |               | Spoken Audio      |
| & PDF Streams     |                   | & Counter-Clauses     |               | Briefings         |
+---------+---------+                   +-----------+-----------+               +---------+---------+
          |                                         |                                     |
          +--------------------+--------------------+-------------------------------------+
                               |
                               v
                +-------------------------------+
                |       AnalysisResult          |
                |  (Risk, Voidability, Report)  |
                +---------------+---------------+
                                |
                                v
                +-------------------------------+
                |         HistoryStore          |
                |   (~/.verdictedge/history.json)
                +-------------------------------+
```

---

## 🚀 Running the App

### Native Desktop GUI (Dioxus)
```bash
cargo run
```

### Headless / CLI Pipeline Verification Mode
```bash
cargo run -- --cli
```

### Running Test Suite
```bash
cargo test
```
