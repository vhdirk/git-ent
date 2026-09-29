# `sgit branch`

List or create branches across the top-level project and every
submodule.

```bash
# List branches in all repos
sgit branch

# Create a branch everywhere
sgit branch -c <branch-name>
```

| Flag             | Description                                 |
|------------------|---------------------------------------------|
| `-c`, `--create` | Name of the branch to create in every repo. |

Creating a branch that already exists is idempotent.
