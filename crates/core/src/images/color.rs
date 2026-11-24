//! Color dominante de una portada: histograma de 4 bits por canal sobre una miniatura,
//! dando más peso a los colores saturados y descartando casi-negros y casi-blancos
//! (bordes, texto), que no sirven como acento.

use crate::Result;
use std::path::Path;

const SAMPLE_SIZE: u32 = 48;

pub fn dominant_color(path: &Path) -> Result<String> {
    let thumb = image::open(path)?.thumbnail(SAMPLE_SIZE, SAMPLE_SIZE).to_rgb8();
    let pixels: Vec<[u8; 3]> = thumb.pixels().map(|p| p.0).collect();
    Ok(to_hex(dominant(&pixels)))
}

fn dominant(pixels: &[[u8; 3]]) -> [u8; 3] {
    let mut weight = vec![0f64; 4096];
    let mut sum = vec![[0f64; 3]; 4096];
    for &[r, g, b] in pixels {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        if max < 24 || min > 232 {
            continue;
        }
        let saturation = f64::from(max - min) / f64::from(max);
        let w = 0.2 + saturation;
        let bucket = (usize::from(r >> 4) << 8) | (usize::from(g >> 4) << 4) | usize::from(b >> 4);
        weight[bucket] += w;
        for (acc, c) in sum[bucket].iter_mut().zip([r, g, b]) {
            *acc += w * f64::from(c);
        }
    }
    let Some((best, &w)) = weight.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)) else {
        return [0, 0, 0];
    };
    if w == 0.0 {
        return average(pixels);
    }
    sum[best].map(|c| (c / w).round() as u8)
}

/// Imagen sin colores útiles (todo negro o todo blanco): su media.
fn average(pixels: &[[u8; 3]]) -> [u8; 3] {
    if pixels.is_empty() {
        return [0, 0, 0];
    }
    let n = pixels.len() as u64;
    let total = pixels.iter().fold([0u64; 3], |acc, p| [acc[0] + u64::from(p[0]), acc[1] + u64::from(p[1]), acc[2] + u64::from(p[2])]);
    total.map(|c| (c / n) as u8)
}

fn to_hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturated_color_beats_larger_gray_and_black_areas() {
        let mut pixels = vec![[0, 0, 0]; 500];
        pixels.extend(vec![[128, 128, 128]; 120]);
        pixels.extend(vec![[200, 40, 30]; 100]);
        assert_eq!(dominant(&pixels), [200, 40, 30]);
    }

    #[test]
    fn all_black_image_falls_back_to_average() {
        assert_eq!(dominant(&[[0, 0, 0], [10, 10, 10]]), [5, 5, 5]);
    }

    #[test]
    fn reads_a_real_png() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.png");
        image::RgbImage::from_pixel(8, 8, image::Rgb([30, 90, 200])).save(&file).unwrap();
        assert_eq!(dominant_color(&file).unwrap(), "#1e5ac8");
    }
}
