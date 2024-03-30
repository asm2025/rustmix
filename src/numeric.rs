use rand::{thread_rng, Rng};

pub fn random() -> f64 {
    let mut rng = thread_rng();
    rng.gen_range(0.0..1.0)
}
