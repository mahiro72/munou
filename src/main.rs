use std::collections::HashMap;
use regex::Regex;
use vibrato::{Dictionary, Tokenizer};
use std::fs;
use std::io::{self, Write};
use lazyrand;

// 特殊な単語ID
static TOP_WORD_ID: isize = 0;
static END_WORD_ID: isize = 1;

// 人工無脳の構造体
struct MarkovChain {
    words: Vec<String>, // 単語のリスト
    word_hash: HashMap<String, isize>, // 単語とIDの対応表
    chain: HashMap<(isize, isize), Vec<isize>>, // マルコフ連鎖の辞書
    tokenizer: Tokenizer, // 形態素解析器
}

impl MarkovChain {
    pub fn new() -> Self {
        // 形態素解析の辞書を読み込む
        let reader = zstd::Decoder::new(fs::File::open("system.dic.zst").unwrap()).unwrap();
        let dict = Dictionary::read(reader).unwrap();

        // 単語辞書の初期化
        let mut word_hash = HashMap::new();
        word_hash.insert("★".to_string(), TOP_WORD_ID);
        word_hash.insert("。".to_string(), END_WORD_ID);

        let mut words = Vec::new();
        words.push("★".to_string());
        words.push("。".to_string());
        MarkovChain{
            words,
            word_hash,
            chain: HashMap::new(),
            tokenizer: Tokenizer::new(dict),
        }
    }

    // 単語をIDを取得する
    pub fn get_word_id(&mut self, word: &str) -> isize { // TODO: &mut selfというキーワードはなに? &はポインタ?
        if let Some(&id) = self.word_hash.get(word) { // TODO &idはどこからでてきた?
            return id;
        }
        let id = self.words.len() as isize; // TODO: なぜ as isize?
        self.word_hash.insert(word.to_string(), id); // TODO: word_hash: HashMap<String, isize, RandomState>のRandomStateは何?
        self.words.push(word.to_string());
        id
    }

    // 文章を形態素解析で分割する
    fn split(&self, text: &str) -> Vec<String> {
        let mut worker = self.tokenizer.new_worker();
        worker.reset_sentence(text);
        worker.tokenize();
        worker.token_iter().map(|t| t.surface().to_string()).collect()
    }

    // マルコフ連鎖の辞書を作成する
    pub fn train(&mut self, text: &str) {
        // 正規表現で不要な文字の削除
        let re = Regex::new(r"(《.*?》|［.*?］|[｜\s\u{3000}\-]|[「」『』])").unwrap();
        let clean_text = re.replace_all(text, "");
        // 形態素解析
        let words: Vec<String> = self.split(&clean_text);
        // 単語IDのリストを作成
        let word_ids: Vec<isize> = words.iter().map(|w| self.get_word_id(w)).collect();
        // マルコフ連鎖の辞書を作成
        let mut tmp = vec![TOP_WORD_ID, TOP_WORD_ID];
        for word_id in word_ids {
            tmp.push(word_id);
            if tmp.len() < 3 { continue; }
            if tmp.len() > 4 {tmp.remove(0);}
            let w = tmp[2];
            let key = (tmp[0], tmp[1]);
            if self.chain.contains_key(&key) {
                self.chain.get_mut(&key).unwrap().push(w);
            } else {
                self.chain.insert(key, vec![w]);
            }
            if w == END_WORD_ID {
                tmp.clear();
                tmp.push(TOP_WORD_ID);
            }
        }
    }

    // 文章を生成する
    pub fn generate(&mut self) -> String {
        self.generate_text(TOP_WORD_ID, TOP_WORD_ID)
    }

    // 次の単語IDを生成する
    fn generate_next_id(&self, w1: isize, w2: isize) -> isize {
        let w_ids = match self.chain.get(&(w1, w2)) {
            Some(w_ids) => w_ids,
            None => return END_WORD_ID,
        };
        if w_ids.is_empty() { return END_WORD_ID; }
        lazyrand::choice(w_ids).unwrap()
    }

    // w1, w2に続く文章を生成する
    pub fn generate_text(&self, w1: isize, w2: isize) -> String {
        let mut result = String::new();
        let mut w1 = w1;
        let mut w2 = w2;
        result.push_str(self.words[w1 as usize].as_str());
        result.push_str(self.words[w2 as usize].as_str());
        let mut w3;
        loop {
            w3 = self.generate_next_id(w1, w2);
            result.push_str(self.words[w3 as usize].as_str());
            if w3 == END_WORD_ID {
                break;
            }
            w1 = w2;
            w2 = w3;
        }
        result.replace("★", "")
    }
}

fn main() {
    // 人工無能を初期化
    let mut markov = MarkovChain::new();
    // テキストを読み込んで学習
    let text = fs::read_to_string("wagahaiwa_nekodearu.txt").unwrap();
    let lines: Vec<&str> = text.split("\n").collect();
    for line in lines {
        markov.train(line);
    }
    println!("{}", markov.generate());

    // 対話モード
    println!(">>> 終了するにはEnterキーを押してください。");
    loop {
        io::stdout().flush().unwrap();
        let mut input = String::new();
        println!(">>> 何か話しかけてください。");
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().is_empty() {
            break;
        }
        // 入力した内容から適当に名刺を抽出して、それを含む文章の生成
        let mut worker = markov.tokenizer.new_worker();
        worker.reset_sentence(input);
        worker.tokenize();
        let words: Vec<String> = worker.token_iter().map(|t| {
            if t.feature().contains("名刺")  {
                t.surface().to_string()
            } else {
                "".to_string()
            }
        }).filter(|w| !w.is_empty()).collect();
        let output = if words.len() == 0 {
            markov.generate()
        } else {
            let word: String = lazyrand::choice(&words).unwrap();
            let word_id = markov.get_word_id(&word);
            let text = markov.generate_text(TOP_WORD_ID, word_id);
            if text.len() < word.len() + 4 {
                markov.generate()
            } else {
                text
            }
        };
        println!("我輩🐱> {}", output);
    }
}
