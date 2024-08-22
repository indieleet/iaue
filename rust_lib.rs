#[no_mangle] pub extern "C" fn f0(f: f32, l: f32, v: f32, t: usize, p: &[f32]) -> Vec<f32> {
                         let freq = t as f32 / f;
                         let length = l * t as f32;
                         (0..freq as usize)
                         .map(|it| it as f32 / f - 0.5)
                         .map(|it| it * v)
                         .cycle()
                         .take(length as usize)
                         .collect()
                    }
                    