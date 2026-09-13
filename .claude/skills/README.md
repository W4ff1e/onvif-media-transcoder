# Project-local skills

Each directory here is one skill, discovered by Claude Code as `.claude/skills/<name>/SKILL.md`.
They are vendored copies of upstream skills, checked in so the
wording stays fixed and so the repo does not depend on a plugin being installed.

Provenance, so a refresh can be diffed against the source it came from:

| Skill | Upstream | Pinned at |
| --- | --- | --- |
| `unslop` | `cursor/plugins` `pstack/skills/unslop` | fetched 2026-08-19 |
| `technical-writing` | `cursor/plugins` `pstack/skills/technical-writing` | fetched 2026-08-19, repo at `60c641e` |
| `handoff` | `mattpocock/skills` `skills/productivity/handoff` | `9c9f36c`, 2026-08-17 |
| `grill-me` | `mattpocock/skills` `skills/productivity/grill-me` | `9c9f36c`, 2026-08-17 |
| `grilling` | `mattpocock/skills` `skills/productivity/grilling` | `9c9f36c`, 2026-08-17 |
| `writing-for-agents` | `mattpocock/skills` `skills/productivity/writing-for-agents` | `9c9f36c`, 2026-08-17 |
| `improve-codebase-architecture` | `mattpocock/skills` `skills/engineering/improve-codebase-architecture` | `9c9f36c`, 2026-08-17 |
| `codebase-design` | `mattpocock/skills` `skills/engineering/codebase-design` | `9c9f36c`, 2026-08-17 |
| `domain-modeling` | `mattpocock/skills` `skills/engineering/domain-modeling` | `9c9f36c`, 2026-08-17 |

Every `.md` file is byte-identical to its upstream copy. The only thing dropped is each
`mattpocock/skills` skill's `agents/openai.yaml`, which configures Codex rather than Claude Code.

## Three of these are dependencies, not standalone picks

`grilling`, `codebase-design` and `domain-modeling` are here because other skills call them by name
through the Skill tool. Deleting one breaks the caller silently, since a skill that names a missing
skill just fails to reach it.

- `grill-me` calls `grilling`. The whole body of `grill-me` is that one call.
- `improve-codebase-architecture` calls `codebase-design`, `grilling` and `domain-modeling`.

`domain-modeling` and `improve-codebase-architecture` both expect a `CONTEXT.md` domain glossary and
ADRs under `docs/adr/`. Neither exists in this repo, and both skills create them on demand, so the
first run of either will offer to write files this repo has never had. Decide that deliberately
rather than in the middle of a review.

## Invocation

`handoff`, `grill-me`, `improve-codebase-architecture` and `technical-writing` carry
`disable-model-invocation: true`, so they run only when a person types `/handoff`, `/grill-me`,
`/improve-codebase-architecture` or `/technical-writing`. The other five can fire on their own
descriptions.

## Refreshing a vendored skill

Re-copy from upstream and update the row above.

```bash
git clone --depth 1 https://github.com/mattpocock/skills.git /tmp/mp-skills
cp /tmp/mp-skills/skills/productivity/handoff/SKILL.md .claude/skills/handoff/SKILL.md

curl -sSL -o .claude/skills/unslop/SKILL.md \
  https://raw.githubusercontent.com/cursor/plugins/main/pstack/skills/unslop/SKILL.md
```

`mattpocock/skills` ships a Claude Code plugin (`claude plugins install mattpocock-skills`) and an
`npx skills@latest add` installer. Neither was used. Both would put a second copy of these skills
beside the ones here, and the plugin's copy updates itself, which is the opposite of pinning.
