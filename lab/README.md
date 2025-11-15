# labs — experiments (madman mode)

A tiny, messy playground for throwing ideas at the wall and seeing what sticks. Keep it chaotic, keep it honest, and commit the interesting bits.

## Philosophy
- Try fast. Break things faster.
- Capture only what helps you reproduce or remember a spark.
- Prefer tiny, disposable experiments over grand plans.

## Quickstart
1. Create a new folder for your experiment: `mkdir labs/<short-name>`  
2. Add a one-line README: `labs/<short-name>/README.md` describing purpose and steps.  
3. Use branches liberally. If it works, prune or promote.

## Repo layout (suggested)
- labs/
    - 2025-11-15-idea-slug/ — timestamped experiments
    - snippets/ — reusable bits worth rescuing
    - notes.md — quick capture of stray thoughts

## How to run an experiment
- Write a single command in the experiment folder (README or run.sh).
- Keep inputs small, outputs observable.
- Capture results and next steps before you forget.

Example:
```
labs/2025-11-15-crazy-idea/
    README.md     # what and why
    run.sh        # one-liner to reproduce
    output.log
```

## Tips
- Favor ephemeral branches: delete when done.
- Tag survivors with `survivor-YYYYMMDD`.
- One experiment = one responsibility.

## Clean-up ritual
- Weekly review: delete trash, rescue gems, summarize survivors.
- Move rescued code to `snippets/` or a proper project.

Happy chaos.