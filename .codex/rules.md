# Project Rules

## General
- **No Comments**: Do not generate comments (single-line or multi-line) in the code unless they are technically required for the language or framework to function.
- **Clean Code**: Code should be self-documenting through clear naming of variables, functions, and classes.
- **Dry Code**: Aim for reusability and avoid duplication.
- **Name Rule**: Use complete words for variable names, functions, and classes. Avoid abbreviations unless they are widely known.
- **No Magic Numbers**: Do not use magic numbers in the code. Use constants instead.
- **No Magic Strings**: Do not use magic strings in the code. Use constants instead.

## Frontend (Angular)
- **Modern DI**: Use the `inject()` function for Dependency Injection instead of constructor injection.
- **Signals First**: Use Angular Signals for state management and reactivity. Only use Observables for async streams like HTTP requests.
- **Strict Typing**: No `any` type allowed. Every response and variable must have a defined interface or type.
- **Lazy Loading**: Use `loadComponent` in routes for all new components.
- **Image Performance**: Use `loading="lazy"` for all library images to maintain performance.
- **Naming Conventions**:
    - Use `camelCase` for properties and variables.
    - Use `.` as a separator for file names (e.g., `album.detail.component.ts`).

## Backend (Rust)
- **Testing**: All new logic or modified backend code must include corresponding unit tests.
- **DTO/Entity Separation**: Never expose database entities directly to the frontend. Use DTOs with `#[serde(rename_all = "camelCase")]`.
- **SQL Security**: All SQL queries must use parameterized bindings (`$1`, `$2`). Never use string concatenation for queries.
- **Consistent Errors**: Map errors to a consistent JSON structure for the frontend to handle.
