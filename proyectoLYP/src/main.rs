// Proyecto Lenguajes y Paradigmas
// Procesamiento de audio mediante Transformada de Fourier (FFT).
// Este programa toma un archivo WAV y genera versiones comprimidas conservando
// solo las N frecuencias de mayor energía y descartando las demás.
// Demuestra un enfoque funcional (iteradores, map/collect) combinado con imperativo (loops de escritura).

mod audio;

use std::env;
use hound::{WavReader, WavWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Argumentos que el usuario pasó por la terminal (el primero siempre es el nombre del programa)
    let argumentos: Vec<String> = env::args().collect();
    
    if argumentos.len() < 2 {
        eprintln!("Uso: {} <archivo.wav>", argumentos[0]);
        std::process::exit(1);
    }
    
    // Ruta del archivo WAV que queremos procesar
    let archivo_entrada = &argumentos[1];

    if !std::path::Path::new(archivo_entrada).exists() {
        eprintln!("Error: No se encontró el archivo '{}'.", archivo_entrada);
        std::process::exit(1);
    }

    println!("Cargando archivo {}...", archivo_entrada);
    
    // Lector del archivo WAV; nos da acceso a las muestras y a la configuración del audio
    let mut lector = WavReader::open(archivo_entrada)?;
    
    // Configuración del audio original: sample rate, número de canales, bits por muestra, etc.
    // La guardamos para poder escribir los archivos de salida con las mismas propiedades.
    let config_audio = lector.spec();
    
    // Las muestras originales vienen en enteros i16 (rango -32768 a 32767).
    // Las convertimos a f32 para poder hacer operaciones matemáticas (FFT, normalización)
    // sin perder precisión en los cálculos intermedios.
    let muestras: Vec<f32> = lector.samples::<i16>()
        .map(|s| s.map(|valor| valor as f32))
        .collect::<Result<Vec<f32>, _>>()?;

    // Distintos niveles de compresión que vamos a probar.
    // Cada número indica cuántas frecuencias conservamos
    let niveles_compresion = [5, 500, 1500, 5000, 15000, 30000, 50000, 100000, 300000, 1000000];

    for &n_frecuencias in &niveles_compresion {
        println!("Recreando audio conservando sólo el top {} de frecuencias...", n_frecuencias);
        
        // Audio reconstruido tras aplicar el filtro FFT con este nivel de compresión
        let audio_reconstruido = audio::procesar_canales(&muestras, config_audio.channels, n_frecuencias);
        
        // Nombre del archivo de salida para este nivel de compresión
        let nombre_salida = format!("output_top_{}.wav", n_frecuencias);

        // Escribimos el resultado en un nuevo WAV con la misma configuración que el original
        // (mismo sample rate, mismos canales) para que sea reproducible sin cambios.
        let mut escritor = WavWriter::create(&nombre_salida, config_audio)?;
        for &muestra in &audio_reconstruido {
            // Redondeamos de vuelta a i16 antes de escribir, ya que el formato WAV lo requiere
            escritor.write_sample(muestra.round() as i16)?;
        }
        escritor.finalize()?;
        
        println!("Guardado: {}", nombre_salida);
    }
    
    println!("Terminado.");
    Ok(())
}
