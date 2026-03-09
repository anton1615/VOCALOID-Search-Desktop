## Context

The desktop app currently disables Tauri WebView CSP entirely by setting `app.security.csp` to `null` in `src-tauri/tauri.conf.json`. This keeps development friction low, but it also removes a browser-level safeguard that would otherwise limit the blast radius of future HTML injection or script execution mistakes.

The current frontend already contains one explicitly sanitized `v-html` rendering path for video descriptions, and the playback UI depends on an embedded NicoNico iframe. Any CSP change therefore needs to preserve the current embed-player flow, local app bundle loading, and the local dev server workflow used by Tauri development.

## Goals / Non-Goals

**Goals:**
- Replace `csp: null` with a minimal baseline CSP that is compatible with the current desktop app architecture.
- Preserve developer workflow for `npm run dev` + Tauri development without introducing frequent manual policy edits.
- Preserve the current NicoNico embedded player experience in both main and PiP windows.
- Make the app safer against future maintenance mistakes by adding a global WebView policy layer.

**Non-Goals:**
- Redesign or tighten Tauri shell capabilities in this change.
- Eliminate every permissive CSP source on the first iteration.
- Refactor frontend rendering patterns beyond what is needed to stay compatible with the new baseline.
- Introduce domain-specific link allowlisting for external page opening in this change.

## Decisions

### 1. Introduce a minimal baseline CSP instead of aiming for a strict CSP immediately

The change will replace `csp: null` with a CSP that covers the current app architecture while intentionally remaining modest in scope. The purpose of this first step is to restore a global protection layer without turning the change into a broad frontend hardening project.

**Rationale:**
- The current risk comes from having no CSP at all, not from the app already operating under a slightly permissive policy.
- A minimal baseline produces meaningful security improvement with lower breakage risk.
- A strict-first approach would likely require multiple rounds of resource auditing and frontend cleanup that are out of scope for this hardening pass.

**Alternatives considered:**
- Keep `csp: null` and rely on local sanitization only. Rejected because it leaves future maintenance mistakes unbounded.
- Adopt a highly restrictive CSP immediately. Rejected because it adds more compatibility uncertainty than needed for the first hardening step.

### 2. Treat embedded NicoNico playback as a first-class compatibility requirement

The CSP baseline must explicitly account for the current NicoNico iframe playback flow and any resource categories it depends on. The policy must support both main-window and PiP-window playback because both are part of the current product behavior.

**Rationale:**
- Embedded playback is a core feature, not an optional integration.
- Security hardening that silently breaks iframe loading or player communication would not be acceptable.
- Writing this requirement into the design prevents the implementation from optimizing only for static app rendering.

**Alternatives considered:**
- Add a generic CSP and discover playback breakage later. Rejected because the embed flow is already a known sensitive integration.

### 3. Keep shell permissions unchanged and document them as future hardening work

This change will not alter `shell:allow-open`. Instead, the change will record that shell permission tightening remains a future security-hardening concern outside the scope of the CSP baseline work.

**Rationale:**
- The current codebase shows a narrow usage of shell open behavior, while the more urgent global gap is the missing CSP.
- Deferring shell changes keeps this change small and lowers implementation risk.
- Explicit documentation prevents the issue from being forgotten while avoiding scope creep.

**Alternatives considered:**
- Tighten shell permissions as part of the same change. Rejected because it mixes two hardening tracks with different verification needs.

### 4. Validate the new baseline in both dev and packaged contexts

Implementation must verify that the CSP works during local Tauri development and after frontend production build packaging. Manual playback validation should include loading the embedded player and confirming that the main playback UI remains functional.

**Rationale:**
- Tauri development and packaged execution use different asset-loading paths.
- A CSP that works only in one mode would create recurring developer friction or release instability.
- Security settings need runtime verification, not just config edits.

**Alternatives considered:**
- Validate only production build behavior. Rejected because development breakage would create immediate workflow pain.

## Risks / Trade-offs

- **[A source is omitted from the initial baseline]** → Mitigation: verify dev startup, packaged build, and embedded playback manually before considering the change complete.
- **[The baseline remains somewhat permissive]** → Mitigation: accept this as an intentional first-step trade-off and document shell hardening plus future CSP tightening as follow-up work.
- **[Future contributors assume CSP alone replaces sanitization]** → Mitigation: keep existing HTML sanitization paths intact and frame CSP as an additional layer, not a substitute.
- **[Different window contexts behave differently under the new policy]** → Mitigation: validate both main and PiP playback flows during manual verification.

## Migration Plan

1. Replace `csp: null` with the agreed minimal baseline in Tauri config.
2. Run the desktop app in development mode and confirm the app shell and embedded playback still load.
3. Build the frontend / packaged app path and confirm the same baseline remains compatible.
4. Update specs so the new desktop WebView security baseline is explicitly documented.
5. Leave shell capability behavior unchanged and note its deferred hardening status in spec-level documentation where appropriate.

Rollback is straightforward: if the baseline unexpectedly breaks the app and cannot be corrected within scope, revert to the previous CSP setting and revisit the policy design with narrower incremental changes.

## Open Questions

- Which exact CSP directives and source lists are minimally required by the current Vue + Tauri + NicoNico embed stack in both dev and production?
- Does any current image, media, or network path rely on additional remote origins that must be explicitly allowed by the baseline?
- Should the deferred shell-hardening note live only in the new security capability spec, or also be mentioned in an existing baseline spec for implementation visibility?
