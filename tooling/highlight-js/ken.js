/*
Language: Ken
Author: Ken federation tooling (tooling/highlight-js)
Description: Ken is a dependently-typed language with a literate `.ken.md`
  surface form (fenced ```ken code blocks); `.ken` files are the plain-source
  form. Registering this definition under the `ken` alias makes highlight.js
  auto-select it for a ```ken fence's info string.
Website: (repo-internal; no public website yet)
Category: functional

GROUNDING (`tooling/README.md`'s discipline): every token category here is a
projection of `crates/ken-elaborator/src/lexer.rs`, the single source of
truth for what Ken actually lexes. When the lexer's token set moves (a
keyword added/retired, a new literal suffix, a new operator spelling), re-grep
the lexer's keyword match arms, its single-char Unicode-alias arms, and its
`lex_numeric` suffix checks, and update the tables below in the same commit.
This file is intentionally coarser than the lexer in one place only: the
lexer *rejects* a lone `<`, `>`, or `/` (they only lex as the first half of
`<=`/`<:`, `>=`/`><`, `/=`/`/\`); this grammar simply never matches them
standalone rather than inventing acceptance the lexer doesn't have.

This is a highlighter, not a checker: it categorises tokens for the eye. It
does not track scope (a shadowed local looks like any other identifier), does
not validate refinement predicates, and does not know which names are bound
vs. free -- all of that requires the real elaborator.
*/
(function (root, factory) {
  if (typeof module === 'object' && module.exports) {
    // CommonJS / Node, e.g. `const ken = require('./ken.js')`.
    module.exports = factory();
  } else if (typeof define === 'function' && define.amd) {
    // AMD loaders.
    define([], factory);
  } else {
    // Plain <script> tag: exposes `window.hljsDefineKen`.
    root.hljsDefineKen = factory();
  }
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  // The highlight.js "language definer": `(hljs) => languageObject`.
  function ken(hljs) {
    // -- Keywords -------------------------------------------------------
    // The COMPLETE reserved-word set, verbatim from the lexer's identifier
    // match arms (lexer.rs, the `match s.as_str() { ... }` block). `view`
    // was retired and is NOT here -- it lexes as an ordinary identifier
    // today. `Type` (capital T) is the keyword; `Type0` is an ordinary
    // uppercase-initial ConId (see CONID below), not a keyword.
    const KEYWORDS = {
      // Ident-continue includes ASCII alnum, `_`, and the prime `'`
      // (e.g. `x'`, `map'`), so widen hljs's default keyword-word pattern
      // to match; without this, a keyword immediately followed by `'`
      // (unusual, but not disallowed) would still be found correctly since
      // primes never appear inside a keyword itself -- this is here mainly
      // so identifier scanning and keyword scanning agree on word shape.
      $pattern: /[A-Za-z_][A-Za-z0-9_']*/,
      keyword: [
        'const', 'fn', 'proc', 'let', 'in', 'Type',
        'requires', 'ensures', 'prove', 'law', 'old', 'space',
        'data', 'match', 'def', 'type',
        'foreign',
        'temporal',
        'class', 'instance', 'derive', 'where',
        'module', 'import', 'use', 'pub',
        'prop', 'lemma', 'proof'
      ]
    };

    // -- Comments ---------------------------------------------------------
    // `--` to end of line ONLY -- the lexer has no block-comment syntax at
    // all (no `{- -}`, no `/* */`); do not add one here.
    const COMMENT = hljs.COMMENT('--', '$', { relevance: 0 });

    // -- Strings ------------------------------------------------------------
    // Double-quoted, single-line: the lexer has NO escape handling (a `"`
    // always closes the string) and a literal newline before the closing
    // quote is a lex error ("unterminated string literal"). Ending the mode
    // at a newline too mirrors that failure mode instead of running the
    // string scope past it.
    const STRING = {
      className: 'string',
      begin: '"',
      end: /["\n]/,
      relevance: 0
    };

    // -- Numeric literals -----------------------------------------------
    // Four forms (`lex_numeric`): integer, float (needs a literal `.`;
    // an exponent is only recognised once a dot has already been read --
    // `1e-9` with no dot does NOT lex as one float token, only `1.0e-9`
    // does), a `d`-suffixed decimal (`0.1d`, or a bare `5d`), and an
    // `f32`-suffixed float32 (`1.5f32`, or a bare `5f32`). Both suffixes
    // are valid with or without a fractional part, and are only consumed
    // when no ident-continue character follows (checked here with a
    // trailing `\b`, an ASCII-only approximation of the lexer's exact
    // ident-continue set -- it does not special-case a literal `'`
    // immediately after a suffix, a pathological case no real Ken code
    // writes). Order matters: the suffixed forms must be tried before the
    // bare float/int forms so `0.1d` isn't mis-split into `0.1` + `d`.
    const NUMBER = {
      className: 'number',
      begin:
        /\b\d+(?:\.\d+)?d\b|\b\d+(?:\.\d+)?f32\b|\b\d+\.\d+(?:[eE][+-]?\d+)?|\b\d+/,
      relevance: 0
    };

    // -- Surface sugar, not a lexer keyword -------------------------------
    // `absurd` is checked-mode surface sugar for Omega-classified `Bottom`
    // elimination (`crates/ken-elaborator/src/elab.rs`, keyed on the bare
    // identifier at resolution time) -- it is a plain `Ident` token to the
    // lexer, not a reserved word. Kept as its own tiny, clearly-commented
    // `built_in` entry (not folded into KEYWORDS) so it doesn't misrepresent
    // the lexer's keyword list; extend this list only for genuine surface
    // sugar of the same kind, never for ordinary prelude names (those are
    // covered for free by the uppercase ConId rule below).
    const BUILT_IN = {
      className: 'built_in',
      begin: /\babsurd\b/,
      relevance: 0
    };

    // -- ConId (uppercase-initial) -> type/title.class --------------------
    // The lexer's own rule: an identifier is a `ConId` (type or data
    // constructor) iff its first character is uppercase; everything else
    // lowercase-initial is an ordinary term variable. This single rule
    // covers every built-in and user type name (`Bool`, `Nat`, `List`,
    // `Omega`, `Bottom`, `Refl`, a user's own `Shape`, ...) with no
    // hand-maintained name list to rot. `Ω`/`Σ`/`Π` are single-codepoint
    // Unicode aliases the lexer folds directly to the ConIds
    // `Omega`/`Sigma`/`Pi`, so they belong in this same class. `Type` alone
    // is excluded (via the negative lookahead) because it is the reserved
    // keyword, not a ConId -- `Type0`, `TypeFoo`, etc. are still ConIds.
    const CONID = {
      className: 'title.class',
      begin: /\b(?!Type\b)[A-Z][A-Za-z0-9_']*\b|Ω|Σ|Π/,
      relevance: 0
    };

    // -- Operators / punctuation ------------------------------------------
    // Every non-bracket, non-string, non-comment token the lexer produces,
    // ASCII spelling and Unicode alias side by side, longest/most-specific
    // alternative first so e.g. `===` wins over `==` wins over `=`, and
    // `+%` wins over bare `+`. Two 2-character sequences share reversed
    // characters and must both be listed explicitly: `\/` (backslash then
    // slash) is `Or`, `/\` (slash then backslash) is `And` -- neither is a
    // prefix of the other, but each is a prefix-adjacent trap for the bare
    // `\` (Lambda) and (absent, see below) bare `/` alternatives, so the
    // bare backslash alternative is listed AFTER them.
    //
    // Deliberately absent: a lone `<`, `>`, or `/` are not valid Ken tokens
    // at all (the lexer errors unless they're the first half of `<=`/`<:`,
    // `>=`/`><`, or `/=`/`/\`) -- this grammar does not invent acceptance
    // for them; such a character in real source is a lex error and is left
    // unstyled here rather than falsely categorised.
    const OPERATOR = {
      className: 'operator',
      begin:
        /->|=>|===|==|::|<=|<:|>=|><|\/=|\\\/|\/\\|\+%|=|:|\||;|\.|,|\\|\+|-|\*|[→⇒≡≤≥≠⊑⊔⊓×λ∧∨∈]/,
      relevance: 0
    };

    return {
      name: 'Ken',
      // `ken` plus Ken's literate `.ken.md` fence-role tags. A Markdown
      // renderer that passes the *whole* fence info string to highlight.js
      // (e.g. marked with a custom renderer) sees `ken example`, not `ken`,
      // and highlight.js resolves a block by exact registered name/alias --
      // so the role-suffixed tags must be aliases to highlight as Ken.
      aliases: ['ken', 'ken example', 'ken reject', 'ken ignore'],
      case_insensitive: false,
      keywords: KEYWORDS,
      contains: [
        COMMENT,
        STRING,
        NUMBER,
        BUILT_IN,
        CONID,
        OPERATOR
      ]
    };
  }

  return ken;
});
