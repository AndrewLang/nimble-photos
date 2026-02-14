# Coding Guidance

This project contains a Rust backend and an Angular frontend.
All changes must respect the following rules.

---

## Backend (Rust)

1. Follow an object-oriented design approach:
   - Use structs to model domain entities.
   - Encapsulate logic inside impl blocks.
   - Avoid putting business logic inside HTTP handlers.
   - Separate layers: api → services → domain → repository.

2. Testing requirements:
   - Every new feature must include unit tests.
   - Tests must be placed under the `tests/` folder.
   - All tests must pass (`cargo test`) before task completion.

3. Quality:
   - Code must compile without warnings.
   - Follow `cargo fmt` formatting.
   - Do not introduce unnecessary dependencies.

---

## Frontend (Angular)

1. Use latest Angular template syntax:
   - Use standalone components.
   - Prefer signals over RxJS where practical.
   - Use modern control flow syntax (`@if`, `@for`, etc.).

2. Use ViewModel pattern:
   - Components should not contain complex logic.
   - Create a ViewModel class or service to manage UI state.
   - Keep UI templates declarative and simple.

3. Quality:
   - Code must pass `ng build` and type checking.
   - Avoid business logic duplication from backend.
   - Do not hardcode API URLs.

---

## General Rules

- Do not modify unrelated files.
- Keep changes minimal and focused.
- Ensure the project builds successfully after modifications.
- Add comments only when necessary to clarify intent.