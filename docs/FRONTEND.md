# Frontend structure

## Buffer View

The buffer view is a list of lines providing view into a buffer to the user.
Every buffer has a cursor, and this cursor may be visible in the current buffer
view.

- **Monospace**: It is assumed, that all characters use the same width (with some
exceptions, see below).

> [!WARNING]
> The exact widths of characters are not yet specificed explicitly. Currently,
  the `unicode-width` crate is used in the app and a monospace font is assumed
  to be used by the frontend in the hope that all lines align properly.

## Focus System

> [!WARNING]
> Not yet implemented. Currently only the buffer view exists as a focusable
  element.
