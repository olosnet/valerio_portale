# Module: templates (src/templates/)

## Purpose

Provides Minijinja-based template rendering with a filesystem loader.

Requires the `templates` feature.

## ADDED Requirements

### Requirement: Template rendering

`TemplatesService` SHALL create a Minijinja environment with a path loader for the
configured templates directory. `render()` SHALL compile and render a named template
with the given context variables, returning the rendered string or a `CornettiError`
(500) on failure.

See `TemplatesService` in `src/templates/services.rs`.

#### Scenario: Successful render
- WHEN `render` is called with a valid template name and context
- THEN the rendered string SHALL be returned

#### Scenario: Template not found
- WHEN `render` is called with a non-existent template name
- THEN a 500 `CornettiError` SHALL be returned

#### Scenario: Minijinja error conversion
- WHEN a `minijinja::Error` is converted to `CornettiError`
- THEN it SHALL produce a 500 error via `errors::templates::template_rendering_error()`
- AND the original error SHALL be stored in `internal_detail`

### Requirement: Configuration

`TemplatesConf::from_env()` SHALL read `TEMPLATES_DIRECTORY`, defaulting to
`./templates`.

See `src/templates/confs.rs`.
