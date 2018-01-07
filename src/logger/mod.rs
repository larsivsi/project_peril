pub enum LogLevel{
    Trace, 
    Debug, 
    Info, 
    Warn, 
    Error, 
}

pub struct Logger{
    log_level: LogLevel,
}

///TODO: make this work so that message is a formatted thing that takes arguments (fmt)
fn print_log (head:&str, message:String){
    print!("{}", head);
    println!("{}", message);
}

impl Logger{
    pub fn set_log_level(&mut self, log_level:LogLevel) {
        self.log_level = log_level;
    }
    
    pub fn print(self, message:String) {
        match self.log_level {
            Trace => print_log("TRACE", message)
        }
    }

    ///TODO: Find some way to set default level.
    pub fn init(log_level:LogLevel) -> Logger{
        Logger {
            log_level: LogLevel::Trace,
        }
    }
}
