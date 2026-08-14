---
name: gauntlet-loop
description: Turns any goal into one short, paste-ready "gauntlet loop" prompt - a prompt that makes an agent set a concrete quality bar, split the work into small judgeable pieces, run a builder and a separate harsh critic on each, compare blind against the bar, and loop until it wins. Works for builds, writing, code, research, or design. Triggers on "/gauntlet-loop:gauntlet-loop", "gauntlet loop", "gauntlet this", "make a gauntlet prompt", "loop until it beats X".
argument-hint: [goal]
allowed-tools: Bash(${CLAUDE_SKILL_DIR}/scripts/resolve-bars.sh *)
license: CC-BY-4.0
---

# Gauntlet Loop

The user gives a goal. You give back ONE short prompt they can paste into a fresh agent session.

You are not doing the work. You are writing the prompt that makes another agent grind on the work until it beats a real reference.

## Flow

1. **Read the goal.** One line restatement in your head, not on screen.
2. **Find the bar material.** Take the first of these that hits, then stop looking:
   - The bar is already in your context, or the user named it. Use it.
   - A bars directory is configured. Read what is in it. See [Configured bars](#configured-bars).
   - Neither. **Ask one question** and wait: is there a spec, requirements doc, or acceptance criteria to measure against, or should the bar be an outside product?

   Do not go hunting through the project for a spec. Every project names and arranges these differently, some keep them in archives or scratch directories, and a wrong file silently becomes the wrong bar for the entire run.
3. **Set the bar.** If you have one, use it. If not, offer **2 or 3 candidate bars**, one line each, and stop. Wait for their pick. Do not write the prompt yet.
4. **Write the prompt.** One block, paste-ready, no preamble, no headings inside it, no narration after it.
5. **Offer to run it.** One flat line under the prompt: "I can run this here." Not a question.

If they say run it, you become the lead agent and follow the prompt you just wrote.

## The bar is the whole trick

Everything else in a gauntlet loop is scaffolding. The loop only produces quality if the thing it compares against is real.

A bar has to pass three tests:

- **Named.** A specific thing, not a category. "Stripe's pricing page" works. "Award-winning SaaS sites" does not.
- **Fetchable.** The critic can actually get it - screenshot the live page, read the published piece, run the binary, open the repo, watch the footage. If the agent cannot obtain it, it will hallucinate the comparison.
- **Comparable.** Both can sit side by side and a judge can pick one. If you cannot imagine the A/B, it is not a bar.

Bars by goal type:

| Goal | Bar that works |
|---|---|
| Website, app, UI | The live site of a specific best-in-class product, screenshotted at the same viewport |
| Game, 3D, visual | Real footage or screenshots from a named shipped title |
| Writing | A specific published piece by a named author or publication, same length and format |
| Code, tooling | A named repo's implementation, plus its benchmark or test suite as the measurable half |
| Research, analysis | A named analyst report or a paper's methods section, judged on rigour and coverage |
| Deck, doc, deliverable | A real artifact from a firm known for it, same page count |
| **New work with no outside equal** | **The project's own spec, requirements, or acceptance criteria, read as pass/fail** |

When you propose bars, prefer the hardest one the agent can genuinely reach. A bar that is too easy makes the loop exit on round one.

If the goal has a measurable half (load time, token cost, benchmark score, word count, pass rate), name it alongside the reference. Taste plus a number beats taste alone.

## When the bar is the project's own spec

Most real work has nothing to copy. Internal tools, billing rules, anything built for one business - there is no shipped product to hold it against. This is the normal case, not the exception, and it is where a gauntlet loop quietly fails: with no outside reference, the critic invents a standard and approves work against it.

A spec fixes that, but only if it is judgeable.

**Gate it before you use it.** A spec is a valid bar only if a critic can read a line and answer pass or fail without deciding anything. Vision docs, goals, and prose descriptions fail the **Comparable** test - they read as agreement rather than judgement, and a soft critic passes everything.

If the material is prose, say so and offer to convert it into a pass/fail checklist first. One line per check, each one answerable without interpretation. That checklist becomes the bar. Do not write the gauntlet prompt against prose.

**Prefer both bars when both exist.** A spec says *what*. An outside product says *how good*. Real dev work usually wants both, and they judge different failures - a build can satisfy every requirement and still feel cheap, or look excellent and do the wrong thing. Name both in the prompt and let the critic check each in turn.

**Silence is not permission.** A spec never covers everything. Say plainly in the prompt that where the spec is silent, the builder flags the gap rather than inventing an answer - otherwise the invented parts are indistinguishable from the specified ones by the time you read the result.

<a id="configured-bars"></a>
## Configured bars

Users who keep their specs in one place can set that directory once and skip the question every run. Do not work this out yourself - run the resolver and act on what it returns:

```
${CLAUDE_SKILL_DIR}/scripts/resolve-bars.sh
```

It checks, in order: an explicit path you pass as an argument, the plugin's `bars_dir` option, then `bars_dir =` in the project's `.claude/gauntlet-loop.conf`. Relative paths resolve against the project root.

Act on the exit code, and nothing else:

| Exit | Meaning | What you do |
|---|---|---|
| `0` | Resolved. Path and candidate files follow. | Read the candidates and treat them as the bar material |
| `3` | Nothing configured | Ask the user for the bar |
| `4` | Configured, but the path is wrong | Tell the user the path is wrong, and stop |

Never search for a spec because the resolver came back empty. Exit `3` means ask, and exit `4` means the user has a broken setting to fix.

When several candidates come back, name the ones that could apply and let the user pick. Do not merge them, and do not assume the newest or largest is the right one.

The config file belongs to the project, not to this plugin, so it survives plugin updates:

```
bars_dir = docs/specs
```

## Prompt template

Adapt the wording every time. Fill the brackets, keep it short, keep the last line.

```
Build [GOAL].

The bar is [BAR]. Get the real thing first and compare against it directly, not against a description of it.

Break this into the smallest pieces that can be improved and judged on their own. For each piece, fan out a builder and a separate critic with fresh context. The critic inspects the actual output, puts it next to the bar blind with the labels stripped, says which one is better, and names the single biggest remaining gap. Then it goes back to the builder.

The critic should be a harsh critic. Praise is not useful. If ours does not win, it keeps going.

/loop on each piece until the critic picks ours blind. Do not stop before that.

Keep a live progress page updating as the work evolves so I can watch it.

Fan out subagents and ultracode.
```

When the bar is a spec, swap the second paragraph for:

```
The bar is [SPEC PATH]. Every check in it has to come back pass. Read it first and judge against what it actually says, not against your reading of what the project probably wants. Where it is silent, flag the gap and ask - do not decide it yourself.
```

When both bars apply, name them together: the spec for what has to be true, the outside reference for how good it has to look or feel.

Rules for what you fill in:

- Bake the bar in as a concrete, fetchable thing. URL, product name, repo, title, or a path to a checked-in spec.
- Add a budget or cost ceiling line **only if the user named one**. No default cap.
- Add tool names only if the goal needs them (image or video generation, a browser, a deploy target).
- Everything else stays out. No architecture, no file layout, no decomposition, no round count, no stack choice unless the user demanded it. The agent decides those, and it decides better than a spec written before the work started.

## Length and voice

Short. Around 120 to 180 words. If the prompt needs a heading to stay readable, it is too long.

Plain sentences. No bullet lists inside the prompt. It should read like someone telling an agent what perfect looks like and refusing to accept less.

## Portability

`/loop` and `ultracode` are Claude Code features. `/loop` reruns the prompt on an interval or lets the model pace itself. `ultracode` opts the turn into multi-agent orchestration.

For any other agent, swap the last two lines for: "Keep looping until the critic picks ours. Run the builders and critics as parallel subagents." The structure carries over unchanged.

## Two filled examples

**Visual goal.** User: "landing page for my running brand, athletic, green and dark, has to feel alive."

Bars offered: A) Nike's current running campaign page B) On Running's homepage C) Gymshark's product landing page. User picks A.

```
Build a landing page for a running brand. Athletic, peak performance, green and dark, energetic, aimed at a young healthy audience. It needs to be interactive and visually unmistakable.

The bar is Nike's current running campaign page. Screenshot it at desktop and mobile and compare against those directly, not against a description of them.

Break this into the smallest pieces that can be improved and judged on their own - hero, motion, type, colour, imagery, interaction, mobile. For each piece, fan out a builder and a separate critic with fresh context. The critic opens the real page in a browser, puts our screenshot next to Nike's blind with the labels stripped, says which is better, and names the single biggest remaining gap. Then it goes back to the builder.

The critic should be a harsh critic. Praise is not useful. If ours does not win, it keeps going.

/loop on each piece until the critic picks ours blind. Do not stop before that.

Keep a live progress page updating as the work evolves so I can watch it.

Fan out subagents and ultracode.
```

**Non-visual goal.** User: "a 2000-word explainer on vector databases for non-engineers."

Bars offered: A) a specific Stripe engineering blog explainer B) a named Julia Evans post C) the Wikipedia article plus a comprehension test. User picks B.

```
Write a 2000-word explainer on vector databases for readers who are smart but not engineers.

The bar is Julia Evans' writing on hard technical topics. Pull three of her actual posts and compare against them directly, not against a description of her style.

Break this into the smallest pieces that can be judged on their own - the opening, each explanation, the diagrams, the analogies, the ending. For each piece, fan out a writer and a separate critic with fresh context. The critic reads ours and hers blind with the bylines stripped, says which one a non-engineer would understand faster, and names the single biggest remaining gap. Then it goes back to the writer.

The critic should be a harsh critic. Praise is not useful. If ours does not win, it keeps going.

/loop on each piece until the critic picks ours blind. Do not stop before that.

Keep a live progress page updating as the work evolves so I can watch it.

Fan out subagents and ultracode.
```

**Internal goal, no outside equal.** User: "the leave request and approval flow for our HR system."

No outside product to copy. The user confirms an acceptance checklist exists at `docs/hr/acceptance.md`.

```
Build the leave request and approval flow for our HR system.

The bar is docs/hr/acceptance.md. Every check in it has to come back pass. Read it first and judge against what it actually says, not against your reading of what the project probably wants. Where it is silent, flag the gap and ask - do not decide it yourself.

Break this into the smallest pieces that can be improved and judged on their own - request submission, balance rules, the approval chain, notifications, the audit trail. For each piece, fan out a builder and a separate critic with fresh context. The critic runs the actual flow, walks the checks that cover that piece, marks each pass or fail, and names the single biggest remaining gap. Then it goes back to the builder.

The critic should be a harsh critic. Praise is not useful. A partial pass is a fail.

/loop on each piece until every check covering it passes. Do not stop before that.

Keep a live progress page updating as the work evolves so I can watch it.

Fan out subagents and ultracode.
```

## What breaks a gauntlet loop

- **A vague bar.** The critic invents a comparison and approves everything. Most common failure by far.
- **A bar that is prose, not checks.** A spec full of goals and intentions reads as agreement rather than judgement. Convert it to pass/fail first.
- **Guessing which file is the spec.** A wrong bar is worse than no bar, because the loop still exits confidently. Ask, or read the configured path. Never search.
- **The builder judging its own work.** The critic must be a separate agent with fresh context. It should not know how hard the builder tried.
- **A soft critic.** Say "harsh" in the prompt and give it a binary job: which one is better, A or B. Scores out of 10 drift upward every round.
- **Named exit after N rounds.** The exit is winning the comparison, or the user stopping the run. Never a round count.
- **Over-specifying.** Every extra instruction is one fewer decision the agent makes with its own judgment. Minimal wins.
