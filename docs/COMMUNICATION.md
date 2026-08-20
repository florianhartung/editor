# Communication between Core and Frontend

The communication between the core application and the frontend is entirely message based. All types
must be serializable/deserializable by design.

Messages can be sent via two types of channels:
- Simple Channel
- RPC

## Implementation

As of now, `FrontendEvent` and `AppEvent` are the main two message types. `FrontendEvent`s are
emitted by the frontend, while `AppEvent`s are emitted by the core application.

These events are sent via async channels.
TODO: Why async? Why channels? Can this be abstracted away?

TODO: Rename event to message/msg?

## Bandwidth-Heavy Messages

While a message-based system might suffice for most applications, a text editor is especially
bandwith-heavy, considering that the frontend may have to re-render an entirely new text buffer per
frame.

It is obvious that this causes issues when the core application and the frontend reside on different
systems with bandwidth and latency limitations between them. It it yet to be seen how impactful this
is in a real world scenario.

For now, the focus is the scenario in which both parts share a common address space, allowing
special types like reference-counted types to be used, assuming they can be adequately
(de-)serialized (e.g. `Arc` or `Rope`).
