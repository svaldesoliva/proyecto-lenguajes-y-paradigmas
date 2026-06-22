// Funciones para procesar audio


use rustfft::{FftPlanner, num_complex::Complex}; //librería fast fourier transformation y numeros complejos

// Aplica compresión aislando el Top N de frecuencias de mayor energía.
// Es una función privada de este módulo (sin pub).
fn aplicar_filtro(samples: &[f32], frecuencias_a_mantener: usize) -> Vec<f32> {
    let n = samples.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let ifft = planner.plan_fft_inverse(n);

    // Transformación inicial a números complejos
    let mut buffer: Vec<Complex<f32>> = samples
        .iter()
        .map(|&x| Complex { re: x, im: 0.0 })
        .collect();

    fft.process(&mut buffer); //fast fourier transformation

    // Extracción de magnitudes
    let mut magnitudes: Vec<f32> = buffer.iter().map(|c| c.norm_sqr()).collect();
    
    //ordenar 
    magnitudes.sort_unstable_by(|a, b| a.total_cmp(b));
    
    // Determinamos el valor mínimo de energía que debe tener una frecuencia para no ser eliminada.
    // Como las magnitudes están ordenadas de menor a mayor, retrocedemos `frecuencias_a_mantener` posiciones
    // desde el final (las de mayor energía) para encontrar ese umbral.
    let umbral_energia = if frecuencias_a_mantener == 0 {
        f32::MAX
    } else if frecuencias_a_mantener >= n {
        0.0
    } else {
        magnitudes[n - frecuencias_a_mantener]
    };

    // Silenciamos las frecuencias cuya energía sea menor al umbral calculado
    for c in buffer.iter_mut() {
        if c.norm_sqr() < umbral_energia {
            *c = Complex { re: 0.0, im: 0.0 };
        }
    }

    ifft.process(&mut buffer);

    // Normalización
    let factor = 1.0 / (n as f32);
    buffer.iter().map(|c| c.re * factor).collect()
}

// separa canales estéreo, filtra de forma separada y despues los junta.
// Función pública expuesta para ser usada desde main.rs
pub fn procesar_canales(samples: &[f32], channels: u16, frecuencias_a_mantener: usize) -> Vec<f32> {
    let num_canales = channels as usize;
    let mut canales: Vec<Vec<f32>> = vec![Vec::with_capacity(samples.len() / num_canales); num_canales];
    
    // Separación de canales
    for (i, &sample) in samples.iter().enumerate() {
        canales[i % num_canales].push(sample);
    }
    
    // Filtrado independiente
    let canales_filtrados: Vec<Vec<f32>> = canales
        .iter()
        .map(|canal| aplicar_filtro(canal, frecuencias_a_mantener))
        .collect();
        
    // entrelazado
    let mut reconstruido = Vec::with_capacity(samples.len());
    let frames = canales_filtrados[0].len();
    for i in 0..frames {
        for canal in &canales_filtrados {
            reconstruido.push(canal[i]);
        }
    }
    
    // Normalización de volumen para evitar silencios
    let max_val = reconstruido.iter().fold(0.0f32, |max, &x| max.max(x.abs()));
    if max_val > 0.0 {
        let factor_escala = 30000.0 / max_val;
        for sample in reconstruido.iter_mut() {
            *sample *= factor_escala;
        }
    }

    reconstruido
}
