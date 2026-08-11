<p align="center">
  <img src="assets/banner.png" alt="gauntlet loop" width="100%">
</p>

# Gauntlet Loop

A skill that turns any goal into one short, paste-ready prompt. That prompt makes your agent pick a real quality bar, split the work into small pieces, run a builder and a separate harsh critic on each one, compare blind against the bar, and keep looping until it wins.

Most agent output stops at "good enough" because nothing is holding it to a standard. This gives it a standard it cannot argue with.

> The gauntlet loop is [Matt Shumer's](https://github.com/mshumer) idea. He wrote the original prompt and named the technique while building [Claude of Duty](https://github.com/mshumer/Claude-of-Duty). This repo packages that pattern as a reusable skill.

## Quick start

**As a plugin.** Add this repo as a marketplace once, then install it wherever
you want it:

```
/plugin marketplace add robonuggets/gauntlet-loop
/plugin install gauntlet-loop@gauntlet-loop
```

Pick the scope when you install: `user` puts it in every project, `project`
commits it so your team gets it too, `local` keeps it to your machine. Updates
come with `/plugin update gauntlet-loop`. Invoke it as
`/gauntlet-loop:gauntlet-loop`, since plugin skills are namespaced.

### Already copied the skill in by hand?

Install the plugin, then run:

```
/gauntlet-loop:migrate
```

It finds every hand-copied copy on your machine, replaces each with a plugin
install **at the same scope**, and keeps whichever ones you had switched off
switched off. It shows you the full list before touching anything, and it leaves
alone any copy you have edited.

Worth doing rather than ignoring: a loose copy and a plugin copy do not override
each other. Claude Code namespaces the plugin one, so you end up with both
`/gauntlet-loop` and `/gauntlet-loop:gauntlet-loop` live, and the loose one never
updates.

Then in your agent:

```
/gauntlet-loop:gauntlet-loop build me a pricing page for my SaaS
```

It offers you 2 or 3 quality bars to aim at, you pick one, and it hands back a single prompt you paste into a fresh session.

## What's included

```
skills/
├── gauntlet-loop/
│   └── SKILL.md      # the whole skill, still one file
└── migrate/
    └── SKILL.md      # /gauntlet-loop:migrate
.claude-plugin/
├── plugin.json       # plugin manifest
└── marketplace.json  # this repo is its own marketplace
bin/                  # migration tool, one build per platform
migrate/              # its source
CHANGELOG.md
README.md
LICENSE               # CC BY 4.0
```

The skill moved from `.claude/skills/gauntlet-loop/` to `skills/gauntlet-loop/`,
which is where Claude Code resolves a plugin's skills, and the invocation gained
the plugin namespace.

## How it works

1. **You give a goal.** Anything. A site, an essay, a CLI tool, a research brief.
2. **It offers 2 or 3 bars.** Each one is a specific, real thing your agent can actually fetch and compare against. Not "award-winning design", but a named page, a named post, a named repo.
3. **You pick one.** It writes one short prompt, around 150 words, and stops.
4. **You paste it into a fresh session.** That agent splits the work, runs builder and critic pairs, and loops.

The critic is the part that matters. It is a separate agent with fresh context, it opens the actual output, it puts your work next to the bar with the labels stripped, and it says which one is better. Not a score out of 10, which drifts upward every round. A pick.

The loop exits when your work wins the blind comparison, or when you stop the run. Never after a fixed number of rounds.

## Why a bar and not a rubric

A rubric asks the agent to grade itself against words it wrote. A bar makes it compare against something that already exists and is undeniably good.

The skill will not accept a vague bar. It checks three things before it writes anything:

- **Named.** A specific thing, not a category.
- **Fetchable.** The critic can screenshot it, read it, run it, or open it. If the agent cannot get the reference, it hallucinates the comparison and approves everything.
- **Comparable.** Both can sit side by side and a judge can pick one.

## Examples

```
/gauntlet-loop a landing page for my running brand, dark and green, has to feel alive
```
Bar becomes a specific brand's live campaign page, screenshotted at desktop and mobile.

```
/gauntlet-loop a 2000 word explainer on vector databases for non-engineers
```
Bar becomes a named writer's actual published posts, judged on which one a non-engineer understands faster.

```
/gauntlet-loop a CLI that formats JSON logs
```
Bar becomes a named tool's implementation plus its benchmark, so taste and a number both have to win.

## Works with any agent

`/loop` and `ultracode` are Claude Code features. `/loop` reruns a prompt until you stop it, and `ultracode` opts a turn into multi-agent orchestration.

For any other agent, the skill swaps those two lines for plain instructions: keep looping until the critic picks ours, and run the builders and critics as parallel subagents. The structure is identical.

## What breaks it

- A vague bar. The critic invents a comparison and approves everything. By far the most common failure.
- The builder judging its own work. The critic needs fresh context and no knowledge of how hard the builder tried.
- A soft critic. Give it a binary job, not a score.
- A fixed round count. The exit is winning, or you calling it.

## Credit

The gauntlet loop technique is **[Matt Shumer's](https://github.com/mshumer)**. He built [Claude of Duty](https://github.com/mshumer/Claude-of-Duty), wrote the [original prompt](https://github.com/mshumer/Claude-of-Duty/blob/main/prompt.md), and named the loop. Every idea underneath this skill - the harsh critic, the blind comparison, the refusal to stop until the work wins - comes from that prompt.

This repo is not the technique. It is a skill that writes a gauntlet loop prompt for you, for any goal, so you do not have to hand-write one each time.

Related reading: [Anthropic on building effective agents](https://www.anthropic.com/engineering/building-effective-agents), which covers the evaluator pattern the loop is built on.

## License

CC BY 4.0. Free to use with attribution.

Skill by Jay E at [RoboNuggets](https://robonuggets.com). Technique by Matt Shumer.
