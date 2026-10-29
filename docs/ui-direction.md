# UI direction

Updated: 2026-10-27

## Goal

Build a deliberate karaoke-machine interface that feels designed for repeated use, not a generic AI-generated dashboard. Keep the current React/Tauri foundation, but replace decorative conventions with clear hierarchy, predictable controls, and dense song-browsing workflows.

## Reference principles

### DAM main karaoke display

Use the DAM machine's room display only as a reference for the singing and post-performance result experiences, not for proprietary assets or trade dress:

- During singing, prioritize lyrics and an immediately readable guide-melody/pitch trace over controls or decoration.
- Keep live feedback glanceable from a distance and never reveal the total score while the song is in progress.
- After the performance, reveal the total prominently, then show a restrained diagnostic breakdown such as pitch accuracy, stability, expression, rhythm/timing, and technique feedback.
- Separate the celebratory score reveal from the detailed review so the first screen is legible across a room.
- Use original graphics, motion, terminology, and scoring presentation; do not reproduce DAM's branded visual treatment.

Official references:

- https://www.clubdam.com/contents/saiten/seimitsusaiten_dx.html
- https://www.clubdam.com/contents/damtomo/damtomo_seimitsusaiten_ai.html

The DAM Denmoku/handset selection interface is explicitly out of scope as a reference.

### Nightingale song selection

Use Nightingale as the functional reference for local-library selection:

- Support folder-first browsing with artist and album hierarchy.
- Offer a compact table as the primary large-library view and artwork grid as an optional view.
- Combine search with location, analysis-state, and lyric-source filters without discarding context.
- Open song details beside the list so the user can inspect metadata and preparation state without losing their place.
- Keep play/prepare and add-to-queue actions distinct; retain per-request key and tempo choices.

Official references:

- https://nightingale.cafe/
- https://nightingale.cafe/docs/getting-started/

## Visual constraints

- Use Material Design 2 as a loose visual-system reference for hierarchy, compact surfaces, restrained elevation, interaction states, and a consistent 4/8 px rhythm. Do not add a Material component dependency solely for appearance.
- Do not copy DAM or Nightingale logos, icons, artwork, exact layouts, colors, or branded terminology.
- Avoid the common generated-dashboard look: excessive rounded cards, glass blur, glowing gradients, floating pills, oversized empty hero areas, and ornamental status copy.
- Use a restrained neutral base, one primary action color, one semantic status palette, and borders/spacing before shadows.
- Prefer a consistent 4/8 px spacing rhythm, modest corner radii, tabular alignment, and system/Japanese UI typefaces.
- Use album artwork only where it improves recognition; do not turn every row into a decorative card.
- Text labels must remain useful without icons. Motion should explain state changes, not decorate idle screens.
- Do not display a decorative product-name lockup in the song-selection navigation; lead with the user's library and tasks.

## Song-selection structure

1. Persistent top area: search, current source, compact/grid switch, and queue entry.
2. Left navigation: folders, artists/albums, history, favorites/saved-for-later, and analysis filters.
3. Main area: compact sortable song table by default, with obvious focus and selection states.
4. Right detail area: selected song metadata, cache/analysis stages, lyrics availability, key/tempo, and separate prepare/play/queue actions.
5. Queue panel: order, singer/request context when added later, key/tempo, reorder, and remove.

The total score remains absent during singing and appears only after the performance, as required elsewhere.

## Singing and result structure

1. Singing: background content, large synchronized lyrics, guide melody/pitch trace, timing cue, and compact technique feedback.
2. Singing controls: secondary and visually quiet; no live total or projected final score.
3. End transition: a short, original reveal that clearly marks the performance as complete.
4. Result summary: final total first, then a compact breakdown and one or two actionable observations.
5. Result detail: section history and diagnostic graphs available after the summary, without crowding the reveal.
