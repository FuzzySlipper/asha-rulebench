This frontend is built as layered infrastructure. Architecture is fixed unless the task explicitly says otherwise.

Use workspace generators for new components, libraries, features, stores, and tests.
Do not create new dependencies without planner approval.
Do not duplicate backend protocol types; use generated protocol exports only.
Do not import another library's internals; public API entrypoints only.
Do not bypass the transport layer for backend communication.
Do not bypass platform ports for browser/host APIs.
Do not bypass the store for application state mutation.
Do not put domain logic in components. Do not put feature logic in shell.
Expose async state as AsyncState<T>; map all failures to classified errors.
Do not close a user-deliverable UI task on deterministic evidence alone: run the live scenario, inspect the rendered artifacts yourself, and report what the UI did, including non-claims. A passing synthetic test is diagnostic, not proof.
Do not use any, non-null assertions, unsafe casts, or lint disables.
Do not add global CSS except through approved token/theme files.
Do not create circular dependencies or reverse dependency direction.
Prefer explicit, boring, typed code over clever abstractions.
When a task seems to require breaking a boundary, stop and request planner review.
