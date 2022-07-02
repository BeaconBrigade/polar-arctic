use super::arctic;

pub fn view(err: arctic::Error) -> String {
    match err {
        arctic::Error::Dumb => "This is so annoying",
        arctic::Error::Stupid => "You are very stupid",
    }.to_string()
}
