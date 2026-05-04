# POP3 Protocol Support and Specific Options

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Enable POP3 as an inbound protocol option alongside IMAP. When POP3 is selected, the configuration screen displays POP3-specific options:

- **Leave messages on server** (default: on)
- **Delete locally when removed from client** (only enabled when "Leave on server" is off)
- **Leave deleted messages** (default: on)
- **Leave on device** (default: on)
- **Maximum messages** (optional integer; limits messages retained locally)

POP3 port defaults: SSL/TLS -> 995, STARTTLS -> 110, None -> 110. POP3 does not support server-side folders, so folder discovery and role assignment (story 7) do not apply. The test connection success screen for POP3 should confirm the connection succeeded without displaying a folder list.

Changing "Leave on server" on an existing account triggers a re-evaluation of the local message store at next sync.

Covers epic sections: FR-1, FR-3, FR-55, FR-56.

## Acceptance criteria

- [ ] POP3 is available as an inbound protocol option at account creation
- [ ] Selecting POP3 shows POP3-specific options (leave on server, delete locally, leave deleted, leave on device, max messages)
- [ ] "Leave messages on server" defaults to on
- [ ] "Delete locally when removed" is only enabled when "Leave on server" is off
- [ ] "Maximum messages" accepts an optional integer
- [ ] Port auto-fill works with POP3 defaults: SSL/TLS -> 995, STARTTLS -> 110, None -> 110
- [ ] POP3 test connection reports success without a folder list
- [ ] Changing "Leave on server" on an existing account triggers re-evaluation at next sync

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-30 (leave messages on server option)
- US-31 (delete locally when removed)
- US-32 (maximum messages field)
- US-33 (leave deleted messages option)

## Notes

Open question OQ-4 from the epic: POP3 does not support server-side folders. The source application does not display a folder list after a POP3 test. Whether the POP3 test success screen should display any additional confirmation beyond "connection succeeded" is undecided.
