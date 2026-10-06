# `git-nest clone`

Clone a repository recursively (including all submodules).

```bash
git-nest clone <url> [dest]
```

| Argument | Description                                                   |
|----------|---------------------------------------------------------------|
| `url`    | Repository URL to clone.                                      |
| `dest`   | Destination directory (optional, inferred from the URL).      |

Equivalent to `git clone --recurse-submodules`.
