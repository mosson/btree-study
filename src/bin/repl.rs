use std::io::{BufRead, Write};

use btree_study::BTree;

// macro_rules! r {
//     ($buf:expr) => {
//         {
//             let mut iter = $buf.split_whitespace();

//             (
//                 iter
//                     .next()

//             )
//         }
//     };
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut btree: BTree<isize, 2> = BTree::new();
    let mut reader = std::io::BufReader::new(std::io::stdin());

    loop {
        print!("[insert [N] | delete [N] | inspect | quit] > ");
        std::io::stdout().flush()?;

        let mut buf = String::new();
        reader.read_line(&mut buf).map_err(|e| e.to_string())?;
        let mut iter = buf.split_whitespace();

        match iter.next().unwrap_or("") {
            "insert" | "add" => {
                match iter.next().unwrap_or("").parse::<isize>() {
                    Ok(value) => {
                        btree.insert(value);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                };
                println!("{:#?}", btree);
            }
            "delete" | "del" | "remove" => {
                match iter.next().unwrap_or("").parse::<isize>() {
                    Ok(value) => match btree.delete(&value) {
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                        _ => {}
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                };
                println!("{:#?}", btree);
            }
            "inspect" | "i" => {
                println!("{:#?}", btree);
            }
            "q" | "quit" => break,
            cmd => {
                println!("Unknown command: {}", cmd);
            }
        }
    }

    Ok(())
}
