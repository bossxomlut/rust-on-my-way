fn main() {
    let mut run_template = <dyn JustRunTemplate>::get_cry_man();
    run_template.run();

    run_template = <dyn JustRunTemplate>::get_smile_man();

    run_template.run();
}

struct SmileMan {
    pub name: String,
}

struct CryMan {
    pub name: String,
}

trait JustRunTemplate {
    fn run_first(&self);
    fn run_second(&self);
    fn run_third(&self);

    fn run(&self) {
        println!("I'm just running!");
        self.run_first();
        self.run_second();
        self.run_third();
    }
}

// ✅ Factory methods đặt ở inherent impl của trait object
impl dyn JustRunTemplate {
    fn get_smile_man() -> Box<dyn JustRunTemplate> {
        Box::new(SmileMan {
            name: String::from("Kieu Phong"),
        })
    }

    fn get_cry_man() -> Box<dyn JustRunTemplate> {
        Box::new(CryMan {
            name: String::from("Kieu Bong"),
        })
    }
}

impl JustRunTemplate for SmileMan {
    fn run_first(&self) {
        println!("{} say he", self.name);
    }
    fn run_second(&self) {
        println!("{} say he he", self.name);
    }
    fn run_third(&self) {
        println!("{} say he he he", self.name);
    }
}

impl JustRunTemplate for CryMan {
    fn run_first(&self) {
        println!("{} cry hu", self.name);
    }
    fn run_second(&self) {
        println!("{} cry hu hu", self.name);
    }
    fn run_third(&self) {
        println!("{} cry hu hu hu", self.name);
    }
}
