# Account Display Properties

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add fields to the account configuration screen for account display name (editable, defaults to username/email), optional color (color picker), optional category (free-text), and optional avatar (image). These properties are persisted with the account and used throughout the application to identify the account in navigation, message lists, and account selectors.

Covers epic sections: FR-57, FR-58.

## Acceptance criteria

- [ ] Account configuration screen includes fields for: display name, color, category, and avatar
- [ ] Display name defaults to the username or email address if not explicitly set
- [ ] Color field uses a color picker control
- [ ] Category is a free-text input
- [ ] Avatar allows selecting an optional image
- [ ] All display properties are persisted on save and shown in navigation/account lists

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-7 (account display name, color, category)
