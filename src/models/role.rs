use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Role {
    Master,
    Slave,
}
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Role::Master => write!(f, "master"),
            Role::Slave => write!(f, "slave"),
        }
    }
}

impl Role {
    pub fn is_master(&self) -> bool {
        match self {
            Role::Master => true,
            Role::Slave => false,
        }
    }
}
