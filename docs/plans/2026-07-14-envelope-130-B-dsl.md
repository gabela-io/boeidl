# Modelo 130 "con sobre" — extensión del DSL (Opción B) — Plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Que un `.boe` exprese el **fichero AEAT completo con sobre** (`<T130…><AUX>…</AUX>` + página + `</T1300…>`) y que `boeidl` genere Rust que lo serialice/deserialice byte-idéntico al fichero real del Modelo 130 (946 bytes).

**Architecture:** Extensión opt-in del DSL sobre la base multi-`record` que introdujo `feat/modelo303`. Se añade un bloque `envelope { param … header = <tmpl>  trailer = <tmpl>  contains = [rec, …] }`. `header`/`trailer` son **plantillas** de texto (literal + `${ident}`) que interpolan `param`s de nivel fichero (ejercicio, periodo). `contains` lista los `record` que forman el cuerpo, concatenados en orden. El sobre es **todo longitud fija**, así que `unmarshal` sigue siendo por offset: verifica los literales de las plantillas y trocea el cuerpo por `record_length`. Un fichero sin `envelope` se comporta exactamente como hoy (backward-compat con `mod130.boe` plano y con el 303).

**Tech Stack:** Rust edition 2021, pest (grammar PEG + parser pest→AST a mano), validador semántico, codegen Rust por plantillas de string. Sin dependencias nuevas.

**Fuente de verdad del layout:** `~/Dev/aldoborrero/autonomo/apps/sheets/src/lib/modelo130.ts` + `test/modelo130.test.ts` (byte-exact validados contra ficheros reales de la AEAT). Cifras reales del golden: 1T 2026, NIF `09030055W`, `BORRERO GONZALEZ` / `ALDO ISMAEL`, `[01]=58809.19`, `[02]=2407.15`, `[03]=56402.04`, `[04]=[07]=[19]=11280.41`.

---

## Layout objetivo (946 bytes)

```
┌─ header (plantilla, 17 bytes) ────────────────────────────────────────────┐
│ <T1300 ${ejercicio}(4) ${periodo}(2) 0000>                                 │
├─ record "aux" (311 bytes) ────────────────────────────────────────────────┤
│  1-5   "<AUX>"          6-75  blancos(70)                                  │
│ 76-79  version_programa 80-83 blancos(4)   84-92 nif_ed(9)                 │
│ 93-305 blancos(213)     306-311 "</AUX>"                                   │
├─ record "pagina" (600 bytes) ─────────────────────────────────────────────┤
│  1-11  "<T13001000>"    12 indicador compl.(esp)  13 tipo decl.(B|G|I|N|U) │
│ 14-22  nif(9)  23-82 apellidos(60)  83-102 nombre(20)                      │
│ 103-106 ejercicio  107-108 periodo                                        │
│ [01]109 [02]126 [03]143§ [04]160 [05]177 [06]194 [07]211§                  │
│ [08]228 [09]245 [10]262 [11]279§ [12]296 [13]313 [14]330§                  │
│ [15]347 [16]364 [17]381§ [18]398 [19]415§   (cada importe = 17 chars)      │
│ 432 marca compl.("X"/esp)  433-445 justificante(13)  446-479 IBAN(34)      │
│ 480-575 reservado(96)  576-588 sello(13)  589-600 "</T13001000>"           │
├─ trailer (plantilla, 18 bytes) ───────────────────────────────────────────┤
│ </T1300 ${ejercicio}(4) ${periodo}(2) 0000>                               │
└────────────────────────────────────────────────────────────────────────────┘
§ = con signo (numN): 03,07,11,14,17,19.  El resto sin signo (num).
Cada importe: 17 chars, importe×100, a la derecha con ceros. Los § (numN):
**positivo → 17 dígitos SIN espacio** (`00000000001128041`); **negativo → 'N' + 16
dígitos** (`N0000000000018695`). NO hay CRLF: el fichero es una tira de 946 bytes.

> ⚠️ **Encoding de signo — NO reutilizar `signed_amount`.** El tipo `signed_amount`
> existente codifica el positivo como `espacio + (width-1) dígitos` (formato del DR130
> **plano**, primera posición = espacio si positivo). El formato **con sobre** (`numN`
> del TS, validado byte-exact) codifica el positivo como **dígitos a lo ancho completo,
> sin espacio**; sólo el negativo lleva `'N'`. Por eso se añade un tipo nuevo
> `signed_n` (Tasks 1/2/4/6) y las casillas §03/07/11/14/17/19 lo usan (Task 7). Los
> negativos coinciden en ambos formatos (`'N'` + dígitos); sólo difiere el positivo.
```

**Encoding:** ISO-8859-1, mayúsculas, sin tildes pero conservando Ñ/Ç — lo que `runtime/sanitize.rs` ya hace. NO copiar el plegado US-ASCII del TS.

**Derivaciones de la página:** `[03]=[01]-[02]` · `[04]=round(20%·max([03],0))` · `[07]=[04]-[05]-[06]` · `[12]=[07]+[11]` · `[14]=[12]-[13]` · `[17]=[14]-[15]-[16]` · `[19]=[17]-[18]`. El redondeo de [04] se expresa `(max([03],0)*20 + 50)/100` (AEAT redondea; la división entera de boeidl trunca).

---

## Estado de partida y decisiones

- **Rama:** `feat/envelope-130` (off `main`). Hoy `main`/`feat/envelope-130` tienen el `BoeFile` **plano** (`fields`/`derives`/`checks` a nivel fichero), SIN `Record`.
- **`feat/modelo303`** (en `origin`, no mergeada) introdujo `BoeFile.records: Vec<Record>`, el tipo `unsigned_amount`, y codegen multi-struct. **La Opción B se construye encima.** → Tarea 0 integra 303.
- **Nombres de struct** (codegen ya existente, `struct_name`): record `"pagina"` → `Mod130Pagina`; record `"aux"` → `Mod130Aux`. El struct de nivel fichero lo generamos aparte como `Mod130Fichero`.
- **Fichero nuevo** `models/mod130_sobre.boe`, modelo `"130"` versión `"2015-v12"` (DR130e15v12). NO se toca `models/mod130.boe` (plano, v11).
- **Colisión de nombre de salida:** `boeidl compile` escribe `mod<number>.rs` (sería `mod130.rs` para ambos). El crate `example` no usa el CLI: su `build.rs` llama `generate()` directamente y elige el nombre de salida, así que compila el plano a `mod130.rs` y el sobre a `mod130_sobre.rs`. Sin cambios en el CLI.
- **Consistencia ejercicio/periodo:** los mismos `${ejercicio}`/`${periodo}` alimentan header y trailer → consistentes por construcción. La página lleva además sus propias casillas de ejercicio/periodo (posiciones 103-108); el usuario las fija coherentemente. Auto-propagación fichero→página queda fuera de scope (YAGNI, nota en el modelo).

---

## Estructura de ficheros

| Fichero | Responsabilidad | Acción |
|---|---|---|
| `crates/boeidl/src/runtime/bytes.rs` | `append_latin1`, `verify_literal`, `parse_signed_amount_n` | Modificar |
| `crates/boeidl/src/runtime/encode.rs` | `encode_signed_amount_n` (estilo `numN`) | Modificar |
| `crates/boeidl/src/runtime/errors.rs` | variante `AeatError::InvalidDelimiter` | Modificar |
| `crates/boeidl/src/runtime/mod.rs` | re-export de los helpers nuevos | Modificar |
| `crates/boeidl/src/ast.rs` | `Envelope`, `Param`, `Template`, `TemplatePart`, `FieldType::SignedAmountN`; `BoeFile.envelope` | Modificar |
| `crates/boeidl/src/grammar.pest` | `envelope_block`, `param_block`, `template` | Modificar |
| `crates/boeidl/src/parser.rs` | parseo de envelope/param/template + keyword `signed_n` | Modificar |
| `crates/boeidl/src/main.rs` | `field_type_name` para `SignedAmountN` (inspect) | Modificar |
| `crates/boeidl/src/validator.rs` | validación del envelope + `decimals` en `signed_n` | Modificar |
| `crates/boeidl/src/codegen/rust.rs` | `SignedAmountN` en records + emitir `Mod130Fichero` | Modificar |
| `models/mod130_sobre.boe` | modelo del 130 con sobre (aux + pagina + envelope) | Crear |
| `example/build.rs` | compilar también `mod130_sobre.boe` | Modificar |
| `example/src/generated.rs` | submódulo `sobre` para el módulo generado | Modificar |
| `example/tests/roundtrip_sobre.rs` | golden byte-exact + round-trip (cifras reales 1T2026) | Crear |
| `crates/boeidl/tests/parse_envelope.rs` | parser+validador del bloque envelope | Crear |

---

## Task 0: Integrar `feat/modelo303`

**Contexto:** La Opción B necesita `BoeFile.records` y el codegen multi-struct de 303. Integramos 303 en `feat/envelope-130` antes de tocar nada. `feat/envelope-130` sólo añade `docs/plans/…`; el merge no debería conflictuar.

- [ ] **Step 1: Confirmar que 303 no es ancestro y que el merge es limpio**

Run:
```bash
git -C /home/aldo/Dev/OpenHacienda/boeidl fetch origin
git -C /home/aldo/Dev/OpenHacienda/boeidl merge-base --is-ancestor origin/feat/modelo303 HEAD && echo "YA integrado" || echo "pendiente"
git -C /home/aldo/Dev/OpenHacienda/boeidl merge --no-commit --no-ff origin/feat/modelo303
```
Expected: `pendiente`, y `merge --no-commit` sin conflictos (`Automatic merge went well`).

- [ ] **Step 2: Correr el suite completo antes de commitear el merge**

Run: `cargo test --workspace`
Expected: PASS (tests de 130 plano y de 303 verdes).

- [ ] **Step 3: Cerrar el merge**

Run:
```bash
git -C /home/aldo/Dev/OpenHacienda/boeidl commit -m "Merge feat/modelo303: base multi-record para el envelope"
```

Si hubiera conflictos en Step 1, resolverlos preservando ambos lados (303 aporta el código; envelope-130 sólo docs) y repetir Step 2 antes de commitear.

---

## Task 1: Runtime — helpers de sobre (`append_latin1`, `verify_literal`) + error

**Files:**
- Modify: `crates/boeidl/src/runtime/errors.rs`
- Modify: `crates/boeidl/src/runtime/bytes.rs`
- Modify: `crates/boeidl/src/runtime/encode.rs`
- Modify: `crates/boeidl/src/runtime/mod.rs`

- [ ] **Step 1: Añadir la variante de error**

En `errors.rs`, dentro de `enum AeatError`, tras `InvalidEncoding`:
```rust
    /// Un delimitador fijo esperado del sobre (`<T130…>`, `<AUX>`, …) no coincide.
    InvalidDelimiter {
        context: String,
        expected: String,
        got: String,
    },
```
Y en el `impl Display`, tras el brazo `InvalidEncoding`:
```rust
            Self::InvalidDelimiter { context, expected, got } => write!(
                f,
                "delimitador inválido en {context}: esperaba `{expected}`, encontrado `{got}`"
            ),
```

- [ ] **Step 2: Escribir los tests de los helpers (fallan)**

En `bytes.rs`, dentro de `mod tests`:
```rust
    #[test]
    fn append_latin1_pushes_bytes() {
        let mut buf = Vec::new();
        append_latin1(&mut buf, "<AUX>").unwrap();
        assert_eq!(buf, b"<AUX>");
        append_latin1(&mut buf, "PEÑA").unwrap();
        assert_eq!(buf[5], b'P');
        assert_eq!(buf[7], 0xD1); // Ñ
    }

    #[test]
    fn append_latin1_rejects_non_latin1() {
        let mut buf = Vec::new();
        assert!(append_latin1(&mut buf, "€").is_err());
    }

    #[test]
    fn verify_literal_ok_and_mismatch() {
        let data = b"<T13002026";
        verify_literal(data, 0, "<T1300", "header").unwrap();
        let err = verify_literal(data, 0, "<AUX>", "header").unwrap_err();
        matches!(err, AeatError::InvalidDelimiter { .. });
    }

    #[test]
    fn verify_literal_short_data() {
        let err = verify_literal(b"<T", 0, "<T1300", "header").unwrap_err();
        matches!(err, AeatError::ShortRecord { .. });
    }
```

- [ ] **Step 3: Correr para ver fallar**

Run: `cargo test -p boeidl --lib append_latin1 verify_literal`
Expected: FAIL (funciones no definidas).

- [ ] **Step 4: Implementar los helpers**

En `bytes.rs`, antes de `#[cfg(test)]`:
```rust
/// Append `s` to `buf`, one ISO-8859-1 byte per char (each `c as u32` < 256).
/// Used by the envelope marshaler for header/trailer templates (sequential,
/// unlike `write_field`'s fixed positions).
pub fn append_latin1(buf: &mut Vec<u8>, s: &str) -> Result<(), AeatError> {
    for c in s.chars() {
        let cp = c as u32;
        if cp >= 256 {
            return Err(AeatError::InvalidValue {
                field: "<template>".to_string(),
                value: s.to_string(),
            });
        }
        buf.push(cp as u8);
    }
    Ok(())
}

/// Verify that the fixed literal `expected` sits at 0-indexed byte offset `at`
/// in `data`. `context` names the segment for the error message.
pub fn verify_literal(
    data: &[u8],
    at: usize,
    expected: &str,
    context: &str,
) -> Result<(), AeatError> {
    let want: Vec<u8> = expected.chars().map(|c| c as u8).collect();
    let end = at + want.len();
    if data.len() < end {
        return Err(AeatError::ShortRecord {
            expected: end,
            got: data.len(),
        });
    }
    if data[at..end] != want[..] {
        let got: String = data[at..end].iter().map(|&b| char::from(b)).collect();
        return Err(AeatError::InvalidDelimiter {
            context: context.to_string(),
            expected: expected.to_string(),
            got,
        });
    }
    Ok(())
}
```
(Nota: `verify_literal` usa offset **0-indexado**, coherente con el marshaler secuencial del sobre; `write_field`/`read_field` siguen 1-indexados.)

- [ ] **Step 5: Encoding con signo estilo `numN` (positivo = dígitos, negativo = 'N')**

En `encode.rs`, junto a `encode_signed_amount`:
```rust
/// Encode a signed amount in the AEAT **envelope** (`numN`) convention:
/// - `value >= 0`: `width` digits, zero-padded, NO sign char.
/// - `value < 0`:  `'N'` + `width - 1` digits, zero-padded.
///
/// Differs from `encode_signed_amount` (flat DR, `' '`-for-positive). `value`
/// is the integer including decimals (12345 == 123.45 with 2 decimals).
pub fn encode_signed_amount_n(
    field: &str,
    value: i64,
    width: usize,
    _decimals: usize,
) -> Result<String, AeatError> {
    if width == 0 {
        return Err(AeatError::FieldOverflow { field: field.to_string(), width, got: 1 });
    }
    let digits = value.unsigned_abs().to_string();
    if value < 0 {
        let dw = width - 1;
        if digits.chars().count() > dw {
            return Err(AeatError::FieldOverflow { field: field.to_string(), width, got: digits.chars().count() + 1 });
        }
        Ok(format!("N{}", pad_left(&digits, dw, '0')))
    } else {
        if digits.chars().count() > width {
            return Err(AeatError::FieldOverflow { field: field.to_string(), width, got: digits.chars().count() });
        }
        Ok(pad_left(&digits, width, '0'))
    }
}
```
Tests en `encode.rs` `mod tests` (positivo sin espacio, cero, negativo con 'N', overflow):
```rust
    #[test]
    fn encode_signed_n_positive_is_all_digits() {
        assert_eq!(encode_signed_amount_n("x", 1_128_041, 17, 2).unwrap(), "00000000001128041");
        assert_eq!(encode_signed_amount_n("x", 0, 17, 2).unwrap(), "00000000000000000");
    }
    #[test]
    fn encode_signed_n_negative_has_n_prefix() {
        assert_eq!(encode_signed_amount_n("x", -18_695, 17, 2).unwrap(), "N0000000000018695");
    }
    #[test]
    fn encode_signed_n_overflow() {
        assert!(encode_signed_amount_n("x", 10_i64.pow(17), 17, 2).is_err());
        assert!(encode_signed_amount_n("x", -(10_i64.pow(16)), 17, 2).is_err());
    }
```

En `bytes.rs`, junto a `parse_signed_amount`:
```rust
/// Parse an envelope-style (`numN`) signed amount: `'N'`-prefixed → negative,
/// otherwise all digits → positive. Mirror of `encode_signed_amount_n`.
pub fn parse_signed_amount_n(buf: &[u8], at: usize, width: usize) -> Result<i64, AeatError> {
    let s = read_field(buf, at, width);
    let (sign, rest): (i64, String) = if s.starts_with('N') {
        (-1, s.chars().skip(1).collect())
    } else {
        (1, s.clone())
    };
    let trimmed = rest.trim_start_matches(['0', ' ']);
    let n: i64 = if trimmed.is_empty() {
        0
    } else {
        trimmed.parse().map_err(|_| AeatError::InvalidValue {
            field: format!("<at {at}>"),
            value: s.clone(),
        })?
    };
    Ok(sign * n)
}
```
Test en `bytes.rs` `mod tests`:
```rust
    #[test]
    fn parse_signed_n_roundtrip() {
        let mut buf = vec![b' '; 20];
        write_field(&mut buf, 1, 17, "00000000001128041", "x").unwrap();
        assert_eq!(parse_signed_amount_n(&buf, 1, 17).unwrap(), 1_128_041);
        let mut buf = vec![b' '; 20];
        write_field(&mut buf, 1, 17, "N0000000000018695", "x").unwrap();
        assert_eq!(parse_signed_amount_n(&buf, 1, 17).unwrap(), -18_695);
    }
```

- [ ] **Step 6: Re-exportar**

En `runtime/mod.rs`:
```rust
pub use bytes::{
    append_latin1, parse_signed_amount, parse_signed_amount_n, parse_unsigned_amount,
    read_field, verify_literal, write_field,
};
pub use encode::{
    encode_number, encode_signed_amount, encode_signed_amount_n, encode_unsigned_amount,
    pad_left, pad_right,
};
```

- [ ] **Step 7: Correr tests**

Run: `cargo test -p boeidl --lib`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/boeidl/src/runtime/
git commit -m "runtime: envelope helpers (append_latin1, verify_literal, signed_n)"
```

---

## Task 2: AST — `Envelope`, `Param`, `Template`

**Files:**
- Modify: `crates/boeidl/src/ast.rs`

- [ ] **Step 0: Añadir el tipo de campo `SignedAmountN`**

En `enum FieldType`, tras `SignedAmount`:
```rust
    /// Importe con signo estilo sobre AEAT (`numN`): positivo = dígitos a lo
    /// ancho completo (sin espacio), negativo = `'N'` + dígitos.
    SignedAmountN,
```

- [ ] **Step 1: Añadir los tipos del envelope**

Tras `struct Record { … }` (o al final del bloque de structs):
```rust
/// Una plantilla de texto de longitud fija: literal + interpolaciones `${ident}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplatePart {
    Lit(String),
    /// Referencia a un `param` del envelope por nombre.
    Field(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template(pub Vec<TemplatePart>);

/// Un valor compartido a nivel fichero, interpolable en header/trailer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub length: usize,
    pub ty: FieldType,
    pub required: bool,
    pub description: Option<String>,
}

/// El "sobre" AEAT: header/trailer de longitud fija que envuelven una
/// secuencia de `record`s (`contains`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub params: Vec<Param>,
    pub header: Template,
    pub trailer: Template,
    /// Nombres de `record` en orden de concatenación.
    pub contains: Vec<String>,
}
```

- [ ] **Step 2: Añadir el campo a `BoeFile`**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct BoeFile {
    pub model: Model,
    pub records: Vec<Record>,
    pub envelope: Option<Envelope>,
}
```

- [ ] **Step 3: `cargo check`**

Run: `cargo check -p boeidl`
Expected: FAIL sólo donde se construye `BoeFile { model, records }` (parser). Se arregla en Task 4.

---

## Task 3: Grammar — `envelope_block`, `param_block`, `template`

**Files:**
- Modify: `crates/boeidl/src/grammar.pest`

- [ ] **Step 1: Reglas de plantilla** (tras la regla `string`)
```pest
// Plantilla: literal + interpolaciones ${ident}. Compound-atomic para NO
// tragarse los espacios (son significativos: el layout es de ancho fijo).
template  = ${ "\"" ~ tmpl_part* ~ "\"" }
tmpl_part = ${ tmpl_ref | tmpl_text }
tmpl_ref  = ${ "${" ~ ident ~ "}" }
tmpl_text = @{ (!("\"" | "${") ~ ANY)+ }
```

- [ ] **Step 2: `param` y `envelope`** (tras `record_block`)
```pest
// ── param (valor compartido a nivel fichero) ─────────────────────────────
param_block = { "param" ~ ident ~ "{" ~ field_attr* ~ "}" }

// ── envelope (sobre) ─────────────────────────────────────────────────────
envelope_block = { "envelope" ~ "{"
    ~ (param_block | env_header | env_trailer | env_contains)*
    ~ "}" }
env_header   = { "header"  ~ "=" ~ template }
env_trailer  = { "trailer" ~ "=" ~ template }
env_contains = { "contains" ~ "=" ~ "[" ~ ident ~ ("," ~ ident)* ~ "]" }
```

- [ ] **Step 3: Añadir `envelope_block` al top-level**
```pest
item = _{ record_block | envelope_block | field_block | derive_stmt | check_block }
```

- [ ] **Step 4: Verificar que compila la gramática**

Run: `cargo check -p boeidl`
Expected: los errores siguen siendo sólo los de Task 2 (construcción de `BoeFile`); la gramática compila.

---

## Task 4: Parser — envelope/param/template

**Files:**
- Modify: `crates/boeidl/src/parser.rs`

- [ ] **Step 1: Recoger el envelope en el bucle top-level**

En `parse`, junto a `let mut records`, añadir `let mut envelope: Option<Envelope> = None;`. En el `match pair.as_rule()`:
```rust
            Rule::envelope_block => {
                if envelope.is_some() {
                    return Err(err("sólo se permite un bloque `envelope` por fichero"));
                }
                envelope = Some(parse_envelope(pair)?);
            }
```
Y al construir el resultado, cambiar la creación de `BoeFile`:
```rust
    Ok(BoeFile { model, records, envelope })
```
(Se mantiene el synthesized record del camino legacy; un fichero con `envelope` **debe** declarar `record` blocks, así que `records` no estará vacío — lo valida Task 5.)

- [ ] **Step 2: Implementar `parse_envelope`, `parse_param`, `parse_template`**

```rust
fn parse_envelope(pair: Pair<Rule>) -> Result<Envelope, ParseError> {
    let mut params = Vec::new();
    let mut header: Option<Template> = None;
    let mut trailer: Option<Template> = None;
    let mut contains: Vec<String> = Vec::new();

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::param_block => params.push(parse_param(child)?),
            Rule::env_header => {
                let t = child.into_inner().next().ok_or_else(|| err("header: falta plantilla"))?;
                header = Some(parse_template(t)?);
            }
            Rule::env_trailer => {
                let t = child.into_inner().next().ok_or_else(|| err("trailer: falta plantilla"))?;
                trailer = Some(parse_template(t)?);
            }
            Rule::env_contains => {
                for id in child.into_inner() {
                    contains.push(id.as_str().to_string());
                }
            }
            r => return Err(err(format!("regla inesperada en envelope: {r:?}"))),
        }
    }

    Ok(Envelope {
        params,
        header: header.ok_or_else(|| err("envelope: falta `header`"))?,
        trailer: trailer.ok_or_else(|| err("envelope: falta `trailer`"))?,
        contains,
    })
}

fn parse_template(pair: Pair<Rule>) -> Result<Template, ParseError> {
    // pair == Rule::template; sus hijos son tmpl_part.
    let mut parts = Vec::new();
    for part in pair.into_inner() {
        let inner = part.into_inner().next().ok_or_else(|| err("tmpl_part vacío"))?;
        match inner.as_rule() {
            Rule::tmpl_ref => {
                // el hijo del ref es el ident
                let id = inner.into_inner().next().ok_or_else(|| err("tmpl_ref sin ident"))?;
                parts.push(TemplatePart::Field(id.as_str().to_string()));
            }
            Rule::tmpl_text => parts.push(TemplatePart::Lit(inner.as_str().to_string())),
            r => return Err(err(format!("regla inesperada en plantilla: {r:?}"))),
        }
    }
    Ok(Template(parts))
}

fn parse_param(pair: Pair<Rule>) -> Result<Param, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().ok_or_else(|| err("param: falta nombre"))?.as_str().to_string();

    let mut length: Option<usize> = None;
    let mut ty: Option<FieldType> = None;
    let mut required = false;
    let mut description = None;

    for attr in inner {
        let (key, value) = key_value(attr)?;
        let v = value.into_inner().next().ok_or_else(|| err("param attr: valor vacío"))?;
        match (key.as_str(), v.as_rule()) {
            ("length", Rule::int) => length = Some(v.as_str().parse().map_err(|e| err(format!("length: {e}")))?),
            ("type", Rule::ident) => {
                ty = Some(match v.as_str() {
                    "alpha" => FieldType::Alpha,
                    "alphanumeric" => FieldType::Alphanumeric,
                    "number" => FieldType::Number,
                    other => return Err(err(format!("param `{name}`: tipo no soportado `{other}` (usa alpha/alphanumeric/number)"))),
                });
            }
            ("required", Rule::ident) => {
                required = match v.as_str() {
                    "true" => true,
                    "false" => false,
                    other => return Err(err(format!("required debe ser true/false, got {other}"))),
                };
            }
            ("description", Rule::string) => description = Some(unquote(v.as_str())),
            (k, r) => return Err(err(format!("param `{name}`: atributo no soportado `{k}` (rule {r:?})"))),
        }
    }

    Ok(Param {
        name,
        length: length.ok_or_else(|| err("param: falta `length`"))?,
        ty: ty.ok_or_else(|| err("param: falta `type`"))?,
        required,
        description,
    })
}
```

- [ ] **Step 3: Reconocer el keyword `signed_n` y arreglar los `match` exhaustivos**

En `parser.rs`, en `parse_field`, dentro del `match v.as_str()` de `("type", Rule::ident)`, añadir:
```rust
                    "signed_n" => FieldType::SignedAmountN,
```
En `main.rs`, en `field_type_name`, el `match f.ty` es exhaustivo — añadir:
```rust
        FieldType::SignedAmountN => match f.decimals {
            Some(d) => format!("amountN({d})"),
            None => "amountN".to_string(),
        },
```

- [ ] **Step 4: Verificar que compila**

Run: `cargo check -p boeidl`
Expected: PASS (salvo los `match` de codegen, que se completan en Task 6; si `cargo check` se queja de no-exhaustividad en `codegen/rust.rs`, es esperado y lo cierra Task 6 — puedes hacer Task 6 antes de compilar el workspace). `cargo test -p boeidl` sigue verde.

- [ ] **Step 5: Commit** (AST + grammar + parser juntos, la gramática y el AST deben ir sincronizados)

```bash
git add crates/boeidl/src/ast.rs crates/boeidl/src/grammar.pest crates/boeidl/src/parser.rs crates/boeidl/src/main.rs
git commit -m "dsl: parse envelope block + signed_n field type"
```

---

## Task 5: Validador — coherencia del envelope

**Files:**
- Modify: `crates/boeidl/src/validator.rs`
- Test: `crates/boeidl/tests/parse_envelope.rs` (Crear)

- [ ] **Step 0: Permitir `decimals` en `signed_n`**

En `validator.rs`, la comprobación de `decimals` sólo acepta hoy `SignedAmount | UnsignedAmount`. Ampliar para incluir el tipo nuevo:
```rust
            if f.decimals.is_some()
                && !matches!(
                    f.ty,
                    FieldType::SignedAmount | FieldType::SignedAmountN | FieldType::UnsignedAmount
                )
            {
```

- [ ] **Step 1: Test de validación (falla)**

Crear `crates/boeidl/tests/parse_envelope.rs`:
```rust
use boeidl::parse;
use boeidl::validator::{validate, DiagLevel};

fn errors(src: &str) -> Vec<String> {
    let file = parse(src).expect("parse");
    validate(&file)
        .into_iter()
        .filter(|d| d.level == DiagLevel::Error)
        .map(|d| d.message)
        .collect()
}

const OK: &str = r#"
model "130" version "2015-v12" {
    encoding = "ISO-8859-1"
    line_ending = "LF"
    record_length = 600
}
record "pagina" {
    record_length = 4
    field abre { at = 1 length = 4 type = alphanumeric fixed = "AAAA" }
}
envelope {
    param ejercicio { length = 4 type = number }
    param periodo   { length = 2 type = alphanumeric }
    header  = "<H${ejercicio}${periodo}>"
    trailer = "</H${ejercicio}${periodo}>"
    contains = [pagina]
}
"#;

#[test]
fn valid_envelope_has_no_errors() {
    assert!(errors(OK).is_empty(), "{:?}", errors(OK));
}

#[test]
fn contains_unknown_record_is_error() {
    let src = OK.replace("contains = [pagina]", "contains = [nope]");
    assert!(errors(&src).iter().any(|m| m.contains("nope")));
}

#[test]
fn unknown_placeholder_is_error() {
    let src = OK.replace("<H${ejercicio}${periodo}>", "<H${ejercicio}${nope}>");
    assert!(errors(&src).iter().any(|m| m.contains("nope")));
}
```

- [ ] **Step 2: Correr para ver fallar**

Run: `cargo test -p boeidl --test parse_envelope`
Expected: FAIL (`valid_envelope_has_no_errors` puede pasar por vacío, pero los dos negativos fallan porque el validador aún no mira el envelope).

- [ ] **Step 3: Añadir la validación**

Al final de `validate`, antes de `diags`:
```rust
    if let Some(env) = &file.envelope {
        let record_names: HashSet<&str> = file.records.iter().map(|r| r.name.as_str()).collect();
        let param_names: HashSet<&str> = env.params.iter().map(|p| p.name.as_str()).collect();

        // params bien formados
        for p in &env.params {
            if p.length == 0 {
                diags.push(Diagnostic::error(format!("envelope param `{}`: `length` debe ser > 0", p.name)));
            }
        }
        // contains → records existentes
        for name in &env.contains {
            if !record_names.contains(name.as_str()) {
                diags.push(Diagnostic::error(format!("envelope `contains`: record desconocido `{name}`")));
            }
        }
        if env.contains.is_empty() {
            diags.push(Diagnostic::error("envelope `contains` no puede estar vacío".to_string()));
        }
        // placeholders de header/trailer → params existentes
        for (label, tmpl) in [("header", &env.header), ("trailer", &env.trailer)] {
            for part in &tmpl.0 {
                if let TemplatePart::Field(f) = part {
                    if !param_names.contains(f.as_str()) {
                        diags.push(Diagnostic::error(format!("envelope {label}: `${{{f}}}` no es un param declarado")));
                    }
                }
            }
        }
    }
```
(`TemplatePart` ya entra por `use crate::ast::*;`.)

- [ ] **Step 4: Correr tests**

Run: `cargo test -p boeidl --test parse_envelope`
Expected: PASS los 3.

- [ ] **Step 5: Commit**

```bash
git add crates/boeidl/src/validator.rs crates/boeidl/tests/parse_envelope.rs
git commit -m "validator: check envelope contains refs and placeholders"
```

---

## Task 6: Codegen — struct `Mod130Fichero`

**Files:**
- Modify: `crates/boeidl/src/codegen/rust.rs`

- [ ] **Step 0: Manejar `SignedAmountN` en el codegen de records**

Cuatro `match` sobre `FieldType` necesitan el nuevo brazo (los tres primeros son exhaustivos → si no, no compila):

`rust_type_for`:
```rust
        FieldType::SignedAmount | FieldType::SignedAmountN | FieldType::UnsignedAmount => "i64",
```
`emit_marshal_field` — nuevo brazo (usa `encode_signed_amount_n`):
```rust
        FieldType::SignedAmountN => {
            let decimals = f.decimals.unwrap_or(0);
            writeln!(
                out,
                "        {{ let s = encode_signed_amount_n(\"{name}\", self.{name}, {}, {})?; \
                 write_field(&mut buf, {}, {}, &s, \"{name}\")?; }}",
                f.length, decimals, f.at, f.length,
            )
            .unwrap();
        }
```
`emit_unmarshal` — nuevo brazo (usa `parse_signed_amount_n`):
```rust
            FieldType::SignedAmountN => {
                writeln!(
                    out,
                    "        out.{name} = parse_signed_amount_n(data, {}, {})?;",
                    f.at, f.length
                )
                .unwrap();
            }
```
`emit_expr` (rama `Expr::Ident`) — tratar `SignedAmountN` como importe (i64), NO como `.as_str()`:
```rust
            Some(FieldType::SignedAmount) | Some(FieldType::SignedAmountN)
            | Some(FieldType::UnsignedAmount) => format!("self.{name}"),
```

- [ ] **Step 1: Llamar al emisor del fichero**

En `generate`, tras el bucle `for r in &file.records { … }`:
```rust
    if let Some(env) = &file.envelope {
        emit_envelope(&mut out, &file.model, env);
    }
```

- [ ] **Step 2: Helpers de nombre**

Añadir cerca de `struct_name`:
```rust
fn file_struct_name(model_number: &str) -> String {
    format!("Mod{model_number}Fichero")
}

/// Longitud de un param al interpolar (= su `length`, ancho fijo).
fn param_len<'a>(env: &'a Envelope, name: &str) -> usize {
    env.params.iter().find(|p| p.name == name).map(|p| p.length).unwrap_or(0)
}
```

- [ ] **Step 3: Implementar `emit_envelope`**

```rust
fn emit_envelope(out: &mut String, model: &Model, env: &Envelope) {
    let name = file_struct_name(&model.number);

    // ── struct ──
    writeln!(out, "#[derive(Debug, Clone, Default, PartialEq, Eq)]").unwrap();
    writeln!(out, "pub struct {name} {{").unwrap();
    for p in &env.params {
        if let Some(d) = &p.description {
            writeln!(out, "    /// {d}").unwrap();
        }
        writeln!(out, "    pub {}: String,", p.name).unwrap();
    }
    for rec in &env.contains {
        writeln!(out, "    pub {}: {},", rec, struct_name(&model.number, rec)).unwrap();
    }
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "impl {name} {{").unwrap();
    emit_envelope_marshal(out, model, env);
    emit_envelope_unmarshal(out, model, env);
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
}
```

- [ ] **Step 4: `marshal`**

```rust
fn emit_envelope_marshal(out: &mut String, model: &Model, env: &Envelope) {
    writeln!(out, "    pub fn marshal(&self) -> Result<Vec<u8>, AeatError> {{").unwrap();
    writeln!(out, "        let mut buf: Vec<u8> = Vec::new();").unwrap();
    emit_template_marshal(out, env, &env.header);
    for rec in &env.contains {
        writeln!(out, "        buf.extend_from_slice(&self.{rec}.marshal()?);").unwrap();
    }
    emit_template_marshal(out, env, &env.trailer);
    writeln!(out, "        Ok(buf)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    let _ = model;
}

fn emit_template_marshal(out: &mut String, env: &Envelope, tmpl: &Template) {
    for part in &tmpl.0 {
        match part {
            TemplatePart::Lit(s) => {
                writeln!(out, "        append_latin1(&mut buf, \"{}\")?;", escape_str(s)).unwrap();
            }
            TemplatePart::Field(name) => {
                let len = param_len(env, name);
                let ty = env.params.iter().find(|p| p.name == *name).map(|p| p.ty);
                match ty {
                    Some(FieldType::Number) => {
                        writeln!(out,
                            "        {{ let digits: String = self.{name}.chars().filter(|c| c.is_ascii_digit()).collect(); \
                             if digits.chars().count() > {len} {{ return Err(AeatError::FieldOverflow {{ field: \"{name}\".to_string(), width: {len}, got: digits.chars().count() }}); }} \
                             append_latin1(&mut buf, &pad_left(&digits, {len}, '0'))?; }}").unwrap();
                    }
                    _ => {
                        // alpha / alphanumeric
                        writeln!(out,
                            "        {{ let s = pad_right(&sanitize_alphanumeric(&self.{name}), {len}, ' '); \
                             if s.chars().count() > {len} {{ return Err(AeatError::FieldOverflow {{ field: \"{name}\".to_string(), width: {len}, got: s.chars().count() }}); }} \
                             append_latin1(&mut buf, &s)?; }}").unwrap();
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 5: `unmarshal`** (offset-based; verifica literales, extrae params, trocea el cuerpo)

```rust
fn emit_envelope_unmarshal(out: &mut String, model: &Model, env: &Envelope) {
    writeln!(out, "    pub fn unmarshal(data: &[u8]) -> Result<Self, AeatError> {{").unwrap();
    writeln!(out, "        let mut out = Self::default();").unwrap();
    writeln!(out, "        let mut off = 0usize;").unwrap();
    emit_template_unmarshal(out, env, &env.header, "header");
    let single = env.contains.len() == 1 && count_records(model, env) == 1;
    for rec in &env.contains {
        let rl_const = record_length_const_name(rec, single);
        let st = struct_name(&model.number, rec);
        writeln!(out,
            "        {{ let end = off + {rl_const}; if data.len() < end {{ return Err(AeatError::ShortRecord {{ expected: end, got: data.len() }}); }} \
             out.{rec} = {st}::unmarshal(&data[off..end])?; off = end; }}").unwrap();
    }
    emit_template_unmarshal(out, env, &env.trailer, "trailer");
    writeln!(out, "        let _ = off;").unwrap();
    writeln!(out, "        Ok(out)").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
}

fn emit_template_unmarshal(out: &mut String, env: &Envelope, tmpl: &Template, ctx: &str) {
    for part in &tmpl.0 {
        match part {
            TemplatePart::Lit(s) => {
                writeln!(out,
                    "        verify_literal(data, off, \"{}\", \"{ctx}\")?; off += {};",
                    escape_str(s), s.chars().count()).unwrap();
            }
            TemplatePart::Field(name) => {
                let len = param_len(env, name);
                let ty = env.params.iter().find(|p| p.name == *name).map(|p| p.ty);
                // read_field es 1-indexado; off es 0-indexado.
                match ty {
                    Some(FieldType::Number) => {
                        writeln!(out,
                            "        out.{name} = read_field(data, off + 1, {len}); off += {len};").unwrap();
                    }
                    _ => {
                        writeln!(out,
                            "        out.{name} = read_field(data, off + 1, {len}).trim_end().to_string(); off += {len};").unwrap();
                    }
                }
            }
        }
    }
}
```
Añadir el helper `count_records`:
```rust
fn count_records(_model: &Model, _env: &Envelope) -> usize {
    // El nº de records del fichero lo conoce `generate`; para el naming del
    // const de longitud usamos `single` calculado ahí. Simplificamos: el
    // codegen del envelope recibe `single` desde `generate`.
    1
}
```
> **NOTA de implementación:** `record_length_const_name(rec, single)` necesita saber si el fichero es de un solo record (para elegir `RECORD_LENGTH` vs `RECORD_LENGTH_<REC>`). En vez del `count_records` de arriba, pasa `single` explícitamente: cambia la firma a `emit_envelope(out, &file.model, env, single)` donde `single = file.records.len() == 1`, y propágalo a `emit_envelope_unmarshal`. Con `mod130_sobre.boe` habrá **2 records** (`aux`, `pagina`), así que `single == false` y los consts serán `RECORD_LENGTH_AUX` / `RECORD_LENGTH_PAGINA`. Elimina `count_records`.

- [ ] **Step 6: Corregir el naming con `single` real** (aplicar la nota anterior)

En `generate`:
```rust
    if let Some(env) = &file.envelope {
        emit_envelope(&mut out, &file.model, env, file.records.len() == 1);
    }
```
`emit_envelope(out, model, env, single)` → pasa `single` a `emit_envelope_unmarshal(out, model, env, single)` → usa `record_length_const_name(rec, single)`.

- [ ] **Step 7: Compilar**

Run: `cargo test -p boeidl`
Expected: PASS (los tests existentes no usan envelope; el codegen nuevo sólo se ejercita en Task 8).

- [ ] **Step 8: Commit**

```bash
git add crates/boeidl/src/codegen/rust.rs
git commit -m "codegen: emit Mod<n>Fichero (envelope marshal/unmarshal)"
```

---

## Task 7: Modelo `models/mod130_sobre.boe`

**Files:**
- Create: `models/mod130_sobre.boe`

- [ ] **Step 1: Escribir el modelo**

```
# Modelo 130 "con sobre" — fichero de presentación real (DR130e15v12).
# 946 bytes = header(17) + aux(311) + pagina(600) + trailer(18). SIN CRLF.
# NO sustituye a mod130.boe (registro plano de 878). Import/presentación en Sede.
# Layout validado byte-exact contra ficheros reales (ver test/modelo130.test.ts).

model "130" version "2015-v12" {
    encoding = "ISO-8859-1"
    line_ending = "LF"       # sin efecto: el fichero no lleva separador de línea
    record_length = 600      # record primario (pagina)
}

# ── AUX (311 bytes): datos de la Entidad Desarrolladora (en blanco en import) ──
record "aux" {
    record_length = 311
    field abre_aux        { at = 1   length = 5   type = alphanumeric fixed = "<AUX>" }
    field version_programa { at = 76  length = 4   type = alphanumeric description = "Versión del programa (ED). Blanco en prefill." }
    field nif_ed          { at = 84  length = 9   type = alphanumeric description = "NIF entidad desarrolladora. Blanco si no aplica." }
    field cierra_aux      { at = 306 length = 6   type = alphanumeric fixed = "</AUX>" }
    # Huecos 6-75, 80-83, 93-305 se rellenan con blancos (warnings de gap esperados).
}

# ── PÁGINA (600 bytes): registro DR 13001 con importes de 17 ──────────────────
record "pagina" {
    record_length = 600
    field abre_pagina { at = 1  length = 11 type = alphanumeric fixed = "<T13001000>" }
    field indicador_complementaria { at = 12 length = 1 type = alphanumeric description = "Indicador página complementaria. Blanco." }
    field tipo_declaracion { at = 13 length = 1 type = alpha required = true domain = "B|G|I|N|U"
        description = "B=a deducir, G=CCC-ingreso, I=ingreso, N=negativa, U=domiciliación." }
    field nif       { at = 14  length = 9  type = alphanumeric required = true description = "NIF del declarante." }
    field apellidos { at = 23  length = 60 type = alphanumeric description = "Apellidos del declarante." }
    field nombre    { at = 83  length = 20 type = alphanumeric description = "Nombre del declarante." }
    field ejercicio { at = 103 length = 4  type = number required = true description = "Ejercicio (AAAA). Debe coincidir con el del sobre." }
    field periodo   { at = 107 length = 2  type = alphanumeric required = true domain = "1T|2T|3T|4T"
        description = "Periodo. Debe coincidir con el del sobre." }

    field c01_ingresos          { at = 109 length = 17 type = unsigned_amount decimals = 2 description = "[01] Ingresos computables." }
    field c02_gastos            { at = 126 length = 17 type = unsigned_amount decimals = 2 description = "[02] Gastos deducibles." }
    field c03_rendimiento_neto  { at = 143 length = 17 type = signed_n        decimals = 2 description = "[03] Rendimiento neto [01]-[02]." }
    field c04_20pct             { at = 160 length = 17 type = unsigned_amount decimals = 2 description = "[04] 20% de [03]." }
    field c05_deducir_anteriores { at = 177 length = 17 type = unsigned_amount decimals = 2 description = "[05] A deducir trimestres anteriores." }
    field c06_retenciones       { at = 194 length = 17 type = unsigned_amount decimals = 2 description = "[06] A deducir retenciones." }
    field c07_pago_fraccionado  { at = 211 length = 17 type = signed_n        decimals = 2 description = "[07] Pago fraccionado [04]-[05]-[06]." }
    field c08_ingresos_agricola { at = 228 length = 17 type = unsigned_amount decimals = 2 description = "[08] Ingresos actividades agrícolas." }
    field c09_2pct              { at = 245 length = 17 type = unsigned_amount decimals = 2 description = "[09] 2% de [08]." }
    field c10_retenciones_agricola { at = 262 length = 17 type = unsigned_amount decimals = 2 description = "[10] A deducir retenciones (agrícola)." }
    field c11_pago_agricola     { at = 279 length = 17 type = signed_n        decimals = 2 description = "[11] Pago fraccionado (agrícola)." }
    field c12_suma_pagos        { at = 296 length = 17 type = unsigned_amount decimals = 2 description = "[12] Suma pagos [07]+[11]." }
    field c13_minoracion_80bis  { at = 313 length = 17 type = unsigned_amount decimals = 2 description = "[13] Minoración art. 80 bis." }
    field c14_diferencia        { at = 330 length = 17 type = signed_n        decimals = 2 description = "[14] Diferencia [12]-[13]." }
    field c15_negativos_anteriores { at = 347 length = 17 type = unsigned_amount decimals = 2 description = "[15] A deducir negativos anteriores." }
    field c16_vivienda          { at = 364 length = 17 type = unsigned_amount decimals = 2 description = "[16] Préstamo vivienda habitual." }
    field c17_total             { at = 381 length = 17 type = signed_n        decimals = 2 description = "[17] Total [14]-[15]-[16]." }
    field c18_anteriores_declaraciones { at = 398 length = 17 type = unsigned_amount decimals = 2 description = "[18] A deducir declaraciones anteriores." }
    field c19_resultado         { at = 415 length = 17 type = signed_n        decimals = 2 description = "[19] Resultado de la declaración." }

    field marca_complementaria { at = 432 length = 1  type = alphanumeric domain = " |X" description = "Marca complementaria." }
    field justificante_anterior { at = 433 length = 13 type = alphanumeric description = "Justificante declaración anterior." }
    field iban    { at = 446 length = 34 type = alphanumeric description = "IBAN para domiciliación." }
    field reservado { at = 480 length = 96 type = alphanumeric description = "Reservado AEAT (blancos)." }
    field sello   { at = 576 length = 13 type = alphanumeric description = "Sello electrónico (blancos)." }
    field cierra_pagina { at = 589 length = 12 type = alphanumeric fixed = "</T13001000>" }

    # Derivaciones. [04] redondea (AEAT) en vez de truncar: (x*20+50)/100.
    derive c03_rendimiento_neto = c01_ingresos - c02_gastos
    derive c04_20pct = (max(c03_rendimiento_neto, 0) * 20 + 50) / 100
    derive c07_pago_fraccionado = c04_20pct - c05_deducir_anteriores - c06_retenciones
    derive c09_2pct = c08_ingresos_agricola * 2 / 100
    derive c11_pago_agricola = c09_2pct - c10_retenciones_agricola
    derive c12_suma_pagos = c07_pago_fraccionado + c11_pago_agricola
    derive c14_diferencia = c12_suma_pagos - c13_minoracion_80bis
    derive c17_total = c14_diferencia - c15_negativos_anteriores - c16_vivienda
    derive c19_resultado = c17_total - c18_anteriores_declaraciones

    check E301 { rule = c03_rendimiento_neto == c01_ingresos - c02_gastos
        severity = error message = "[03] debe ser [01]-[02]." }
    check E307 { rule = c07_pago_fraccionado == c04_20pct - c05_deducir_anteriores - c06_retenciones
        severity = error message = "[07] debe ser [04]-[05]-[06]." }
    check E319 { rule = c19_resultado == c17_total - c18_anteriores_declaraciones
        severity = error message = "[19] debe ser [17]-[18]." }
    check W001 { rule = tipo_declaracion == "N" implies c19_resultado <= 0
        severity = warning message = "Declaración negativa: [19] debería ser <= 0." }
}

# ── SOBRE ─────────────────────────────────────────────────────────────────────
envelope {
    param ejercicio { length = 4 type = number       required = true description = "Ejercicio (AAAA)." }
    param periodo   { length = 2 type = alphanumeric  required = true description = "Periodo (1T..4T)." }
    header  = "<T1300${ejercicio}${periodo}0000>"
    trailer = "</T1300${ejercicio}${periodo}0000>"
    contains = [aux, pagina]
}
```

- [ ] **Step 2: `check` del modelo (parseo + validación semántica)**

Run: `cargo run -p boeidl -- check models/mod130_sobre.boe`
Expected: `ok: models/mod130_sobre.boe is valid` (con warnings de gap en `aux` en stderr — esperados).

- [ ] **Step 3: Inspeccionar posiciones**

Run: `cargo run -p boeidl -- inspect models/mod130_sobre.boe`
Expected: la tabla lista los campos de `aux` y `pagina`; verificar `c01_ingresos` en `at=109 len=17` y `cierra_pagina` en `at=589 len=12`.

- [ ] **Step 4: Commit**

```bash
git add models/mod130_sobre.boe
git commit -m "models: mod130_sobre.boe (130 con sobre, importes de 17)"
```

---

## Task 8: Integración `example` + golden byte-exact

**Files:**
- Modify: `example/build.rs`
- Modify: `example/src/generated.rs`
- Create: `example/tests/roundtrip_sobre.rs`

- [ ] **Step 1: Compilar el modelo del sobre en el build**

En `example/build.rs`, tras escribir `mod130.rs`, añadir un segundo bloque:
```rust
    // Modelo 130 con sobre.
    let input_env = PathBuf::from("../models/mod130_sobre.boe");
    println!("cargo:rerun-if-changed={}", input_env.display());
    let src_env = std::fs::read_to_string(&input_env)
        .unwrap_or_else(|e| panic!("reading {}: {e}", input_env.display()));
    let file_env = boeidl::parse(&src_env).expect("parse mod130_sobre.boe");
    let errors_env: Vec<_> = boeidl::validate(&file_env)
        .into_iter()
        .filter(|d| d.level == boeidl::DiagLevel::Error)
        .collect();
    if !errors_env.is_empty() {
        for d in &errors_env { eprintln!("semantic error: {}", d.message); }
        panic!("mod130_sobre.boe failed semantic validation");
    }
    let code_env = boeidl::codegen::rust::generate(&file_env);
    std::fs::write(out_dir.join("mod130_sobre.rs"), code_env).expect("write mod130_sobre.rs");
```
(Reutiliza el `out_dir` ya calculado; muévelo arriba si hace falta.)

- [ ] **Step 2: Exponer el módulo generado**

En `example/src/generated.rs`, añadir un submódulo (mantiene los imports actuales de `Mod130` intactos):
```rust
#[allow(unused, clippy::all, unused_parens, dead_code, non_snake_case)]
pub mod sobre {
    include!(concat!(env!("OUT_DIR"), "/mod130_sobre.rs"));
}
```

- [ ] **Step 3: Test golden + round-trip (falla al compilar hasta que exista el codegen)**

Crear `example/tests/roundtrip_sobre.rs`:
```rust
//! Golden byte-exact del Modelo 130 con sobre (946 bytes) + round-trip.
//! Cifras reales 1T 2026 (ver ~/.../test/modelo130.test.ts).

use boeidl_example::generated::sobre::{Mod130Aux, Mod130Fichero, Mod130Pagina};

fn make() -> Mod130Fichero {
    let mut pag = Mod130Pagina::default();
    pag.tipo_declaracion = "I".to_string();
    pag.nif = "09030055W".to_string();
    pag.apellidos = "BORRERO GONZALEZ".to_string();
    pag.nombre = "ALDO ISMAEL".to_string();
    pag.ejercicio = "2026".to_string();
    pag.periodo = "1T".to_string();
    pag.c01_ingresos = 5_880_919; // 58809.19
    pag.c02_gastos = 240_715; //     2407.15
    pag.compute_derived(); // [03]=5640204, [04]=1128041, [07]=[12]=[14]=[17]=[19]=1128041

    Mod130Fichero {
        ejercicio: "2026".to_string(),
        periodo: "1T".to_string(),
        aux: Mod130Aux::default(),
        pagina: pag,
    }
}

#[test]
fn file_is_946_bytes() {
    assert_eq!(make().marshal().unwrap().len(), 946);
}

#[test]
fn envelope_header_and_trailer() {
    let b = make().marshal().unwrap();
    assert_eq!(&b[0..17], b"<T130020261T0000>");
    assert_eq!(&b[17..22], b"<AUX>");
    assert_eq!(&b[322..328], b"</AUX>");
    assert_eq!(&b[928..946], b"</T130020261T0000>");
}

#[test]
fn page_tag_and_identity() {
    let b = make().marshal().unwrap();
    let at = |c: usize| 328 + c - 1; // pos página 1-indexada → offset fichero
    assert_eq!(&b[328..339], b"<T13001000>");            // apertura página (offset 328)
    assert_eq!(&b[at(14)..at(14) + 9], b"09030055W");    // NIF pos 14
    assert_eq!(&b[at(103)..at(103) + 4], b"2026");       // ejercicio pos 103
    assert_eq!(&b[at(589)..at(589) + 12], b"</T13001000>"); // cierre página pos 589
}

#[test]
fn casillas_encoding() {
    let b = make().marshal().unwrap();
    let at = |c: usize| 328 + c - 1; // pos página 1-indexada → offset fichero
    assert_eq!(&b[at(109)..at(109) + 17], b"00000000005880919"); // [01] num
    assert_eq!(&b[at(143)..at(143) + 17], b"00000000005640204"); // [03] numN +
    assert_eq!(&b[at(211)..at(211) + 17], b"00000000001128041"); // [07]
    assert_eq!(&b[at(415)..at(415) + 17], b"00000000001128041"); // [19] = [07]
}

#[test]
fn round_trip() {
    let f = make();
    let b = f.marshal().unwrap();
    let f2 = Mod130Fichero::unmarshal(&b).unwrap();
    assert_eq!(f2.ejercicio, "2026");
    assert_eq!(f2.periodo, "1T");
    assert_eq!(f2.pagina.nif, "09030055W");
    assert_eq!(f2.pagina.c01_ingresos, 5_880_919);
    assert_eq!(f2.pagina.c19_resultado, 1_128_041);
    assert_eq!(f2.marshal().unwrap(), b); // re-serialize idéntico
}

#[test]
fn unmarshal_rejects_bad_header() {
    let mut b = make().marshal().unwrap();
    b[0] = b'X'; // rompe "<T1300…"
    assert!(Mod130Fichero::unmarshal(&b).is_err());
}
```

- [ ] **Step 4: Correr para ver fallar → luego pasar**

Run: `cargo test -p boeidl_example --test roundtrip_sobre`
Expected primero: FAIL de compilación si algo del codegen no cuadra; iterar sobre `emit_envelope` hasta PASS de los 6 tests. Si `casillas_encoding` falla en `[04]/[07]`, revisar el redondeo `(x*20+50)/100`.

- [ ] **Step 5: Commit**

```bash
git add example/build.rs example/src/generated.rs example/tests/roundtrip_sobre.rs
git commit -m "example: golden byte-exact del 130 con sobre (946 bytes) + round-trip"
```

---

## Task 9: Backward-compat, fmt/clippy, verificación final

- [ ] **Step 1: Suite completa**

Run: `cargo test --workspace`
Expected: PASS todo — 130 plano (`RECORD_LENGTH==878`), 303, envelope, y los tests nuevos.

- [ ] **Step 2: Formato y lint**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings`
Expected: sin cambios de fmt pendientes y clippy limpio. (Si clippy se queja del código generado, confirmar que `example/src/generated.rs` mantiene `#[allow(clippy::all, …)]` en el submódulo `sobre`.)

- [ ] **Step 3: Golden opcional contra fichero real**

Si el usuario aporta un `.txt` presentado real del 1T2026, guardarlo en `testdata/mod130_sobre_1t2026.txt` y añadir un test que compare `make().marshal().unwrap()` con sus bytes exactos. (Hasta entonces, el golden se auto-verifica contra el layout validado del TS.)

- [ ] **Step 4: Commit final si Step 2 tocó algo**

```bash
git add -A && git commit -m "fmt/clippy tras soporte de envelope 130"
```

---

## Criterio de "hecho"

- [ ] `cargo test --workspace` verde, incluidos 130 plano y 303 (backward-compat).
- [ ] `Mod130Fichero::marshal()` produce **946 bytes** que empiezan por `<T1300…` y terminan en `</T1300…>`, byte-idénticos al layout validado (cifras 1T2026).
- [ ] Round-trip: `unmarshal(marshal()) == entrada` y `marshal(unmarshal(bytes)) == bytes`.
- [ ] Los `check` (E301/E307/E319) validan contra las cifras reales con el redondeo de [04].
- [ ] `models/mod130.boe` plano intacto; el DSL sin `envelope` se comporta como antes.

## Notas / deuda técnica consciente (YAGNI)

- **Redondeo de [04]:** el modelo del sobre usa `(x*20+50)/100` (redondeo AEAT); el plano `mod130.boe` sigue truncando. No se unifican en este PR.
- **Consistencia sobre↔página:** ejercicio/periodo se declaran en el sobre *y* en la página; el usuario los fija coherentes. Auto-propagación (fichero→records) o un `check` cross-record quedan para un PR futuro.
- **ED en el AUX:** `version_programa`/`nif_ed` existen como campos pero van en blanco en el import de prefill; el golden los deja vacíos.
- **Generalización 303/349/390:** el bloque `envelope` ya soporta `repeat`/`when` conceptualmente (plan original), pero **no** se implementan aquí — se añadirán cuando un modelo real con páginas repetidas los necesite.
```