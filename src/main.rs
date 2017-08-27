extern crate rand;
use std::io;
use rand::Rng;
use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cmp::max;

// 基本的に http://d.hatena.ne.jp/jetbead/20130222/1361603458 の写経
// 理論解説は http://blog.brainpad.co.jp/entry/2016/06/27/110000 が解りやすい

// structのメンバ関数にしてprivate publicを切り分けたほうが良い気がする
// デバッグでprivateな方が良さ気なメソッド(rhoとか)も呼びたかったので妥協
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
        // 最初に1が立つのが何ビット目かを返す。例 1->1、2->2、3->1
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
        // 文字列からハッシュを生成し、下位10（mainで決めた値）ビットをアドレスにする
        self.s.insert(str.to_string());
        let x = self.hash(str);
        let j = 1 + self.lower_bit(x);
        let w = self.upper_bit(x);
        self.regs[j as usize] = max(self.regs[j as usize], self.rho(w));
    }

    fn estimate(&self) -> u64{
        let mut e = self.alpha(self.m) * self.harmonic_mean() * ((self.m * self.m) as f64);
        
        if e <= 2.5f64 * self.m as f64 {
            let mut v : i32 = 0;
            for i in 1..self.m {
                if self.regs[i as usize] == 0 {
                    v += 1;
                }
            }
            if v != 0 {
                e = self.m as f64 * (self.m as f64 / v as f64).ln();
            }
        }
        if e > 1.0/30.0 * 4294967296.0{
            e = -4294967296.0 * (1.0 - e/4294967296.0).log2();
        }
        println!("real,estimate = {},{}", self.s.len(), e);
        e as u64
    }

    fn dump_register_value(&self){
        // レジスタの値を書き出す。デバッグ用
        println!("value of register (regs[j])");
        let mut outstr : String = "".to_string();
        for j in 1..self.m+1 {
            // int to strなどの変換はこうやるっぽい
            outstr += &format!("{} ",self.regs[j as usize]); 
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
    // 一文字ずつ表現（今回は使わなかったけど、ハマる要素なのでメモ）
    for i in 0..outstr.len() {
        println!("{}", outstr.chars().nth(i).unwrap());
    }
    */
    outstr
}

fn main() {
    println!("HLL by rust");
    println!("set the number of letter to generate");

    let mut hlog = HyperLogLog {b:0, m:0, regs: Vec::new(), s:BTreeSet::new() };
    // レジスタのサイズ。この場合は2^10になる
    hlog.init(10);
    loop{
        // 標準入力から何文字の文字列を創るか決定する。
        // https://rust-lang-ja.github.io/the-rust-programming-language-ja/1.6/book/guessing-game.html を参照
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("Failed to read");
        let guess: u32 = match guess.trim().parse(){
	    Ok(num) => num,
	    Err(_) => continue,
        };

        for i in 0..100000 {
            let inpstr = randomstr(guess);
            // randomstrでguess文字の文字列生成、addでhyperloglogに追加、estimateで数える
            hlog.add(&inpstr);
            if i % 100 == 0 {
                hlog.estimate();
            }
        }
        // 最後にレジスタの中身を見てみる
        hlog.dump_register_value();
        
        break;
    }
}
