# Certificate Pinning on Test Failure

## Parent Feature

#1.4 Manual Server Configuration

## What to build

When the inbound connection test fails due to an untrusted server certificate, display the certificate's fingerprint and offer a "Trust this certificate" action. Accepting this action pins the fingerprint for this account, bypassing the system trust store for future connections to this server. After trusting, re-testing should succeed.

Per design note N-4, certificate pinning is presented only in response to a failed test, not as a proactive setting — the user must see the actual certificate before trusting it.

Covers epic sections: FR-15.

## Acceptance criteria

- [ ] When a test fails due to an untrusted certificate, the certificate fingerprint is displayed
- [ ] A "Trust this certificate" action is offered alongside the error
- [ ] Accepting the trust action pins the certificate fingerprint for this account
- [ ] After trusting, re-running the test succeeds without the certificate error
- [ ] The pinned fingerprint is persisted with the account settings
- [ ] Certificate pinning is only offered in response to a failed test, not as a proactive setting

## Blocked by

- Blocked by `5-inbound-test-connection`

## User stories addressed

- US-17 (display fingerprint and offer trust on untrusted certificate)

## Notes

Open question OQ-3 from the epic: certificate pinning lifetime is currently indefinite (no expiration). A certificate rotation on the server would trigger a new trust prompt, but there is no mechanism to review pinned certificates. This should be flagged for a future design decision.
