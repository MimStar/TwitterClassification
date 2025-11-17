use std::fs;
use csv::Error;

pub fn words_dictionnary_to_reg(path: &str) -> Result<Vec<String>, Error> {
    /*
    let f = File::open(path)?;
    
    TO DO, build list word per word instead of dumping then processing
    
    let mut rdr = BufReader::new(f);
    let mut re_string: String = "".to_string();

    let cursor = io::Cursor::new(b"lorem-ipsum-dolor");

    let mut buf= Vec::with_capacity(15);

    let mut list = Vec::new();
    while rdr.read_until(b',', &mut buf)? > 0 {
        //println!("{:?}", buf);
        let test = str::from_utf8(&buf).unwrap();
        //println!("{:?}", test);

        //re_string += str::from_utf8(&buf).unwrap();
        //println!("{:?}\n\n", re_string);
        //re_string += "|";
        //buf= Vec::with_capacity(15);
        buf.clear();
        list.push(test.to_string());
    }*/
    
    let dump = fs::read_to_string(path)?;
    let mut list = Vec::new();
    dump.split(",").for_each(|word| list.push(word.trim().to_string()));
    println!("{:?}", list);
    Ok(list)
}


/*
pub fn words_dictionnary_to_vec(path: String, content_column: usize, polarity_column: usize) -> Result<Vec<String>, Error> {
    if let Ok(mut rdr) = Reader::from_path(path)
    && let Ok(mut wtr) = Writer::from_path(path) {
        rdr.records().into_iter().for_each(move |record| {
            let content =  record?.get(content_column)?;
            let re = Regex::new()
        });
    }
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}*/