use std::collections::HashMap;
use std::ops::{Add, Mul, Div};
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Costs {
    labor: f64,
    raw: HashMap<String, f64>,
}

