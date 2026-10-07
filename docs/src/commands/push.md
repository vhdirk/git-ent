# `git-ent push`

Push to remote across all repos that have commits to push.

```bash
git-ent push

# With push options (e.g. GitLab MR controls)
git-ent push -o merge_request.create -o merge_request.squash
```

You can define a `pushmr` convenience alias in config:

```toml
[alias]
pushmr = "push -o merge_request.create -o merge_request.remove_source_branch -o merge_request.squash"
```

Then run:

```bash
git-ent pushmr
```

| Flag                   | Description                                                  |
|------------------------|--------------------------------------------------------------|
| `-o`, `--push-option`  | A push option forwarded to `git push -o`. Repeatable.        |

Iterates depth-first. Repos with no remote, a detached HEAD, or no
unpushed commits are silently skipped. If no upstream is set, one is
created automatically.

Recommended GitLab MR push options:

- `merge_request.create`
- `merge_request.remove_source_branch`
- `merge_request.squash`
