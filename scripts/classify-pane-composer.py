#!/usr/bin/env python3
"""Classify what is sitting on an agent pane's composer line.

Reads `tmux capture-pane -e -p` output on stdin (escape sequences REQUIRED --
see below) and prints exactly one verdict word:

    queued        a delivery is queued behind an active turn -- HEALTHY, leave it
    paste         `[Pasted Content ...]` stranded on the composer -- submit it
    slash:<cmd>   an allow-listed slash command stranded, unsubmitted -- submit it
    ghost         the composer holds the UI's own suggestion text -- NOT input
    other         the composer holds something else -- ⛔ NEVER submit this
    clear         nothing on the composer

⛔ WHY THE ESCAPE SEQUENCES ARE NOT OPTIONAL. An idle pane renders a *suggestion*
on its composer line -- "Explain this codebase", "Find and fix a bug in
@filename". With colour stripped that is indistinguishable from text a sender
actually delivered, and a sweep that submits it sends the agent a fabricated
instruction nobody wrote. Measured 2026-07-26: the UI wraps suggestion text in
`ESC[2m` (dim) and real composer content undimmed, so the escape stream carries
the one bit that plain text does not. Capture with `-e` or this cannot be
answered -- it is not a heuristic we are choosing over a better one, it is the
only available discriminator.

⛔ AND `other` IS DELIBERATELY NOT ACTIONABLE. Submitting arbitrary composer text
is not a repair: it could be a half-typed operator command, or content whose
sender is mid-compose. Only a stranded *paste* (a delivery that provably already
arrived complete) and an allow-listed *slash command* are safe to press Enter on.
Widening this allow-list widens what the fleet can be made to run unattended.
"""
import re
import sys

# ⛔ Allow-list, not a pattern. Each entry is a command whose ONLY effect is on
# the receiving seat's own context, so submitting one that was already stranded
# cannot act on the repo, the fleet, or GitHub.
SAFE_SLASH = {"/compact"}

QUEUED = "Messages to be submitted after next tool call"
PROMPT = "›❯>"          # Codex `›`, Claude `❯`, plain `>`
SGR = re.compile(r"\x1b\[([0-9;]*)m")


def is_dim_sgr(seq):
    """True if this SGR sequence turns dim ON.

    ⛔ Do NOT spell this as a substring/regex test for `2m`: `ESC[22m` is dim
    *OFF* and `ESC[12m` is a font selector, and both end in `2m`. Parse the
    parameters and look for the exact code 2, or the check inverts on the very
    sequence that would clear the attribute.
    """
    m = SGR.fullmatch(seq)
    if not m:
        return False
    return "2" in [p for p in m.group(1).split(";")]


def composer_content(line):
    """Return (text, is_dim) for a composer line, or None if not one."""
    # Walk past leading whitespace and SGR runs to find the prompt glyph.
    i = 0
    while i < len(line):
        m = SGR.match(line, i)
        if m:
            i = m.end()
            continue
        if line[i] in " \t":
            i += 1
            continue
        break
    if i >= len(line) or line[i] not in PROMPT:
        return None
    rest = line[i + 1:]
    # The dim attribute must apply to the CONTENT, so look only at the SGR runs
    # between the prompt glyph and the first visible character. A dim run later
    # in the line styles something else and says nothing about the composer.
    j, is_dim = 0, False
    while j < len(rest):
        m = SGR.match(rest, j)
        if m:
            if is_dim_sgr(m.group(0)):
                is_dim = True
            j = m.end()
            continue
        if rest[j] in " \t":
            j += 1
            continue
        break
    return SGR.sub("", rest).strip(), is_dim


def main():
    pane = sys.stdin.read()
    if QUEUED in SGR.sub("", pane):
        print("queued")
        return
    verdict = "clear"
    for line in pane.splitlines():
        got = composer_content(line)
        if got is None:
            continue
        text, is_dim = got
        if not text:
            continue
        if text.startswith("[Pasted Content"):
            verdict = "paste"       # a paste is never a suggestion
            break
        if is_dim:
            verdict = "ghost"
            continue
        if text in SAFE_SLASH:
            verdict = "slash:" + text
            break
        verdict = "other"
    print(verdict)


if __name__ == "__main__":
    main()
