# Security Policy

## Reporting a vulnerability

Please do not disclose suspected vulnerabilities in a public issue, discussion, or pull request. Use GitHub's private vulnerability reporting feature for `Rubentxu/bevy-2d-editor` when available. If private reporting is unavailable, contact the repository owner through their GitHub profile and request a private channel before sharing technical details.

Include the affected version or commit, environment, reproduction steps, expected impact, and any suggested mitigation. Maintainers should acknowledge a complete report within seven days, provide status updates as investigation proceeds, and coordinate disclosure after a fix is available.

## Supported versions

| Version | Supported |
| --- | --- |
| Latest tagged release | Yes |
| `main` | Best effort; development code |
| Older tagged releases | No; upgrade to the latest release |

Security fixes are applied to the latest release line. Backports are not currently guaranteed.

## Browser storage and OPFS

Bevy 2D Editor stores project metadata and assets in the browser Origin Private File System (OPFS).

- Storage is isolated by origin, browser profile, and browser implementation. A project at one host or port is not automatically available at another.
- OPFS is local browser storage, not encrypted project backup or access control. Anyone who can use the same unlocked operating-system/browser profile may be able to access editor data.
- Clearing site data, private browsing cleanup, browser-profile deletion, or storage eviction can remove projects. Export and back up important projects separately.
- Do not store secrets, API keys, credentials, or confidential tokens in Scene, asset, schema, source, or project files.
- Hosting deployments should use HTTPS and a stable single origin. Changing origins creates a separate OPFS namespace.
- The optional AI proxy should receive provider credentials through server environment variables, never through browser project data or committed frontend code.

Cross-origin storage isolation bugs, unintended project-data exposure, unsafe generated output, and AI proxy authentication or credential leaks should be reported as vulnerabilities.
