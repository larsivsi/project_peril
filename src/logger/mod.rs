pub enum LogLevel{
    Trace = 0, 
    Debug = 1, 
    Info = 2, 
    Warn = 3, 
    Error = 4, 
}

pub struct Logger{
    log_name: String,
    log_level: LogLevel,
}


impl Logger{
    pub fn set_log_level(&mut self, log_level:LogLevel) {
        self.log_level = log_level;
    }

    ///TODO: make this work so that message is formatted and takes arguments (fmt)
    fn print_log (&self, head:&str, message:&str){
        print!("[{}] ", self.log_name);
        print!("[{}] ", head);
        println!("- {}", message);
    }
    
    pub fn trace(&self, message:&str) {
        match self.log_level {
            LogLevel::Trace =>
                self.print_log("TRACE", message),
            _ => {}
        }
    }
    
    pub fn debug(&self, message:&str) {
        match self.log_level {
            LogLevel::Trace | LogLevel::Debug =>
                self.print_log("DEBUG", message),
            _ => {}
        }
    }
    
    pub fn info(&self, message:&str) {
        match self.log_level {
             LogLevel::Trace | LogLevel::Debug | LogLevel::Info =>
                self.print_log("INFO", message),
            _ => {}
        }
    }
    
    pub fn warn(&self, message:&str) {
        match self.log_level {
             LogLevel::Trace | LogLevel::Debug | LogLevel::Info | LogLevel::Warn =>
                self.print_log("WARN", message),
            _ => {}
        }
    }
    
    pub fn error(&self, message:&str) {
        match self.log_level {
             LogLevel::Trace | LogLevel::Debug | LogLevel::Info | LogLevel::Warn | LogLevel::Error =>
                self.print_log("ERROR", message),
        }
    }

    ///TODO: Find some way to set default level. Optional params or something.
    pub fn init(name:&str, log_level:LogLevel) -> Logger{
        Logger {
            log_name: String::from(name),
            log_level: log_level,
        }
    }
}
