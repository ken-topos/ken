# `local/` — your private drop zone

Anything you put in this directory is **gitignored** (except this README), so it
never reaches the public repo. Use it for research notes, AI-conversation
transcripts, scratch files, throwaway experiments — anything you want kept locally
alongside the repo but out of version control.

The ignore rule lives in `../.gitignore`:

```
/local/*
!/local/README.md
```

So a stray `git add -A` cannot sweep these into a commit. If you ever *do* want to
track something from here, move it out of `local/` first.
