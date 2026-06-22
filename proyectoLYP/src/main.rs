// Proyecto Lenguajes y Paradigmas
// Procesamiento de audio mediante Transformada de Fourier (FFT)
// Funcional + Imperativo + Modular

mod audio;

use std::env;
use hound::{WavReader, WavWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Lectura de argumentos de la terminal
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Uso: {} <archivo.wav>", args[0]);
        std::process::exit(1);
    }
    
    let archivo_entrada = &args[1];

    if !std::path::Path::new(archivo_entrada).exists() {
        eprintln!("Error: No se encontró el archivo '{}'.", archivo_entrada);
        std::process::exit(1);
    }

    println!("Cargando archivo {}...", archivo_entrada);
    
    let mut reader = WavReader::open(archivo_entrada)?;
    let spec = reader.spec();
    
    // Leemos los valores del audio original (están en enteros i16) 
    // y los transformamos a f32 (decimales) para poder procesarlos.
    let samples: Vec<f32> = reader.samples::<i16>()
        .map(|s| s.map(|val| val as f32))
        .collect::<Result<Vec<f32>, _>>()?;

    let frecuencias = [5, 500, 1500, 5000, 15000, 30000, 50000, 100000, 300000, 1000000];

    for &i in &frecuencias {
        println!("Recreando audio conservando sólo el top {} de frecuencias...", i);
        
        let reconstruido = audio::procesar_canales(&samples, spec.channels, i);
        
        let salida = format!("output_top_{}.wav", i);

        // escribir archivo con wavwriter
        let mut writer = WavWriter::create(&salida, spec)?;
        for &sample in &reconstruido {
            writer.write_sample(sample.round() as i16)?;
        }
        writer.finalize()?;
        
        println!("Guardado: {}", salida);
    }
    
    println!("Terminado.");
    Ok(())
}
