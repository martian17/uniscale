fn is_power_of_10(n: u64) -> bool {
    if n == 0 {
        return false;
    }
    let mut n = n.clone();
    while n % 10 == 0 {
        n /= 10;
    }
    n == 1
}


struct Unit {
    name: &'static str,
    plural: &'static str,
    scale: u64,
    rscale: u64,
    breakpoints: Vec<u64>,
    decimal: bool,
}

impl Unit {
    fn to_string(&self, value: u64) -> String{
        if value == 1 {
            return format!("1 {}", self.name);
        } else {
            return format!("{} {}", value, self.plural);
        }
    }
}


struct UnitScale {
    units: Vec<Unit>,
}

struct Label {
    str: String,
    value: i64,
}

impl UnitScale {
    fn new(name: &'static str) -> Self{
        return Self{
            units: vec![Unit{
                name: name,
                plural: name,
                scale: 1,
                rscale: 1,
                breakpoints: vec![1, 2, 5],
                decimal: true
            }]
        };
    }
    fn last_mut(&mut self) -> &mut Unit {
        return self.units.last_mut().expect("UnitScale mut be initialized. UnitScale.units is empty. Use UnitScale::new()");
    }
    fn last(&self) -> &Unit {
        return self.units.last().expect("UnitScale mut be initialized. UnitScale.units is empty. Use UnitScale::new()");
    }
    fn append_unit(&mut self, name: &'static str) -> &mut Self{
        let last_scale = self.last().scale;
        self.units.push(Unit{
            name: name,
            plural: name,
            scale: last_scale * 1000,
            rscale: 1000,
            breakpoints: vec![1, 2, 5],
            decimal: true
        });
        self
    }
    fn plural(&mut self, plural: &'static str) -> &mut Self{
        let mut last = self.last_mut();
        last.plural = plural;
        self
    }
    fn rscale(&mut self, rscale: u64) -> &mut Self{
        let second_last_scale = self.units.get(self.units.len() - 2).expect("UnitScale::rscale can only be called on appended unit").scale;
        let mut last = self.last_mut();
        if ! is_power_of_10(rscale) {
            last.decimal = false;
        }
        last.scale = second_last_scale * rscale;
        last.rscale = rscale;
        self
    }
    fn breakpoints(&mut self, breakpoints: Vec<u64>) -> &mut Self{
        let mut last = self.last_mut();
        last.breakpoints = breakpoints;
        self
    }
    fn decimal(&mut self, decimal: bool) -> &mut Self{
        let mut last = self.last_mut();
        last.decimal = decimal;
        self
    }

    fn value_to_string(&self, value: i64) -> String {
        let mut unit_values = Vec::<u64>::new();
        let mut value_abs = value.abs() as u64;
        for unit in self.units.iter().skip(1) {
            unit_values.push(value_abs%unit.rscale);
            value_abs /= unit.rscale;
        }
        unit_values.push(value_abs);
        let mut result = String::new();
        if value < 0 {
            result.push('-');
        }
        let mut initialized = false;
        for (unit_val, unit) in unit_values.into_iter().rev().zip(self.units.iter().rev()) {
            if unit_val == 0 && !initialized {
                continue;
            }
            if initialized {
                result.push(' ');
            }
            initialized = true;
            result.push_str(&unit.to_string(unit_val));
        }
        return result;
    }
    // fn value_to_string_truncated
    //
    // fn ticks(&self, start:u64, stop: u64, count: u64) -> Vec<Label> {

    // }
}


fn main() {
    let mut time_scale = UnitScale::new("ps");
    time_scale.breakpoints(vec![1, 2, 5])
        .append_unit("ns").rscale(1000)
        .append_unit("μs").rscale(1000)
        .append_unit("ms").rscale(1000)
        .append_unit("s").rscale(1000)
        .append_unit("m").rscale(60).breakpoints(vec![1, 2, 3, 4, 5, 10, 15, 20, 30])
        .append_unit("h").rscale(60).breakpoints(vec![1, 2, 3, 4, 5, 10, 15, 20, 30])
        .append_unit("day").plural("days").rscale(24).breakpoints(vec![1, 2, 3, 4, 6, 8, 12]);
    println!("{}",time_scale.value_to_string(9111111000000000000i64));
    println!("{}",time_scale.value_to_string(-9111111000123456789i64));
}
