## Parent Feature

#3.16 Per-Message Actions

## What to build

A global preference that selects between soft delete (mark with IMAP `\Deleted` flag without expunging — message remains recoverable) and hard delete (mark and immediately expunge — irreversible). Hard delete is the default. The "delete permanently" action always hard-deletes regardless of this preference. For POP3 accounts, add a "leave on server" option: deleted messages can be marked locally but retained on the server, or fully removed, as configured (FR-59, FR-65, Design Note N-9).

## Acceptance criteria

- [ ] A global preference allows choosing between soft and hard delete
- [ ] With soft delete, deleting marks with `\Deleted` but does not expunge (AC-18)
- [ ] With soft delete, messages remain on server and can be recovered (AC-18)
- [ ] With hard delete (default), deleting marks and expunges immediately
- [ ] "Delete permanently" always hard-deletes regardless of the preference
- [ ] POP3 accounts offer a "leave on server" option for deleted messages
- [ ] Preference persists across restarts

## Blocked by

- Blocked by 16-delete-to-trash-with-undo

## User stories addressed

- US-57 (choose between soft and hard delete)
