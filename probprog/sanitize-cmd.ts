// sanitize-cmd.ts — normalize a recorded command line before it lands
// in a committed receipts file.
//
// A receipt's `cmd` field documents the ARGUMENT SHAPE of the measured
// invocation (tool, flags, workload sizes); the reproduction path is
// the make target, so machine-local path prefixes carry no information
// and do not ship. Rules, applied in order:
//
//   1. caller-declared prefixes (`--subst NAME=/abs/prefix`, repeatable)
//      rewrite to `$NAME` wherever they occur at a path boundary —
//      longest prefix first, so nested prefixes resolve to the most
//      specific name;
//   2. the working directory rewrites to `.` (a command recorded from
//      the repo root reads as runnable from the repo root);
//   3. any absolute path still standing collapses to its final
//      component (`/dev/...` is exempt: shell redirections such as
//      `2>/dev/null` keep their meaning).

export type Subst = readonly [name: string, prefix: string];

/// Parse one `NAME=/abs/prefix` argument into a substitution pair.
export function parseSubst(v: string): Subst | null {
  const eq = v.indexOf("=");
  if (eq <= 0) return null;
  const name = v.slice(0, eq);
  const prefix = v.slice(eq + 1);
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name) || !prefix.startsWith("/")) {
    return null;
  }
  return [name, prefix];
}

export function sanitizeCmd(
  raw: string,
  cwd: string,
  substs: readonly Subst[],
): string {
  const rules: Subst[] = [
    ...substs.map(([n, p]): Subst => [`$${n}`, trimSlash(p)]),
    [".", trimSlash(cwd)],
  ]
    .filter(([, p]) => p.startsWith("/") && p.length > 1)
    .sort((a, b) => b[1].length - a[1].length);
  let out = raw;
  for (const [name, prefix] of rules) {
    const esc = prefix.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    // Path boundary: the prefix ends the token, or a path separator,
    // whitespace, or quote follows — so a sibling directory whose name
    // merely extends the prefix is never rewritten.
    out = out.replace(new RegExp(`${esc}(?=/|["'\\s]|$)`, "g"), () => name);
  }
  // Fallback guarantee: no absolute path survives. Not preceded by a
  // word character, `$`, or `.` (so `$NAME/sub/path` and `./rel/path`
  // stay intact); two or more components (so lone `/` operators in
  // shell text are untouched).
  return out.replace(/(?<![\w$.])(?:\/[\w.+-]+){2,}/g, (m) =>
    m.startsWith("/dev/") ? m : m.slice(m.lastIndexOf("/") + 1),
  );
}

function trimSlash(p: string): string {
  return p.length > 1 ? p.replace(/\/+$/, "") : p;
}
