# Alarm Clock website


## Overview

A website that simulates an Alarm Clock. It will show the current local time and two alarms can be set.
When an alarm goes off (i.e. visually), the user has the option to snooze or cancel.

## Claude skipping files

Do not read/use/write any of the files in: /claude-skip/


## System Requirements

## Tech Stack

- **Frontend:** React 19 + TypeScript + Vite + Tailwind CSS v4 + shadcn/ui + React Router v7
- **Backend:** Node.js + Express + TypeScript
- **Database:** SQLite with Drizzle ORM
- **Auth:** JWT-based (register, login, protected routes)
- **Package Manager:** npm
- **Testing:** Vitest (all layers), React Testing Library (frontend components), msw (HTTP mocking for scraper tests), Playwright (E2E tests)

## Project Structure

```
garden-automation/
├── CLAUDE.md
├── package.json
├── .env                          # Configuration
├── src/
├── docs/
│   ├── epics/
│       ├── user-stories/
│           ├── tasks/
│           ├── bugs/

```

## Bug template
Bugs should be written to .md file in /docs/epics/user-stories/bugs using filename format: "yyyy-MM-dd_hh-mm-ss, {TITLE}.md" where the date-time is the discovery.
The content of the file:
- Title
- Description
- Steps to reproduce
- Expected
- Actual
- Additional context info

## File Ownership Rules (CRITICAL)
Each agent must ONLY create and edit files in their assigned directory. This prevents merge conflicts.

| Agent | Owned directories | Do NOT touch |
|-------|-------------------|--------------|
| DB Engineer | `src/db/` | everything else |
| Backend Dev | `src/api/`, `src/services/`, `tests/api/`, `tests/services/`, `.env` | `src/db/`, `src/client/` |
| Frontend Dev | `src/client/` (including co-located `*.test.tsx` and `src/client/lib/api.ts`) | `src/db/`, `src/api/`, `src/services/`, `tests/` |
| Tester | `e2e/` (Playwright E2E tests) | `src/`, `tests/` |
| UX Reviewer | **read-only** — no file ownership; creates Bugs issues for major problems | (never writes files) |

### Test ownership rationale
Each agent owns the tests for the code they write, enabling TDD within each slice:
- **Backend Dev** writes `tests/api/` and `tests/services/` tests alongside their route and service code
- **Frontend Dev** writes component tests co-located in `src/client/components/`
- **Tester** writes E2E tests in `e2e/` using Playwright, organized by priority tiers (platinum/gold/silver/all)

### UX review rationale
The UX Reviewer runs after the Frontend Dev and provides two-tier feedback: minor issues are relayed back to the Frontend Dev to fix before the commit; major issues become GitHub tickets for a future ralph loop.

## Coding Standards
- TypeScript strict mode everywhere
- Use `async/await`, no raw callbacks
- Express error handling: wrap async routes with try/catch, return proper HTTP status codes
- Frontend: functional components only, no class components
- Use named exports, not default exports
- All API responses follow: `{ data: T }` for success, `{ error: string }` for failure
- Passwords hashed with bcrypt (min 10 salt rounds)
- JWT tokens expire in 7 days
- CORS enabled for `http://localhost:5173` (Vite dev server)

## Running the Project
```bash
# Install dependencies
npm install

# Run database migrations
npx tsx src/db/migrate.ts

# Seed development data (adds example searches and categories)
npx tsx src/db/seed.ts

# Start backend (port 3001)
npx tsx src/api/index.ts

# Start frontend (port 5173)
cd src/client && npx vite

# Run the daily pipeline manually
npx tsx src/services/scraper.ts

# Run all unit tests
npx vitest run

# Run E2E tests (platinum tier — fast, top-priority flows)
npx playwright test --project=platinum

# Run E2E tests (all tiers)
npx playwright test --project=all
```
