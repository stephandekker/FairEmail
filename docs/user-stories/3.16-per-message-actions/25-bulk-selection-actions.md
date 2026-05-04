## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to select multiple messages in the message list and apply any per-message action to all selected messages in a single operation. When multiple messages are selected, a floating action panel appears offering all applicable actions. Bulk actions must handle at least 500 messages without timeout or UI freeze, with progress indication for operations on more than 50 messages (FR-71 bulk portion, FR-77, NFR-7).

## Acceptance criteria

- [ ] Users can select multiple messages in the message list
- [ ] A floating action panel appears when messages are selected
- [ ] Pressing bulk "move" moves all selected messages to the chosen folder with undo (AC-22)
- [ ] All per-message actions work in bulk mode with same semantics as single-message
- [ ] Bulk operations on 500 messages complete without timeout or UI freeze (NFR-7)
- [ ] Progress indication is shown for bulk operations on more than 50 messages
- [ ] Select-all and deselect-all controls are available

## Blocked by

- Blocked by 22-configurable-toolbar-context-menu

## User stories addressed

- US-66 (select multiple messages and apply any action in bulk)
