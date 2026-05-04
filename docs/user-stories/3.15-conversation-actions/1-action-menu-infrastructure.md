# Conversation Action Menu Infrastructure

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, when viewing a message, I want access to a reply/action menu listing all available conversation actions, so that I can choose how to respond to or act on the message.

## Blocked by
*(none — this is the foundational slice)*

## Acceptance Criteria
- A reply/action menu is accessible from the message view UI (FR-2).
- The menu lists the core actions: reply, reply-all, forward, and edit-as-new.
- Conditional actions (reply-to-list, send read receipt, redirect/bounce, resend) are hidden when their preconditions are not met, and shown when they are (FR-5).
  - Reply-to-list: visible only when the message has a List-Post header.
  - Send read receipt: visible only when the message has a Disposition-Notification-To header.
  - Redirect/bounce: visible only when the feature is enabled in settings, a Return-Path exists, and it is not the user's own address.
  - Resend: shown in a dimmed/disabled state when message headers have not been downloaded; active otherwise.
- All menu items have descriptive labels, are keyboard-accessible, and are compatible with screen readers (NFR-6).
- Invoking any action from the menu opens the compose window in under one second for locally-available messages (NFR-1).

## Mapping to Epic
- FR-1, FR-2, FR-5, FR-6
- NFR-1, NFR-6

## HITL / AFK
HITL — menu layout and action ordering are UX-sensitive; a design review is recommended before finalising.

## Notes
- This story establishes the menu shell and conditional visibility logic. The individual action behaviors (what happens when each item is clicked) are implemented in subsequent stories.
- The reply button with configurable gestures (FR-3, FR-4) is a separate story (19-reply-button-configuration) that builds on top of this menu.
