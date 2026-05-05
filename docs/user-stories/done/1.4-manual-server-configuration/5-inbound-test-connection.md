# Inbound Test Connection — Happy Path

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add a "Test Connection" button to the inbound configuration screen. When pressed, the application connects to the configured server using the entered parameters (host, port, encryption, username, password), authenticates, enumerates the server's folder list (IMAP), and disconnects cleanly. During the test a progress indicator is visible and all input fields and buttons are disabled.

On success, the test reports:
- The list of discovered folders (IMAP) with auto-detected folder types
- Whether the server supports IDLE (push); if not, a warning about polling fallback
- Whether the server supports UTF-8

The test must begin within one second of pressing the button (NFR-1) and time out after a reasonable period rather than hanging indefinitely.

This story covers the happy path only. Error diagnostics are in a separate story.

Covers epic sections: FR-32 through FR-35, NFR-1.

## Acceptance criteria

- [ ] A "Test Connection" button is present on the inbound configuration screen
- [ ] Pressing the button initiates a live connection using the currently entered parameters
- [ ] A progress indicator is displayed during the test
- [ ] All input fields and buttons are disabled during the test
- [ ] On success, the discovered folder list is displayed (IMAP)
- [ ] Folder types (Inbox, Sent, Drafts, Trash, Spam, Archive) are auto-detected
- [ ] The test reports whether the server supports IDLE; if not, a polling fallback warning is shown
- [ ] The test reports whether the server supports UTF-8
- [ ] The test begins within one second of pressing the button
- [ ] The test times out with a failure message rather than hanging indefinitely
- [ ] After the test completes (success or failure), input fields are re-enabled

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-13 (test connection validates config before save)
- US-14 (successful test shows folder list with auto-detected types)
- US-15 (reports IDLE support)
- US-18 (progress indicator, fields disabled during test)
