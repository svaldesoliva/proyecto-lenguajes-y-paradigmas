// Módulo de procesamiento de audio usando la Transformada Rápida de Fourier (FFT).
// Expone una función pública (procesar_canales) que recibe muestras crudas de audio
// y devuelve una versión filtrada conservando solo las frecuencias más energéticas.

// FftPlanner: coordina la creación de los algoritmos FFT según el tamaño del buffer.
// Complex: número complejo (parte real + imaginaria), el formato que usa la FFT internamente.
use rustfft::{FftPlanner, num_complex::Complex};

// Recibe las muestras de un solo canal y devuelve ese canal reconstruido,
// conservando únicamente las `frecuencias_a_mantener` de mayor energía.
// El resto se pone a cero en el dominio de la frecuencia antes de invertir.
fn aplicar_filtro(muestras: &[f32], frecuencias_a_mantener: usize) -> Vec<f32> {
    // Número total de muestras; la FFT necesita saberlo para planificar su algoritmo
    let total_muestras = muestras.len();
    
    // El planificador decide internamente el algoritmo más eficiente para este tamaño
    let mut planificador = FftPlanner::new();
    
    // Transformada directa: pasa del dominio del tiempo al dominio de la frecuencia
    let transformada_directa = planificador.plan_fft_forward(total_muestras);
    
    // Transformada inversa: vuelve del dominio de la frecuencia al dominio del tiempo
    let transformada_inversa = planificador.plan_fft_inverse(total_muestras);

    // La FFT trabaja con números complejos; convertimos cada muestra real
    // a un número complejo con parte imaginaria en cero
    let mut espectro: Vec<Complex<f32>> = muestras
        .iter()
        .map(|&x| Complex { re: x, im: 0.0 })
        .collect();

    // Aplicamos la FFT: el buffer pasa de representar amplitudes en el tiempo
    // a representar amplitudes de cada frecuencia (el espectro)
    transformada_directa.process(&mut espectro);

    // Calculamos la energía (magnitud al cuadrado) de cada frecuencia del espectro.
    // Usamos norm_sqr() en vez de norm() porque es más rápido y nos sirve igual para comparar.
    let mut energias: Vec<f32> = espectro.iter().map(|c| c.norm_sqr()).collect();
    
    // Ordenamos de menor a mayor energía para poder ubicar el umbral fácilmente
    energias.sort_unstable_by(|a, b| a.total_cmp(b));
    
    // Determinamos el valor mínimo de energía que debe tener una frecuencia para no ser eliminada.
    // Como las energías están ordenadas de menor a mayor, retrocedemos `frecuencias_a_mantener`
    // posiciones desde el final para encontrar ese umbral.
    let umbral_energia = if frecuencias_a_mantener == 0 {
        // Si pedimos cero frecuencias, silenciamos todo
        f32::MAX
    } else if frecuencias_a_mantener >= total_muestras {
        // Si pedimos más frecuencias de las que hay, las conservamos todas
        0.0
    } else {
        energias[total_muestras - frecuencias_a_mantener]
    };

    // Silenciamos (ponemos a cero) todas las frecuencias cuya energía no alcance el umbral
    for componente in espectro.iter_mut() {
        if componente.norm_sqr() < umbral_energia {
            *componente = Complex { re: 0.0, im: 0.0 };
        }
    }

    // Invertimos la FFT para volver al dominio del tiempo
    transformada_inversa.process(&mut espectro);

    // La IFFT de rustfft no normaliza automáticamente: cada valor queda multiplicado por N.
    // Dividimos por el total de muestras para recuperar las amplitudes originales.
    let factor_normalizacion = 1.0 / (total_muestras as f32);
    espectro.iter().map(|c| c.re * factor_normalizacion).collect()
}

// Punto de entrada público del módulo.
// Separa los canales del audio entrelazado (L, R, L, R...), aplica el filtro FFT
// a cada canal de forma independiente y vuelve a entrelazarlos al final.
pub fn procesar_canales(muestras: &[f32], canales_totales: u16, frecuencias_a_mantener: usize) -> Vec<f32> {
    let num_canales = canales_totales as usize;
    
    // Cada posición del vector es un canal distinto; pre-asignamos su capacidad
    // para no hacer realocaciones innecesarias durante la separación
    let mut canales: Vec<Vec<f32>> = vec![Vec::with_capacity(muestras.len() / num_canales); num_canales];
    
    // El audio entrelazado viene como [L0, R0, L1, R1, ...].
    // Separamos cada muestra al canal que le corresponde según su posición
    for (indice, &muestra) in muestras.iter().enumerate() {
        canales[indice % num_canales].push(muestra);
    }
    
    // Filtramos cada canal por separado para no mezclar frecuencias entre ellos
    let canales_filtrados: Vec<Vec<f32>> = canales
        .iter()
        .map(|canal| aplicar_filtro(canal, frecuencias_a_mantener))
        .collect();
        
    // Volvemos a entrelazar los canales filtrados: [L0, R0, L1, R1, ...]
    let mut audio_entrelazado = Vec::with_capacity(muestras.len());
    let num_fotogramas = canales_filtrados[0].len(); // todos los canales tienen el mismo largo
    for indice_fotograma in 0..num_fotogramas {
        for canal in &canales_filtrados {
            audio_entrelazado.push(canal[indice_fotograma]);
        }
    }
    
    // Después del filtrado el volumen puede quedar muy bajo (muchas frecuencias silenciadas).
    // Normalizamos llevando la muestra de mayor amplitud absoluta a 30000, que está cerca
    // del techo de i16 (32767) pero deja un pequeño margen para evitar clipping.
    let amplitud_maxima = audio_entrelazado.iter().fold(0.0f32, |max, &x| max.max(x.abs()));
    if amplitud_maxima > 0.0 {
        let factor_escala = 30000.0 / amplitud_maxima;
        for muestra in audio_entrelazado.iter_mut() {
            *muestra *= factor_escala;
        }
    }

    audio_entrelazado
}
