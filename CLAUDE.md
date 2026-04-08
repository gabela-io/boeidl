# CLAUDE.md

## Proyecto
boeidl — compilador DSL para formatos de fichero posicional de la AEAT (Hacienda española).

## Stack
- Rust (edition 2021, MSRV 1.75)
- pest para parsing PEG
- clap para CLI
- Nix flake con numtide/blueprint

## Estructura
- `crates/boeidl/` — el compilador (parser, AST, codegen, CLI)
- `models/` — ficheros .boe con definiciones de modelos AEAT
- `example/` — crate ejemplo que usa boeidl como build artifact
- `testdata/` — golden files para tests

## Convenciones
- Código y comentarios en inglés, documentación del DSL y mensajes de error en español
- Los ficheros .boe usan nombres de campo en español (c01_ingresos, nif, ejercicio)
- Tests: `cargo test` en la raíz del workspace
- Formateo: `cargo fmt`, linting: `cargo clippy`
- No usar unwrap() en código de librería, solo en tests y examples

## Workflow
1. Editar grammar.pest y ast.rs juntos — la gramática y el AST deben estar sincronizados
2. Después de cambiar la gramática, correr `cargo test -p boeidl` para verificar
3. El codegen genera Rust idiomático con derives de Debug, Clone, PartialEq
4. Los golden tests se actualizan con `UPDATE_GOLDEN=1 cargo test`

## Contexto AEAT — Formato BOE
- Los diseños de registro de la AEAT definen ficheros de texto de posición fija
- La AEAT llama "formato BOE" al fichero exportable/importable del formulario web
- Encoding: ISO-8859-1 (Latin-1), no UTF-8
- Line ending: CRLF (\r\n)
- Caracteres alfanuméricos: A-Z, 0-9, espacio, Ñ y Ç. Mayúsculas, sin tildes.
- Importes con signo: primera posición = espacio (positivo) o N (negativo)
- Importes con decimales: las últimas N posiciones son decimales, sin separador
- Campos marcados MI = solo impresión, PI = solo presentación internet
- Fuente oficial: https://sede.agenciatributaria.gob.es/Sede/ayuda/disenos-registro.html
