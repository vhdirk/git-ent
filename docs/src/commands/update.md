# `sgit update`

Initialize and update all submodules recursively.

```bash
sgit update
```

Equivalent to `git submodule update --init --recursive`. Useful after
a plain `git clone` or when submodule pointers have been updated by
a pull/merge.
