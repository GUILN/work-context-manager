Projects are accommodated inside the `work context repo` (`<repo>/<project>/<context>.md`).

## Change commands
- `context-manager new` — supports `new project` and `new context`; if unspecified, prompts the user
- `context-manager new context` — prompts with the projects to which the context should be created

## New (sub) commands
- `context-manager list` — lists projects inside the `work context repo`
- `context-manager open <Project>` — if project is not specified, prompts all listed projects and lets the user choose; lists all contexts (md documents) inside the chosen project's folder, then opens the chosen one in the editor

## Other concerns
- [x] Update needed documents
- [x] Update Makefile (if needed) — not needed
- [x] Always keep adding tests
- [x] Update my own (local) installation when you finish