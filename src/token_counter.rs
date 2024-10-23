pub struct TokenCounter {
    count: i32
}

impl TokenCounter {
    pub fn new() -> TokenCounter {
        TokenCounter{
            count:0
        }
    }
    pub fn next(&mut self) -> i32 {
        self.count+=1;
        self.count
    }
}