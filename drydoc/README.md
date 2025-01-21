#  DRYdoc

Keep your doctests DRY with `drydoc`. Write example code templates once and reuse them (everywhere).

## Install

```term
$ cargo add drydoc
```

## Use

Store code used in your examples as templates on disk for easy reuse. Easily hide boilerplate:

    ///```
    #![doc = drydoc!(path = "docs/one.rs", hidden = true)]
    #![doc = drydoc!(path = "docs/two.rs")]
    ///```

Go beyond static templates with the `jinja` feature that supports inline TOML table config:

    ///```
    #[doc = drydoc!(path = "docs/person.rs", toml = { name = "Schneems", dog = "Cinco" })]
    ///```

## Doc macro FYI

- Document modules with `#![doc = <macro>]` (hash+bang)
- Document everything else with `#[doc = <macro>]` (just a hash, no bang)

Also, if you just need plain-old static content, you can use `include_str!` (with a relative path instead of from crate root) and skip this library entirely.

## Jinja support

This library uses [minijinja](https://docs.rs/minijinja/2.6.0/minijinja/) for JINJA templating. For examples of template syntax, see [Template Designer Documentation](https://jinja.palletsprojects.com/en/stable/templates/).

The basics are: Use curly brackets for dynamic content, for example:

    // <CARGO_MAINFEST_DIR>/docs/person.rs
    #[derive(Debug)]
    struct Person {
        name: String,
    }

    impl Person {
        fn new() -> Self {
            Self {
                name: "{{name | default('Richard')}}".to_string(),
            }
        }
    }

In this example `{{name | default('Richard')}}` is a JINJA template placeholder. We're instructing JINJA to either want a `name` that we provided or fallback to `'Richard'`. With this template in place, you can call it with inline toml that defines a name:

    ///```
    #[doc = drydoc!(path = "docs/person.rs", toml = { name = "Schneems" })]
    ///```

Or you can call it without any values and let it fall back on the default:

    ///```
    #[doc = drydoc!(path = "docs/person.rs")]
    ///```

This example is simple, yet changing the contents of a string isn't terribly useful. You could use it for something more aggressive, such as dynamically changing the return type of a function, for example:

    // <Cargo.toml dir>/docs/winnow_repeat.rs
    fn parser(s: &str) -> {{result_type}}<&str, Vec<&str>> {
        repeat(0.., "ABC").parse_peek(s)
    }

Then, depending on where the example is used, you could toggle different types:

    ///```
    #[doc = drydoc!(path = "docs/winnow_repeat.rs", toml = { result_type = "winnow::IResult" })]
    #[doc = drydoc!(path = "docs/winnow_repeat.rs", toml = { result_type = "winnow::PResult" })]
    ///```

You can use control and flow operators like [for](https://jinja.palletsprojects.com/en/stable/templates/#for) and [if](https://jinja.palletsprojects.com/en/stable/templates/#if). You're only limited by your imagination (and the Rust compiler and JINJA syntax).

## Advanced JINJA: Extends, Include and Import

You can include a jinja template from another by using a full path from the crate root and the `include` jinja keyword:

```jinja
{% include "docs/header.rs" %}
```

If you want to include a template but hide it from the end user, you can use the `dochide` filter that is provided by drydoc:

```jinja
{% filter dochide %}
{% include "docs/header.rs" %}
{% endfilter %}
```

Every file in the crate is available as a template to import by default. For more info on `includes` and related functionality, see [template inheritance](https://jinja.palletsprojects.com/en/stable/templates/#template-inheritance).

## Rust-analyzer support

If you're storing your examples with a `.rs` extension, your editor will format them correctly on save; however, rust-analyzer and friends don't understand JINJA syntax and might accidentally think it's Rust code. One trick to forcing it to not split one line into three is to clamp it by adding rust comments, you can hide these from view by prepending the doc hide `# ` character to the line. For example:

    # // Load bearing comment: Force rust-analyzer to keep vvvv as one line
    fn parser(s: &str) -> {{result_type}}<&str, Vec<&str>> {
    # // Load bearing comment: Force rust-analyzer to keep ^^^^ as one line
        repeat(0.., "ABC").parse_peek(s)
    }

## Escape from JINJA:

If you happen to write Rust code that minijinja thinks is jinja syntax you can escape it [see the Escaping section](https://jinja.palletsprojects.com/en/stable/templates/#escaping). If you have a whole template that you don't want to be evaluated by JINJA, you can wrap it in raw statements like:

    ## {% raw %}
    // ...
    ## {% raw %}

The `{% raw %}` tells Jinja, "Everything between these tags is NOT a JINJA template"; the hash in front `# ` hides these implementation details from our readers (so they won't show up in the output docs).
