pub fn hash(pwd: String) -> Vec<u8> { 
    pwd.as_bytes().to_owned() 
}

pub fn verify(pwd: String) -> () {
    ()
}
