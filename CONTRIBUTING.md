# Contributing to Komodo PDF

Thanks for your interest in Komodo. Before you start, two things to read:

- **[SPEC.md](SPEC.md)** — the scope and the locked decisions. Komodo is deliberately
  **not** trying to be Adobe Acrobat (see [Non-Goals](SPEC.md#8-non-goals)). PRs that push
  scope back toward editing/forms/signatures/OCR-before-its-time will be declined, however
  good the code. If you want to propose a scope change, open an issue first.
- **This file** — how we license contributions (the DCO, below).

## License of contributions

Komodo is licensed under **GPLv3** (see [LICENSE](LICENSE)). By contributing, you agree
your contributions are licensed under the same terms.

## Developer Certificate of Origin (DCO) — required

We use the **Developer Certificate of Origin** instead of a CLA. It's a lightweight,
per-commit statement that you have the right to submit the code you're contributing. We
require it from the very first commit so the project's provenance is clean — there is no
retroactive way to add it later.

Every commit must be **signed off**. Add the sign-off automatically with:

```sh
git commit -s -m "your message"
```

This appends a line to your commit message:

```
Signed-off-by: Your Name <your.email@example.com>
```

By signing off, you certify the DCO below (verbatim from <https://developercertificate.org>):

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
1 Letterman Drive
Suite D4700
San Francisco, CA, 94129

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

The name and email in your `Signed-off-by` must be real and match your commit author
identity. Set them once with:

```sh
git config user.name  "Your Name"
git config user.email "your.email@example.com"
```

### Forgot to sign off?

Amend the last commit:

```sh
git commit --amend -s --no-edit
```

For multiple commits on a branch, rebase and sign off the range:

```sh
git rebase --signoff origin/main
```

PRs with unsigned commits will be blocked until every commit carries a valid sign-off.

## Pull request basics

- Branch from `main`; keep PRs focused (one logical change).
- Match the surrounding code's style and conventions.
- Include a clear description of *what* and *why*; link the relevant issue.
- If you touched behavior, say how you verified it.

## Reporting bugs / proposing features

Open an issue. For anything that changes scope, get agreement in the issue **before**
writing code — it saves everyone a wasted PR.

## Privacy promise (don't break it)

Komodo's whole reason to exist is that **the user's documents don't go to a Komodo
server — there is none.** Contributions must not add telemetry, analytics, "phone-home,"
or any upload of document content. Network calls go only to the AI provider the user
explicitly configured. When in doubt, open an issue.
