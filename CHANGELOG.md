# Unreleased

- Add: Support for `include`-ing one template from within another. Any file from the cargo root can be imported. (https://github.com/schneems/drydoc/pull/2)
- Add: A custom filter `dochide` this can be used within a jinja template to prepend every line with a rustdoc hide directive `# `. This can be used in combination with the ability to `include` another template to compose files. (https://github.com/schneems/drydoc/pull/2)

## 0.1.0

- First
