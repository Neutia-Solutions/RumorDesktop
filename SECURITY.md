# Security reporting

Do not report vulnerabilities or exposed credentials through public issues, PRs, logs or screenshots.

Once this repository is hosted and private vulnerability reporting is enabled, use **Security → Advisories → Report a vulnerability**. Include the affected public commit/release, impact and a minimal synthetic reproduction. Do not submit browser profiles, tokens, private adapter code or real reading history.

If that reporting button is unavailable, ask the maintainer for a private reporting channel without disclosing the vulnerability. No security email address or response-time guarantee is configured yet. The hosting setup must enable/test private reporting before soliciting security reports.

Adapter implementation vulnerabilities belong in that adapter's private repository, visible only to its collaborators. Coordinate shared-protocol fixes privately, then publish a sanitized public explanation after the fix. Never assume that a GitHub Actions artifact, PR attachment or deleted issue comment is private.

Only the current development branch is maintained at present; there are no released supported versions. The private adapter repositories have separate access and release policies.
