use std::collections::HashSet;
use rand::Rng;
use serde::Serialize;

const COLORS: u8 = 2;

fn random_color(rng: &mut impl Rng) -> u8 {
    rng.gen_range(1..=COLORS)
}

fn permute_color(color: u8, rng: &mut impl Rng) -> u8 {
    (color + rng.gen_range(1..=COLORS)) % (COLORS + 1)
}

fn gen_field(size: usize) -> Vec<u8> {
    (0..size).map(|_| 0).collect()
}

fn write_block(pos: usize, size: usize, color: u8, mut field: Vec<u8>) -> Vec<u8> {
    for i in 0..size {
        field[pos + i] = color;
    }
    field
}

fn print(x: &[u8], name: &str) {
    print!("{name}: ");
    for i in x {
        print!("{i}");
    }
    println!();
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Default, Hash, Eq)]
struct Example {
    input: Vec<u8>,
    output: Vec<u8>,
    wrong: Vec<Vec<u8>>,
}

fn remove_duplicates_examples(examples: &mut Vec<Example>) {
    let mut seen = HashSet::new();
    examples.retain(|example| seen.insert(example.clone()));
}

fn remove_duplicates(example: &mut Example) {
    let mut seen = HashSet::new();
    example.wrong.retain(|vector| seen.insert(vector.clone()));
}

fn remove_wrong_wrongs(example: &mut Example) {
    example.wrong.retain(|vector| *vector != example.input);
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Default)]
struct Example2D {
    input: Vec<Vec<u8>>,
    output: Vec<Vec<u8>>,
    wrong: Vec<Vec<u8>>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Default)]
struct ArcTask2D {
    train: Vec<Example2D>,
    test: Vec<Example2D>,
}

impl From<Example> for Example2D {
    fn from(x: Example) -> Self {
        Example2D {
            input: vec![x.input],
            output: vec![x.output],
            wrong: x.wrong,
        }
    }
}

pub fn save_json_to_file<T: Serialize>(t: &T, name: &str) {
    use std::io::Write;
    let mut file = std::fs::File::create(name).unwrap();
    let json = serde_json::to_string(&t).unwrap();
    write!(file, "{}", json).unwrap();
}

fn main() {
    let mut rng = rand::thread_rng();
    let size = 12;

    let mut examples = vec![];
    for _ in 0..1000000 {
        let block_size = rng.gen_range(1..size - 1); // -1 is for moving left
        let block_pos = rng.gen_range(0..=size - block_size - 1); // -1 is for moving left
        let color = random_color(&mut rng);

        let question = write_block(block_pos, block_size, color, gen_field(size));
        let answer = write_block(block_pos+1, block_size, color, gen_field(size));
        // print(&question, "qu");
        // print(&answer, "an");

        let mut wrongs = vec![];

        if block_pos+2+block_size < size {
            let wrong1 = write_block(block_pos+2, block_size, color, gen_field(size));
            // print(&wrong1, "w1");
            wrongs.push(wrong1);
        }

        if block_pos > 0 {
            let wrong2 = write_block(block_pos-1, block_size, color, gen_field(size));
            // print(&wrong2, "w2");
            wrongs.push(wrong2);
        }

        let mut wrong3 = question.clone();
        wrong3[block_pos+block_size] = color;
        wrong3[block_pos+block_size-1] = 0;
        // print(&wrong3, "w3");
        wrongs.push(wrong3);

        let wrong4 = write_block(block_pos+2, block_size-1, color, gen_field(size));
        // print(&wrong4, "w4");
        wrongs.push(wrong4);

        let wrong5 = write_block(block_pos+1, block_size-1, color, gen_field(size));
        // print(&wrong5, "w5");
        wrongs.push(wrong5);

        let wrong6 = write_block(block_pos+1, block_size, permute_color(color, &mut rng), gen_field(size));
        // print(&wrong6, "w6");
        wrongs.push(wrong6);

        let wrong7 = write_block(block_pos, block_size+1, color, gen_field(size));
        // print(&wrong7, "w7");
        wrongs.push(wrong7);

        let mut example = Example {
            input: question,
            output: answer,
            wrong: wrongs,
        };
        remove_duplicates(&mut example);
        remove_wrong_wrongs(&mut example);

        examples.push(example);
        
        // println!();
    }

    remove_duplicates_examples(&mut examples);
    examples.sort_by_key(|x| x.input.clone());

    dbg!(examples.len());

    for (i, example) in examples.into_iter().enumerate() {
        let task = ArcTask2D {
            train: vec![],
            test: vec![example.into()],
        };

        save_json_to_file(&task, &format!("tasks/1d_move_1_pix_{i}.json"));
    }
}
