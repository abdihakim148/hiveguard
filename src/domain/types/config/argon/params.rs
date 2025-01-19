use serde::{Serialize, Deserialize};
use argon2::Params as ArgonParams;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Params {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub output_length: Option<usize>
}


impl Default for Params {
    fn default() -> Self {
        let memory_cost = ArgonParams::DEFAULT_M_COST;
        let time_cost = ArgonParams::DEFAULT_T_COST;
        let parallelism = ArgonParams::DEFAULT_P_COST;
        let output_length = None;
        Self{memory_cost, time_cost, parallelism, output_length}
    }
}


impl From<Params> for ArgonParams {
    fn from(params: Params) -> Self {
        let m_cost = params.memory_cost;
        let t_cost = params.time_cost;
        let p_cost = params.parallelism;
        let output_len = params.output_length;
        ArgonParams::new(m_cost, t_cost, p_cost, output_len).unwrap_or_default()
    }
}