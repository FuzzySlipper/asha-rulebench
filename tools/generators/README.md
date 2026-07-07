# Workspace Generators

Generator stubs live here so new projects have an obvious place to encode the architecture rules.

Expected generators:

- `lib`: one layer lib with tags and public barrel
- `feature`: vertical feature lib plus live scenario stub
- `component`: presentational component in `components`
- `store`: signals store with `AsyncState<T>`
- `platform-port`: interface, browser adapter, and fake
- `live-scenario`: opt-in Playwright live scenario using the artifact collector
