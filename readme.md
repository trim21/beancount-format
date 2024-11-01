format beancount

# install

```shell
pip install beancount-format

beancount-format ./beans/
```

as pre-commit hooks

```yaml
repos:
  - repo: https://github.com/trim21/beancount-format
    rev: ...
    hooks:
      - id: beancount-format
```