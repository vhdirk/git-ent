# `sgit checkout`

Legacy counterpart to [`sgit switch`](./switch.md). Supports the two
main `git checkout` workflows recursively:

- Switching branches across the whole tree.
- Restoring individual files from a specific branch.

```bash
# Switch to an existing branch everywhere
sgit checkout <branch>

# Create a new branch and check it out everywhere
sgit checkout -b <branch>

# Restore specific files from a branch (routed to the owning repo)
sgit checkout <branch> -- <file>...
```

| Flag            | Description                                                          |
|-----------------|----------------------------------------------------------------------|
| `-b`            | Create the branch before checking it out.                            |
| `--`, `<files>` | Restore the listed files from `<branch>` instead of switching heads. |

For new projects prefer `sgit switch`, which provides a cleaner
interface (`-c`/`-C`/`--detach`) and matches modern `git` conventions.

Repos where the target branch does not exist emit a warning; processing
continues with the remaining repos.
