
pub struct Shader {
    pub s_type: String,
    pub parameters: Vec<Parameter>,
    pub proxies: Vec<Proxy>
}

pub struct Proxy {
    pub r_type: String,
    pub parameters: Vec<Parameter>
}

pub struct Parameter {
    pub p_type: String,
    pub value: String
}