# Rust config azure environment
## Description
Environment variables are allowed to have square brackets in them. They are used by the [config](https://crates.io/crates/config) crate to pass arrays.
However, for some reason azure decided that they won't allow square brackets in their environment variables.

This crate enables us to pass arrays on azure without using square brackets by replacing digit keys as an array.

Example:
`__` is our separator
Before: `oauth_allowed_scopes__0=email`
After: `oauth_allowed_scopes[0]=email`
