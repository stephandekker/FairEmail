## Parent Feature

#3.16 Per-Message Actions

## What to build

Extend the flag/star feature with a color picker that lets users associate an arbitrary RGB color with a flagged message. The chosen color is stored locally and displayed as a tinted star icon. Additionally, recognise known server-side color keywords — Open-Xchange `$cl_1`–`$cl_10` and provider label keywords `$label1`–`$label5` — and display their predefined colors automatically when syncing from the server (FR-19, FR-21, FR-22).

## Acceptance criteria

- [ ] Long-pressing (or secondary-clicking) the flag control opens a color picker
- [ ] Selecting a color stores it locally and displays a tinted star icon
- [ ] Clearing the color reverts to the default star appearance while keeping the flag set
- [ ] Messages with Open-Xchange `$cl_7` keyword display an orange flag color without user configuration (AC-7)
- [ ] Messages with provider `$label1`–`$label5` keywords display the corresponding predefined colors
- [ ] Flag colors persist across application restarts

## Blocked by

- Blocked by 1-toggle-flag-star

## User stories addressed

- US-17 (associate a color with a flagged message)
- US-20 (recognise server-side named color keywords)

## Notes

- Open question OQ-2 from the epic: whether user-chosen colors should be written back to the server as keywords is unresolved. This story implements local-only color storage plus server keyword recognition, matching the source application's behaviour (Design Note N-4). Writing colors back is deferred pending a decision.
