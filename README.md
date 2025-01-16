# Dry Doc

Keep your doctests DRY with `drydoc`.

## What

Store your doctest code on disk and selectively show or hide it. This allows you to iteratively build up tutorials that have boilerplate code you don't want to show again and again:

    First add:

    ```
    #![doc = doc_show!("docs/one.rs")]
    ```

    Then add:

    ```
    #![doc = doc_hide!("docs/one.rs")]
    #![doc = doc_show!("docs/two.rs")]
    ```

    Which lets you:

    ```
    #![doc = doc_hide!("docs/one.rs")]
    #![doc = doc_hide!("docs/two.rs")]
    #![doc = doc_show!("docs/three.rs")]
    ```

As a bonus, if you store your doctest code in a `.rs` file then your editor will auto-format it for you.

## Install

```term
$ cargo add drydoc
```

## Known limitations

- This macro is not recursive. If you load a file from disk, it cannot load or compose additional files. This could be a future feature.
- There's no templating support (but there could be, wink). Would it be cool if you could ust JINJA to write tests? We think so.
