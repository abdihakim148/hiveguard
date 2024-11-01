use argon2::{Params as ArgonParams, Error};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Params {
    memory: u32,
    iteration: u32,
    parallelism: u32,
    output_length: Option<usize>
}


impl TryFrom<Params> for ArgonParams {
    type Error = Error;
    fn try_from(params: Params) -> Result<Self, Self::Error> {
        let (m_cost, t_cost, p_cost, output_len) = (params.memory, params.iteration, params.parallelism, params.output_length);
        ArgonParams::new(m_cost, t_cost, p_cost, output_len)
    }
}

impl From<ArgonParams> for Params {
    fn from(value: ArgonParams) -> Self {
        let (memory, iteration, parallelism, output_length) = (value.m_cost(), value.t_cost(), value.p_cost(), value.output_len());
        Self{memory, iteration, parallelism, output_length}
    }
}

impl Default for Params {
    fn default() -> Self {
        ArgonParams::default().into()
    }
}


impl PartialEq<ArgonParams> for Params {
    fn eq(&self, other: &ArgonParams) -> bool {
        self.memory == other.m_cost() && self.iteration == other.t_cost() && self.parallelism == other.p_cost() && self.output_length == other.output_len()
    }
}