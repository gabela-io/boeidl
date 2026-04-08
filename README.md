# boeidl

Compilador DSL en Rust para los **diseños de registro** (formato BOE) de la AEAT.

Los modelos tributarios de la Agencia Tributaria española se presentan como ficheros de texto de posición fija con un diseño publicado en XLS/PDF. `boeidl` captura ese diseño en un fichero `.boe` legible y genera código Rust que **serializa**, **deserializa** y **valida** los registros.

> Estado: alpha. V1 implementa el **Modelo 130** (pago fraccionado IRPF).

## Qué hace

Dado un fichero `mod130.boe`:

```boe
model "130" version "2015-v11" {
    encoding = "ISO-8859-1"
    line_ending = "CRLF"
    record_length = 878
}

field nif {
    at = 13
    length = 9
    type = alphanumeric
    required = true
    description = "NIF del declarante."
}

field c01_ingresos {
    at = 77
    length = 13
    type = signed_amount
    decimals = 2
    description = "Casilla [01]: Ingresos computables."
}

# ... más campos ...

derive c03_rendimiento_neto = c01_ingresos - c02_gastos
derive c04_20pct = max(c03_rendimiento_neto, 0) * 20 / 100

check E301 {
    rule = c03_rendimiento_neto == c01_ingresos - c02_gastos
    severity = error
    message = "Rendimiento neto [03] debe ser [01] - [02]."
}
```

`boeidl compile` produce un módulo Rust con:

```rust
pub const MODEL_NUMBER: &str = "130";
pub const RECORD_LENGTH: usize = 878;

pub struct Mod130 {
    pub nif: String,
    pub c01_ingresos: i64,
    pub c02_gastos: i64,
    pub c03_rendimiento_neto: i64,
    // ...
}

impl Mod130 {
    pub fn marshal(&self) -> Result<Vec<u8>, AeatError> { /* ... */ }
    pub fn unmarshal(data: &[u8]) -> Result<Self, AeatError> { /* ... */ }
    pub fn compute_derived(&mut self) { /* ... */ }
    pub fn validate(&self) -> Vec<AeatDiagnostic> { /* ... */ }
}
```

Que se usa así:

```rust
let mut m = Mod130::default();
m.nif = "12345678Z".to_string();
m.c01_ingresos = 1_000_000; // 10 000,00 €
m.c02_gastos   =   300_000; //  3 000,00 €
m.compute_derived();        // calcula c03, c04, c07, c12, c14, c17, c19
assert!(m.validate().is_empty());

let bytes = m.marshal()?;   // 878 bytes + CRLF = 880 en ISO-8859-1
```

## CLI

```bash
# Compilar un .boe a Rust
boeidl compile models/mod130.boe --out crates/mi_app/src/generated/

# Validar sintaxis y semántica
boeidl check models/mod130.boe

# Inspeccionar el diseño (tabla legible)
boeidl inspect models/mod130.boe
```

`inspect` imprime algo así:

```
Modelo 130 (version 2015-v11)
Encoding: Iso8859_1
Record length: 878

#    Name                       At     Len    Type           Fixed     Usage
─────────────────────────────────────────────────────────────────────────────
1    modelo                     1      3      number         130       all
2    pagina                     4      2      number         01        print_only
6    nif                        13     9      alphanumeric             all
12   c01_ingresos               77     13     amount(2)                all
...
```

Muy útil para verificar contra el PDF oficial de la AEAT campo por campo.

## Sintaxis `.boe`

### Bloque `model`

```boe
model "<número>" version "<diseño>" {
    encoding      = "ISO-8859-1" | "ASCII"
    line_ending   = "CRLF" | "LF"
    record_length = <número>
}
```

### Bloque `field`

```boe
field <nombre> {
    at          = <posición_1_indexed>
    length      = <longitud>
    type        = alpha | alphanumeric | number | signed_amount
    decimals    = <n>                     # solo signed_amount
    required    = true | false
    fixed       = "<valor>"               # valor constante
    domain      = "A|B|C"                 # alternativas válidas
    usage       = all | print_only | internet_only
    description = "<texto>"
}
```

**Tipos y convenciones AEAT**:

| Tipo | Align | Padding | Alfabeto |
|---|---|---|---|
| `alpha` | izquierda | espacios | A-Z Ñ Ç |
| `alphanumeric` | izquierda | espacios | A-Z Ñ Ç 0-9 |
| `number` | derecha | ceros | 0-9 |
| `signed_amount` | — | — | `' '`/`N` + dígitos (decimales sin separador) |

El texto se sanea automáticamente en `marshal`: mayúsculas, sin tildes, preservando Ñ y Ç. Los importes con signo codifican el signo en la primera posición (` ` positivo, `N` negativo).

### Bloques `derive` y `check`

```boe
derive <campo> = <expresión>

check <código> {
    rule     = <expresión_bool>
    severity = error | warning
    message  = "<texto>"
}
```

Operadores: `+ - * /`, `max(a, b)`, `min(a, b)`, `== != > < >= <=`, `implies`.

Las derivaciones referencian solo campos declarados antes del objetivo (se valida en `boeidl check`). Si un campo tiene `fixed`, las derivaciones sobre él se ignoran — `fixed` siempre gana.

## Integración en tu proyecto

Como artefacto de build (`build.rs`):

```rust
// build.rs
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=models/mod130.boe");
    let status = Command::new("boeidl")
        .args(["compile", "models/mod130.boe", "--out", "src/generated/"])
        .status()
        .expect("boeidl debe estar instalado: cargo install boeidl");
    assert!(status.success());
}
```

Y en `src/generated/mod.rs`:

```rust
#[allow(unused, clippy::all, unused_parens, dead_code, non_snake_case)]
mod mod130 { include!("mod130.rs"); }
pub use mod130::*;
```

Alternativamente, si `boeidl` es una dependencia directa (misma workspace), puedes llamar a la API de librería desde `build.rs`. Ver `example/build.rs` en este repo.

> El directorio `src/generated/` debería ir en `.gitignore`: es un artefacto de build.

## Estructura del repo

```
boeidl/
├── crates/boeidl/              # el compilador (lib + bin)
│   ├── src/
│   │   ├── grammar.pest        # gramática PEG del DSL
│   │   ├── ast.rs              # tipos del AST
│   │   ├── parser.rs           # pest → AST
│   │   ├── validator.rs        # validación semántica
│   │   ├── codegen/rust.rs     # AST → Rust generado
│   │   ├── runtime/            # helpers para el código generado
│   │   └── main.rs             # CLI (clap)
│   └── tests/                  # parser + validator + codegen
├── models/
│   └── mod130.boe              # Modelo 130 completo
├── example/                    # crate demo que consume boeidl
│   ├── build.rs                # genera mod130.rs en $OUT_DIR
│   └── tests/roundtrip.rs      # marshal + unmarshal end-to-end
└── testdata/
    └── mod130_expected.rs      # golden snapshot del código generado
```

## Desarrollo

```bash
nix develop               # shell con la toolchain fija
cargo test                # 51 tests: parser, validator, codegen, runtime, round-trip
cargo run -p boeidl-example   # imprime un registro BOE completo por stdout
```

Refrescar el golden snapshot tras cambios intencionales en el codegen:

```bash
UPDATE_GOLDEN=1 cargo test -p boeidl --test codegen_130
```

## Convenciones AEAT — formato BOE

- **Nombre oficial**: "formato BOE" es como la AEAT llama al fichero exportable/importable del formulario web. La extensión es `.<modelo>` (ej: `.130`).
- **Encoding**: ISO-8859-1 (Latin-1). Ñ = `0xD1`, Ç = `0xC7`.
- **Line ending**: CRLF (`\r\n`).
- **Longitud fija**: 878 bytes por registro para el Modelo 130, + 2 bytes de CRLF = 880.
- **Texto**: mayúsculas, sin vocales acentuadas, solo caracteres ASCII del rango ISO-8859-1 (más Ñ y Ç).
- **Importes**: en céntimos, sin separadores; el signo va en la primera posición (` ` positivo, `N` negativo).
- **Campos `MI`** (solo impresión) y **`PI`** (solo presentación telemática): expresados como `usage = print_only` / `usage = internet_only`.

Fuente oficial del diseño de registro:
<https://sede.agenciatributaria.gob.es/Sede/ayuda/disenos-registro.html>

## Modelos soportados

| Modelo | Descripción | Estado |
|---|---|---|
| **130** | Pago fraccionado IRPF (DR130e15v11) | ✅ implementado |

## Licencia

MIT

## OpenHacienda

`boeidl` forma parte de [OpenHacienda](https://github.com/OpenHacienda), herramientas open-source para interactuar con la administración pública española.
