# FairEmail — Information Sources for Reverse Engineering

This document lists every place we can collect information from while reverse engineering the user-facing feature set of the FairEmail Android app. Sources are grouped by locality (in-repo vs. external) and ordered roughly by signal strength for *user-visible feature* discovery.

---

## 1. In-repository sources (highest authority — ground truth)

### 1.1 Source code

- [app/src/main/java/](../app/src/main/java/) — primary application code (Java). Sub-packages under `eu/faircode/email/` contain the bulk of feature implementations: activities, fragments, adapters, workers, helpers.
- [app/src/main/aidl/](../app/src/main/aidl/) — AIDL interfaces (e.g. OpenPGP integration boundary).
- [app/src/main/jni/](../app/src/main/jni/) — native code (used for things like compact-table / bundled libs).
- [app/src/main/AndroidManifest.xml](../app/src/main/AndroidManifest.xml) — declared activities, services, receivers, providers, permissions, intents handled, deep links, shortcuts, exported components. **Excellent index of "surface area" features.**
- [app/src/](../app/src/) — flavor/build-type variants worth diffing for feature gating:
  - `play/`, `fdroid/`, `github/`, `amazon/` (distribution channels — gated billing/update logic, store-specific features)
  - `debug/`, `release/`
  - `large/` (large-screen tweaks)
  - `extra/`, `dummy/` (paid/extra features vs. stubs)
- [app/schemas/](../app/schemas/) — Room database schema JSON per version. Excellent for understanding entities (Account, Identity, Folder, Message, Rule, Contact, Attachment, etc.) and their evolving columns.
- [app/build.gradle](../app/build.gradle) and [build.gradle](../build.gradle) — flavors, build configs, feature flags, compile-time toggles.
- [app/proguard-rules.pro](../app/proguard-rules.pro) — sometimes reveals retained class surfaces.
- Sibling modules: [openpgp-api/](../openpgp-api/), [decrypt/](../decrypt/), [eml/](../eml/), [colorpicker/](../colorpicker/), [oauth/](../oauth/) — feature subsystems extracted into modules.

### 1.2 Resources (UI surface — what the user actually sees)

- [app/src/main/res/layout/](../app/src/main/res/layout/) — every screen, dialog, list item. File names map closely to features (e.g. `fragment_compose.xml`, `fragment_rule.xml`, `dialog_filter.xml`).
- [app/src/main/res/menu/](../app/src/main/res/menu/) — toolbar/overflow menus per screen — direct list of user actions.
- [app/src/main/res/values/strings.xml](../app/src/main/res/values/strings.xml) — canonical English UI copy. The single best textual catalog of user-visible features (titles, labels, descriptions, settings text, help text, error messages).
- [app/src/main/res/values/arrays.xml](../app/src/main/res/values/arrays.xml) — enumerated choices (sync intervals, themes, encryption modes, etc.).
- [app/src/main/res/xml/](../app/src/main/res/xml/) — `preferences*.xml` (settings hierarchy — a *direct* feature catalog), `shortcuts.xml`, `widgets.xml`, `file_paths.xml`, backup rules, sharing targets.
- [app/src/main/res/drawable*/](../app/src/main/res/drawable/) — icon names hint at features.
- [app/src/main/res/values-*/](../app/src/main/res/) — translations (cross-check copy when EN is ambiguous).
- [app/src/main/res/anim/](../app/src/main/res/anim/), [color/](../app/src/main/res/color/), [font/](../app/src/main/res/font/) — visual/UX details.

### 1.3 Bundled assets & docs (shipped inside the APK)

- [app/src/main/assets/FAQ.md](../app/src/main/assets/FAQ.md) — in-app FAQ (mirrored from repo FAQ).
- [app/src/main/assets/CHANGELOG.md](../app/src/main/assets/CHANGELOG.md) — feature deltas per version.
- [app/src/main/assets/SETUP-*.md](../app/src/main/assets/) — per-locale setup guides.
- [app/src/main/assets/](../app/src/main/assets/) — ISP database (`providers.xml`-style), security blocklists, tutorials, etc. Worth a full directory walkthrough.
- [app/src/main/resExtra/](../app/src/main/resExtra/) and [app/src/main/resources/](../app/src/main/resources/) — additional packaged resources.

### 1.4 Repository-root documentation

- [README.md](../README.md) — high-level "Main features" list — the user-facing pitch (good seed list).
- [FAQ.md](../FAQ.md) / [FAQ.yaml](../FAQ.yaml) — *the* canonical feature/behavior reference, hundreds of Q&A entries. Often the only place where rationale and edge cases are documented.
- [CHANGELOG.md](../CHANGELOG.md) — chronological feature additions/changes.
- [SETUP.md](../SETUP.md) — provider setup matrix.
- [PRIVACY.md](../PRIVACY.md) — privacy-relevant features (tracker blocking, image proxy, etc.).
- [SECURITY.md](../SECURITY.md) — security posture/features.
- [ATTRIBUTION.md](../ATTRIBUTION.md) — third-party libraries → indirectly reveals capabilities (JavaMail, OpenPGP, BouncyCastle, etc.).
- [PLAYSTORE.txt](../PLAYSTORE.txt) — Play Store description (curated feature list).
- [data_safety_export.csv](../data_safety_export.csv) — Play data-safety declaration (data types collected/used → maps to features).
- [tutorials/](../tutorials/) — `FIRST-CONFIG.md`, `MANUAL-CONFIG.md`, `SETTINGS-OVERVIEW.md`, plus screenshots in `tutorials/images/`.
- [metadata/en-US/](../metadata/en-US/) — Fastlane store listing: `title.txt`, `short_description.txt`, `full_description.txt`, `changelogs/`, `images/` (used by F-Droid / store fronts).
- [screenshots/](../screenshots/) — official screenshots (compose, conversation, folders, navigation, widget, …) — visual feature anchors.
- [images/](../images/) — banners, marketing assets.
- [privacy/](../privacy/) — privacy policy materials.
- [patches/](../patches/) — local patches to vendored libs (reveal customizations).
- [tools/](../tools/) — supporting scripts (often hint at internal data formats / features).

### 1.5 Git history

- `git log` / `git blame` / `git log --follow <file>` — when a feature was introduced, by which commit, with what message. Use to date features and find related commits.
- Tags / release commits — map versions to feature batches (cross-reference with CHANGELOG).

---

## 2. Official external sources

- **Project website** — https://email.faircode.eu/ (FairEmail home page, feature highlights, pricing).
- **FAQ (web)** — https://github.com/M66B/FairEmail/blob/master/FAQ.md (rendered version of in-repo FAQ; often deep-linked from the app).
- **GitHub repository** — https://github.com/M66B/FairEmail
  - Releases page (per-version notes).
  - Issues (closed/open) — often the most detailed discussion of feature behavior, edge cases, and rationale from the author M66B.
  - Pull requests / commits — implementation history.
  - Discussions tab (if enabled).
- **FairCode (vendor)** — https://www.faircode.eu/ — corporate site, support contact, related products.
- **Support contact form** — https://contact.faircode.eu/?product=fairemailsupport
- **Crowdin translation project** — https://crowdin.com/project/open-source-email — string keys + translations (sometimes reveals strings not yet shipped).
- **Distribution listings** (descriptions, screenshots, what's-new):
  - Google Play — https://play.google.com/store/apps/details?id=eu.faircode.email
  - F-Droid (IzzyOnDroid) — https://apt.izzysoft.de/fdroid/
  - GitHub Releases — APK + release notes
  - Amazon Appstore (see `app/src/amazon/`)
- **Privacy policy & data-safety pages** on the project/store sites.

---

## 3. Community / third-party sources

- **Reddit** — r/FairEmail, r/fossdroid threads (user-reported workflows, real usage patterns).
- **XDA Developers** forum threads.
- **Stack Overflow / Android Stack Exchange** — Q&A referencing FairEmail behavior.
- **Reviews** on Play Store / F-Droid / AlternativeTo — what users perceive as headline features.
- **Blog posts and comparison articles** ("best privacy email apps for Android") — feature comparisons.
- **YouTube walkthroughs / tutorials** — visual feature demos.

---

## 4. Runtime / dynamic inspection (last resort, complementary)

- Install the app and explore the UI: **Settings** screen hierarchy is the most direct user-facing feature index.
- Inspect SharedPreferences keys at runtime (cross-reference with code-side preference keys).
- Inspect the Room database on device (entities, indices).
- Notification channels, app shortcuts, share targets, tile services — enumerable from a running install.
- `adb shell dumpsys package eu.faircode.email` — declared components.

---

## Suggested traversal order for feature extraction

1. Seed list from **README.md "Main features"**, **PLAYSTORE.txt**, and **metadata/en-US/full_description.txt**.
2. Expand using **`res/xml/preferences*.xml`** (settings = features) and **`res/menu/`** (actions = features).
3. Cross-reference each candidate feature against **strings.xml** for canonical naming.
4. Pull behavioral detail from **FAQ.md** and the matching **Java class(es)**.
5. Use **CHANGELOG.md** + **git log** to date the feature and find its introducing commit.
6. Validate with **screenshots/** and **tutorials/** for the user-facing surface.
7. Fall back to **GitHub issues** and the **website** for nuance not captured in code or docs.
