#!/usr/bin/env python3
"""Classify what is sitting on an agent pane's composer line.

Reads `tmux capture-pane -e -p` output on stdin (escape sequences REQUIRED --
see below) and prints exactly one verdict word:

    unreadable    ⛔ nothing was captured, or this was misused -- NOT `clear`
    queued        a delivery is queued behind an active turn -- HEALTHY, leave it
    busy          the seat is mid-turn -- ⛔ NEVER submit, it destroys live work
    paste         `[Pasted Content ...]` stranded on the composer -- submit it
    slash:<cmd>   an allow-listed slash command stranded, unsubmitted -- submit it
    ghost         the composer holds the UI's own suggestion text -- NOT input
    other         the composer holds something else -- ⛔ NEVER submit this
    clear         nothing on the composer

⛔⛔ THE COMPOSER IS THE **LAST** PROMPT-GLYPH LINE, NEVER THE FIRST. A pane holds
its transcript above the composer, and a submitted command stays ECHOED there as
`› /compact` for the rest of the session. This classifier used to iterate forward
and `break` on the first match, so it reported a CONSUMED echo as a live stranded
delivery -- twice on a live seat, 2026-07-26, once while the seat was 14 minutes
into a turn. ⇒ Its verdict was a function of **how much scrollback the caller
happened to capture**, which means it was not measuring the composer at all. A
`-S -40` and a `-S -300` of the same pane at the same instant could disagree.

⛔ AND A BUSY SEAT IS NEVER REPAIRABLE. A high-effort turn shows NEITHER
`Working` NOR `esc to interrupt` -- only a spinner glyph and an elapsed counter --
so keying busy-ness on either word reports IDLE on a seat that is working. The
detector below keys on the *shape* `<glyph> <verb> ... (<elapsed>` and is
deliberately positional: it looks only at the pane TAIL, because the status line
renders directly above the composer while `(1m 3s)` in a transcript echo does not.
⚠ Past-tense forms (`Worked for 1m 47s`, `Cogitated for 34m 18s`) are NOT busy and
are controlled for -- they are the shape a finished turn leaves behind.

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

# ⛔ The BUSY shape, measured -- not guessed. Both live spellings:
#     `• Working (23s • esc to interrupt)`            Codex
#     `✻ Thundering… (14m 13s · ↓ 55.1k tokens)`      Claude, high effort
#     `✽ Scurrying… (32m 6s · thinking with high effort)`
# A leading non-alphanumeric status glyph, then a word, then an elapsed
# parenthetical. ⚠ The spinner VERB is randomized, so it must not appear here;
# and `Working`/`esc to interrupt` must not be required, because a high-effort
# turn prints neither.
BUSY = re.compile(r"^\s*[^\w\s]\s+\S+.*\((?:\d+h\s+)?(?:\d+m\s+)?\d+s\b")

# How many trailing lines count as "the status region". The spinner renders
# directly above the composer; a duration inside a scrollback tool-result echo
# does not. ⛔ Positional, so an elapsed-looking number in the transcript cannot
# manufacture a false BUSY that silently blinds the sweep.
BUSY_TAIL_LINES = 10


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
        if line[i].isspace():
            i += 1
            continue
        break
    if i >= len(line) or line[i] not in PROMPT:
        return None
    rest = line[i + 1:]
    # The dim attribute must apply to the CONTENT, so look only at the SGR runs
    # between the prompt glyph and the first visible character. A dim run later
    # in the line styles something else and says nothing about the composer.
    #
    # ⛔ SKIP ANY UNICODE WHITESPACE HERE, NOT JUST `" \t"` -- @steward, measured
    #    2026-07-26 on a live `moot-runtime-implementer` pane. Claude renders its
    #    composer as `❯` + U+00A0 NO-BREAK SPACE + `ESC[2m` + text, and a
    #    `rest[j] in " \t"` test does not match U+00A0. The loop therefore BROKE
    #    at the separator, never reached the dim run, and returned is_dim=False
    #    for text that was dim on screen.
    #
    #    ⚠ That failed in the UNSAFE direction: `slash:/compact` instead of
    #    `ghost`, so the sweep would have pressed Enter on a healthy Claude
    #    seat's own SUGGESTION TEXT and destroyed its context. `.strip()` below
    #    removes U+00A0, so the text matched the allow-list exactly.
    #
    #    ⭐ The `ghost-slash` control did not catch it, and was not wrong: it is
    #    written with the Codex `›` glyph and an ASCII space, so the entire
    #    Claude glyph+NBSP shape was absent from the control POPULATION. The
    #    detector was fine; the fixtures could not reach the defect. Controls are
    #    now parameterised over both prompt shapes.
    j, is_dim = 0, False
    while j < len(rest):
        m = SGR.match(rest, j)
        if m:
            if is_dim_sgr(m.group(0)):
                is_dim = True
            j = m.end()
            continue
        if rest[j].isspace():
            j += 1
            continue
        break
    return SGR.sub("", rest).strip(), is_dim


def is_busy(plain_lines):
    """True if the pane's status region shows a turn in progress.

    ⛔ Two independent tells, because neither alone covers both UIs: the
    `esc to interrupt` affordance (Codex, and Claude at ordinary effort) and the
    bare spinner+elapsed shape (Claude at high effort, which prints no
    affordance at all). Both are read only within the tail region.
    """
    tail = plain_lines[-BUSY_TAIL_LINES:]
    for line in tail:
        if "esc to interrupt" in line:
            return True
        if BUSY.match(line):
            return True
    return False


def classify(pane):
    """Return the verdict word for one `capture-pane -e -p` capture."""
    # ⛔ An EMPTY capture is `unreadable`, never `clear`. `clear` asserts the
    # composer was seen and held nothing; an empty buffer asserts only that the
    # probe saw nothing at all, and reading the second as the first is how a
    # failed capture became a benign verdict (task #76). A probe that cannot
    # observe its subject does not get to report on it.
    if not pane.strip():
        return "unreadable"

    plain_lines = SGR.sub("", pane).splitlines()

    if QUEUED in "\n".join(plain_lines):
        return "queued"
    if is_busy(plain_lines):
        return "busy"

    # ⛔ THE LAST prompt-glyph line, not the first. Everything above it is
    # transcript, including echoes of commands that already ran. See the module
    # docstring: keying on the first match made the verdict depend on capture
    # depth and reported consumed echoes as live stranded deliveries.
    last = None
    for line in pane.splitlines():
        got = composer_content(line)
        if got is not None:
            last = got
    if last is None:
        return "clear"

    text, is_dim = last
    if not text:
        return "clear"
    if text.startswith("[Pasted Content"):
        return "paste"          # a paste is never a suggestion
    if is_dim:
        return "ghost"
    if text in SAFE_SLASH:
        return "slash:" + text
    return "other"


def main():
    # ⛔ FAIL CLOSED ON MISUSE. This script reads stdin only. Invoked with a path
    # argument it used to read an empty (or interactive) stdin and print `clear` —
    # a caller's typo became "nothing to repair" (task #76).
    if len(sys.argv) > 1:
        sys.stderr.write(
            "classify-pane-composer: reads a `tmux capture-pane -e -p` capture on "
            "STDIN and takes no arguments (got: %s)\n" % " ".join(sys.argv[1:]))
        print("unreadable")
        return 2
    if sys.stdin.isatty():
        sys.stderr.write(
            "classify-pane-composer: stdin is a terminal — pipe a capture in\n")
        print("unreadable")
        return 2
    verdict = classify(sys.stdin.read())
    print(verdict)
    return 2 if verdict == "unreadable" else 0


if __name__ == "__main__":
    sys.exit(main())
