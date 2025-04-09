use std::{fs::File, io::Read, path::Path};

use rand::{rng, rngs::ThreadRng, Rng};


/// A mask used for tree placement loaded from a raw file
pub struct Mask(Vec<f32>, usize);
impl Mask {
    pub fn from_file<T: AsRef<Path>>(path: T, size: usize) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let data: &[f32] = bytemuck::cast_slice(data.as_slice());
        Ok(Self(data.to_owned(), size))
    }
    pub fn sample_at(&self, x: f32, y: f32) -> f32 {
        let width = self.1 as f32;
        let ix = (x * width) as usize;
        let iy = (y * width) as usize;
        debug_assert!(iy * self.1 + ix < self.0.len(), "index ({x},{y}) out of bounds ({ix}, {iy})->({})", iy * self.1 + ix);
        self.0[iy * self.1 + ix]
    }
}


/// a bluenoise generator that works on a mask
/// 
/// this allows random noise with minimum distance whilst also following a mask
pub struct BlueNoise<'a>(fast_poisson::Iter<2>, &'a Mask);
impl<'a> BlueNoise<'a>{
    pub fn new(distance: f32, mask: &'a Mask) -> Self {
        let iter = fast_poisson::Poisson::<2>::new()
            .with_dimensions([1.0, 1.0], distance as f64);   
        Self(iter.iter(), mask)
    }
}
impl Iterator for BlueNoise<'_> {
    type Item = [f32; 2];
    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rng();
        loop {
            let x = self.0.next()?;
            let sample_point = self.1.sample_at(x[0] as f32, x[1] as f32);
            if rng.random_range(0.00 .. 1.0) < sample_point {
                return Some([x[0] as f32, x[1] as f32])
            }
        }
    }
}