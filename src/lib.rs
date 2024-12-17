pub mod worm;

#[derive(Debug, PartialEq, Clone)]
pub struct HostSpec {
    pub host_addr: String,
    pub username: String,
}

impl From<&str> for HostSpec {
    fn from(spec: &str) -> HostSpec {
        if let Some(sep_i) = spec.find('@') {
            let host_addr = spec[(sep_i + 1)..].to_owned();
            let username = spec[..sep_i].to_owned();
            HostSpec {
                host_addr,
                username,
            }
        } else {
            HostSpec {
                host_addr: spec.to_owned(),
                username: users::get_current_username()
                    .expect("No username specified and unable to retrieve the current user's name")
                    .to_string_lossy()
                    .to_string(),
            }
        }
    }
}
