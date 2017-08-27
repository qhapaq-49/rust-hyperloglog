extern crate rand;
use std::io;
use rand::Rng;
use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cmp::max;

trait HLLfunc{
    fn alpha(&self, mm: i32) -> f64;
    fn hash(&self, str: &String) -> u64;
    fn lower_bit(&self, x: u64) -> u64;
    fn upper_bit(&self, x: u64) -> u64;
    fn rho(&self, x: u64) -> i32;
    fn harmonic_mean(&self) -> f64;
    fn init(self : &mut Self, initb: i32);
    fn add(self : &mut Self, str: &String);
    fn estimate(&self) -> u64;
    fn dump_register_value(&self);
}

struct HyperLogLog{
    b: i32,
    m: i32,
    regs: Vec<i32>,
    s: BTreeSet<String>, // HyperLogLogがうまく行ったかのチェック用
}

impl HLLfunc for HyperLogLog{
    fn alpha(&self, mm: i32) -> f64 {
        match mm {
            16 => 0.673,
            32 => 0.697,
            64 => 0.709,
            128 | 256 | 512 | 1024 | 2048 | 4096 | 8192 | 16384 | 32768 => 0.7213 / (1.0 + 1.079 / (mm as f64)),
            _ => 0.0,
        }
    }

    fn hash(&self, str: &String) -> u64{
        let mut s = DefaultHasher::new();
        str.hash(&mut s);
        s.finish()     
    }

    fn upper_bit(&self, x:u64) -> u64{
        x >> self.b
    }

    fn lower_bit(&self, x:u64) -> u64{
        let mask: u64 = (1 << self.b) -1;
        x & mask
    }

    fn rho(&self, x:u64) -> i32 {
        for i in 0..32{
            if (x >> i) & 1 != 0{
               return i+1;
            }
        }
        33
    }
    
    fn harmonic_mean(&self) ->f64 {
        let mut ret :f64 = 0.0;
        for j in 1..self.m+1{
            ret += 1.0/2.0f64.powf(self.regs[j as usize] as f64);
        }
        1.0 / ret
    }

    fn init(self : &mut Self, initb: i32){
        self.b = initb;
        self.m = 2i32.pow(initb as u32);
        self.regs.clear();
        for _ in 0..self.m+1{
            self.regs.push(0);
        }
    }

    fn add(self : &mut Self, str: &String){
        self.s.insert(str.to_string());
        let x = self.hash(str);
        let j = 1 + self.lower_bit(x);
        let w = self.upper_bit(x);
        self.regs[j as usize] = max(self.regs[j as usize], self.rho(w));
    }

    fn estimate(&self) -> u64{
        let mut E = self.alpha(self.m) * self.harmonic_mean() * ((self.m * self.m) as f64);
        
        if E <= 2.5f64 * self.m as f64 {
            let mut V : i32 = 0;
            for i in 1..self.m {
                if self.regs[i as usize] == 0 {
                    V += 1;
                }
            }
            if V != 0 {
                E = self.m as f64 * (self.m as f64 / V as f64).ln();
            }
        }
        if E > 1.0/30.0 * 4294967296.0{
            E = -4294967296.0 * (1.0 - E/4294967296.0).log2();
        }
        println!("real,estimate = {},{}", self.s.len(), E);
        E as u64
    }

    fn dump_register_value(&self){
        let mut outstr : String = "".to_string();
        for j in 1..self.m+1 {
            outstr += &format!("{} ",self.regs[j as usize]); // int to strなどの変換はこうやるっぽい
        }
        println!("{}", outstr);
    }
}


fn randomstr(n: u32) -> String{
    let atoz = "abcdefghijklmnopqrstuvwxyz";
    let mut outstr = "".to_string();
    for _ in 0..n {
        let secret = rand::thread_rng().gen_range(0,26);
        outstr += &atoz[secret..secret+1];
    }
    /*
    // 一文字ずつ表現
    for i in 0..outstr.len() {
        println!("{}", outstr.chars().nth(i).unwrap());
    }
    */
    outstr
}

fn main() {
    println!("HLL by rust");
    println!("set the number of letter to generate");
    // assume m = 2^b, b in [4,16]
    let mut hlog = HyperLogLog {b:0, m:0, regs: Vec::new(), s:BTreeSet::new() };
    hlog.init(10);
    loop{
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("Failed to read");
        let guess: u32 = match guess.trim().parse(){
	    Ok(num) => num,
	    Err(_) => continue,
        };

        for i in 0..100000 {
            let inpstr = randomstr(guess);
            //println!("{}",inpstr);
            hlog.add(&inpstr);
            if i % 100 == 0 {
                hlog.estimate();
            }
        }
        
        break;
    }
}
