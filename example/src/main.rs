//! Minimal demo: populate a Mod130, derive, marshal, write to stdout.

use boeidl_example::generated::{Mod130, MODEL_NUMBER, MODEL_VERSION, RECORD_LENGTH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Modelo {MODEL_NUMBER} v{MODEL_VERSION} — {RECORD_LENGTH} bytes/record");

    let mut m = Mod130::default();
    m.tipo_declaracion = "I".to_string();
    m.nif = "12345678Z".to_string();
    m.comienzo_apellido = "GARC".to_string();
    m.apellidos = "GARCIA LOPEZ".to_string();
    m.nombre = "JUAN".to_string();
    m.ejercicio = "2025".to_string();
    m.periodo = "1T".to_string();
    m.c01_ingresos = 1_000_000; // 10.000,00 €
    m.c02_gastos = 300_000; //  3.000,00 €
    m.persona_contacto = "JUAN GARCIA".to_string();
    m.telefono = "600000000".to_string();

    m.compute_derived();

    let diags = m.validate();
    for d in &diags {
        eprintln!("{:?}: {}", d.severity, d.message);
    }

    let bytes = m.marshal()?;
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)?;
    std::io::Write::write_all(&mut std::io::stdout(), b"\r\n")?;
    Ok(())
}
