use std::env;

pub fn parse_ip(ip: &str) -> Result<[u8; 4], String> {
    let parts: Vec<&str> = ip.split('.').collect(); //make input string into string array demilited by .
    if parts.len() != 4 { //throw error if not right size
        return Err(format!("Invalid IP '{}': must contain 4 octets", ip));
    }
    let mut octets = [0u8; 4]; //make fixed size array of 4 bytes initialized to 00000000
    for (i, part) in parts.iter().enumerate() {
        let value = part
            .parse::<u8>() //attempts to convert each octet into unsigned 8 bit integer
            .map_err(|_| format!("Invalid octet '{}' in IP '{}'", part, ip))?;
        octets[i] = value; //insert octet into octets array
    }
    Ok(octets) //final result like [192, 168, 0, 1]
}

#[derive(Debug, PartialEq)]
// command line structure that will be passed to function 
pub struct Args {
    pub data_file: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub output_file: String,
}

pub fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 7 { //check correct # of arguments. 
        return Err(format!(
            "Expected 6 arguments, got {}",
            args.len() - 1
        ));
    }
    //error handle if can't convert to u16 
    let src_port = args[4].parse::<u16>().map_err(|_| "Invalid source port".to_string())?; 
    let dst_port = args[5].parse::<u16>().map_err(|_| "Invalid destination port".to_string())?;

    Ok(Args {
        data_file: args[1].clone(), //.clone() gives owned string to struct
        src_ip: args[2].clone(),
        dst_ip: args[3].clone(),
        src_port,
        dst_port,
        output_file: args[6].clone(),
    })
}

