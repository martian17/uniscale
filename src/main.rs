type BaseNumType = u64;
type CompositeUnit = Vec<BaseNumType>;

#[derive(Clone)]
struct Unit{
    name: &'static str,
    tick_base: CompositeUnit,
    fmt_fn: Option<fn(CompositeUnit) -> String>,
    size: BaseNumType,
}

#[derive(Clone)]
struct UnitBuilder<'a>{
    name: &'static str,
    scale_builder: &'a ScaleBuilder,
    size: BaseNumType,
    tick_base: CompositeUnit,
    fmt_fn: Option<fn(CompositeUnit) -> String>,
}

impl<'a> UnitBuilder<'a>{
    fn new(scale_builder: &'a ScaleBuilder) -> Self{
        let size: BaseNumType = if let Some(last) = scale_builder.last_unit() {
            last.size * 1000
        } else {
            1
        };
        return Self{
            name: "",
            scale_builder: scale_builder,
            size: size,
            fmt_fn: None,
            tick_base: vec![2,5,10],
        };
    }
    fn name(&mut self, name: &'static str) -> &mut Self {
        self.name = name;
        self
    }
    fn relative_to_last (&mut self, rscale: BaseNumType) -> &mut Self {
        self.size = if let Some(last) = self.scale_builder.last_unit() {
            last.size * rscale
        } else {
            rscale
        };
        self
    }
    fn relative_to (&mut self, unit_name: &'static str, rscale: BaseNumType) -> &mut Self {
        self.size = if let Some(unit) = self.scale_builder.get_unit(unit_name) {
            unit.size * rscale
        } else {
            panic!("Unit {unit_name} not defined");
        };
        self
    }
    fn tick_base (&mut self, tick_base: CompositeUnit) -> &mut Self {
        self.tick_base = tick_base;
        self
    }
    fn build(self) -> Unit{
        return Unit{
            name: self.name,
            tick_base: self.tick_base,
            fmt_fn: self.fmt_fn,
            size: self.size,
        }
    }
}

#[derive(Clone)]
struct ScaleBuilder{
    name: &'static str,
    to_unit_counts_fn: Option<fn(BaseNumType) -> CompositeUnit>,
    from_unit_counts_fn: Option<fn(&CompositeUnit) -> Result<BaseNumType, ScaleError>>,
    fmt_fn: Option<fn(&CompositeUnit) -> String>,
    units: Vec<Unit>,
}

impl ScaleBuilder{
    fn new(name: &'static str) -> Self{
        return Self {
            name: name,
            to_unit_counts_fn: None,
            from_unit_counts_fn: None,
            fmt_fn: None,
            units: Vec::new(),
        }
    }
    fn to_composite(&mut self, cb: fn(BaseNumType) -> CompositeUnit) -> &mut Self{
        self.to_unit_counts_fn = Some(cb);
        self
    }
    fn from_composite(&mut self, cb: fn(&CompositeUnit) -> Result<BaseNumType, ScaleError>) -> &mut Self{
        self.from_unit_counts_fn = Some(cb);
        self
    }
    fn fmt(&mut self, cb: fn(&CompositeUnit) -> String) -> &mut Self{
        self.fmt_fn = Some(cb);
        self
    }
    fn unit<F>(&mut self, cb: F) -> &mut Self
        where for<'a> F: FnOnce(&'a mut UnitBuilder<'a>) -> &'a UnitBuilder<'a>
        // where F: FnOnce(&mut UnitBuilder) -> R
    {
        let mut ub = UnitBuilder::new(self);
        let mut ub = cb(&mut ub);
        self.units.push(ub.clone().build());
        self
    }

    fn last_unit(&self) -> Option<&Unit> {
        return self.units.last();
    }
    fn get_unit(&self, name: &'static str) -> Option<&Unit> {
        for unit in self.units.iter() {
            if unit.name == name {
                return Some(unit);
            }
        }
        return None;
    }
    fn build(&mut self) -> Scale {
        return Scale{
            name: self.name,
            to_unit_counts_fn: self.to_unit_counts_fn,
            from_unit_counts_fn: self.from_unit_counts_fn,
            fmt_fn: self.fmt_fn,
            units: self.units.clone(),
        }
    }
}


struct Scale{
    name: &'static str,
    to_unit_counts_fn: Option<fn(BaseNumType) -> CompositeUnit>,
    from_unit_counts_fn: Option<fn(&CompositeUnit) -> Result<BaseNumType, ScaleError>>,
    fmt_fn: Option<fn(&CompositeUnit) -> String>,
    units: Vec<Unit>,
}

// Extremely cheap error for result. Result<xxx, ScaleError> should be equivalent to Option<xxx>
#[derive(Copy, Clone, Debug)]
enum ScaleError {
    OutOfBounds,
}
// for usecase with Anyhow
impl std::fmt::Display for ScaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScaleError::OutOfBounds => {
                write!(f, "ScaleError: OutOfBounds")
            }
        }
    }
}
impl std::error::Error for ScaleError{}

fn radix_floor(val: BaseNumType, radix: BaseNumType) -> BaseNumType{
    let n = val/radix;
    return radix*n;
}

impl Scale{
    fn get_unit(&self, name: &'static str) -> Option<&Unit>{
        for unit in self.units.iter() {
            if unit.name == name {
                return Some(unit);
            }
        }
        return None;
    }
    fn units_big_to_small(&self) -> impl Iterator<Item = &Unit>{
        self.units.iter().rev()
    }
    // conversion from base unit
    fn to_composite(&self, value: BaseNumType) -> CompositeUnit {
        if let Some(to_composite) = self.to_unit_counts_fn {
            return to_composite(value);
        }
        let mut result: CompositeUnit = Vec::new();
        let mut value = value.clone();
        for unit in self.units_big_to_small() {
            let n = value / unit.size;
            value -= unit.size * n;
            result.push(n);
        }
        return result;
    }
    // conversion to base units
    fn from_composite(&self, composite_unit: &CompositeUnit) -> Result<BaseNumType, ScaleError> {
        if let Some(from_composite) = self.from_unit_counts_fn {
            return from_composite(composite_unit);
        }
        let mut result: BaseNumType = 0;
        for (unit, value) in self.units_big_to_small().zip(composite_unit.iter()) {
            result += unit.size * *value;
        }
        return Ok(result);
    }
    fn fmt_composite(&self, composite_unit: &CompositeUnit) -> String {
        let mut result = String::new();
        for (unit, value) in self.units_big_to_small().zip(composite_unit.iter()) {
            if *value == 0 {
                continue;
            }
            if result.len() != 0 {
                result.push(' ');
            }
            result.push_str(&format!("{} {}", value, unit.name));
        }
        return result;
    }
    fn fmt_base_unit(&self, value: BaseNumType) -> String {
        return self.fmt_composite(&self.to_composite(value));
    }
    fn is_unit_counts_canonical(&self, composite_unit: &CompositeUnit) -> bool {
        if let Ok(value) = self.from_composite(composite_unit) {
            return self.to_composite(value) == *composite_unit;
        } else {
            return false;
        }
    }
    fn increment_unit(&self, composite_unit: &CompositeUnit, unit_index: usize, radix: BaseNumType) -> CompositeUnit{
        let mut counts = composite_unit.clone();
        for i in (unit_index+1)..composite_unit.len() {
            counts[i] = 0;
        }
        let mut current_unit_index = unit_index;
        counts[current_unit_index] += radix;
        loop{
            if self.is_unit_counts_canonical(&counts) {
                break;
            }
            counts[current_unit_index] = 0;
            current_unit_index -= 1;
            counts[current_unit_index] += 1;
        }
        return counts;
    }
    fn ticks(&self, start: BaseNumType, end: BaseNumType, max_cnt: usize) -> Ticks {
        if end <= start {
            panic!("ticks: end must be larger than start");
        }
        let min_gap = (end - start) / max_cnt as BaseNumType;
        let mut tick_unit_index = self.units.len() - 1;
        let mut tick_unit = &self.units[tick_unit_index];
        for (i, unit) in self.units_big_to_small().enumerate() {
            if min_gap < unit.size {
                continue;
            }
            tick_unit_index = i;
            tick_unit = unit;
            break;
        }
        let mut radix: BaseNumType = 1;
        'outer: loop {
            let base_multiplier = radix;
            for factor in tick_unit.tick_base.clone().into_iter() {
                println!("radix: {}", radix);
                if radix * tick_unit.size > min_gap {
                    break 'outer;
                }
                radix = base_multiplier * factor;
            }
        }
        let unit_start = self.to_composite(start);
        let mut counts = unit_start.clone();
        let mut ticks = Ticks{
            labels: Vec::new(),
            aux_labels: Vec::new(),
        };
        counts[tick_unit_index] = radix_floor(counts[tick_unit_index], radix);
        loop {
            counts = self.increment_unit(&counts, tick_unit_index, radix);
            let val = self.from_composite(&counts).unwrap();// guaranteed to be canonical by
            // increment_unit
            if val > end {
                break;
            }
            ticks.labels.push((self.fmt_composite(&counts), val))
        }
        return ticks;
    }
}

#[derive(Debug)]
struct Ticks {
    labels: Vec<(String, BaseNumType)>,// relative ticks
    aux_labels: Vec<(String, BaseNumType)>,// for showing full formatted composite_unit values once in a while, so the user can know the absolute location
}



// fn main() {
//     let mut scale = ScaleBuilder::new("尺貫法")
//         .unit(|u|u.name("寸"))
//         .unit(|u|u.name("尺").relative_to_last(10))
//         .unit(|u|u.name("丈").relative_to_last(10))
//         .unit(|u|u.name("間").relative_to("尺", 6))
//         .unit(|u|u.name("町").relative_to_last(60))
//         .unit(|u|u.name("里").relative_to_last(36))
//         .build();
// 
//     println!("町: {}", scale.get_unit("町").unwrap().size);
//     println!("里: {}", scale.get_unit("里").unwrap().size);
//     println!("{:?}", scale.to_composite(3600));
//     println!("{}", scale.fmt_base_unit(3605));
//     println!("{}", scale.fmt_base_unit(129600));
//     println!("{}", scale.fmt_base_unit(12345678923));
//     // let mut counts = scale.to_composite(1);
//     // for i in 0..1000000000 {
//     //     if i%1000000 == 0 {
//     //         println!("{}", scale.fmt(&counts));
//     //     }
//     //     counts = scale.increment_unit(&counts, 5, 1);
//     // }
//     println!("start: {}", scale.fmt_base_unit(3000));
//     println!("{:?}", scale.ticks(3000, 6000, 10));
//     println!("end: {}", scale.fmt_base_unit(6000));
// }

fn main() {
    let mut scale = ScaleBuilder::new("time")
        .unit(|u|u.name("ps"))
        .unit(|u|u.name("ns").relative_to_last(1000))
        .unit(|u|u.name("μs").relative_to_last(1000))
        .unit(|u|u.name("ms").relative_to_last(1000))
        .unit(|u|u.name("s").relative_to_last(1000))
        .unit(|u|u.name("m").relative_to_last(60))
        .unit(|u|u.name("h").relative_to_last(60))
        .unit(|u|u.name("d").relative_to_last(24))
        .build();

    println!("ms: {}", scale.get_unit("ms").unwrap().size);
    println!("d: {}", scale.get_unit("d").unwrap().size);
    println!("{:?}", scale.to_composite(3600));
    println!("{}", scale.fmt_base_unit(3605));
    println!("{}", scale.fmt_base_unit(129600));
    println!("{}", scale.fmt_base_unit(12345678923));
    // let mut counts = scale.to_composite(1);
    // for i in 0..1000000000 {
    //     if i%1000000 == 0 {
    //         println!("{}", scale.fmt(&counts));
    //     }
    //     counts = scale.increment_unit(&counts, 5, 1);
    // }
    println!("start: {}", scale.fmt_base_unit(300000000000000000));
    println!("{:?}", scale.ticks(300000000000000000, 600000000000000000, 10));
    println!("end: {}", scale.fmt_base_unit(600000000000000000));
}


    // let mut scale = ScaleBuilder::new().to_composite(|val|{
    //     let scale: Vec<u64> = Vec::new();
    //     let mut val = val.abs();
    //     // ps
    //     scale.push(val%1000);
    //     val /= 1000;
    //     // ns
    //     scale.push(val%1000);
    //     val /= 1000;
    //     // μs
    //     scale.push(val%1000);
    //     val /= 1000;
    //     // ms
    //     scale.push(val%1000);
    //     val /= 1000;
    //     // s
    //     scale.push(val%60);
    //     val /= 60;
    //     // m
    //     scale.push(val%60);
    //     val /= 60;
    //     // h
    //     scale.push(val%24);
    //     val /= 24;
    //     // d
    //     scale.push(val);
    //     return scale;
    // }).fmt(|scale|{
    //     let ps = scale[0];
    //     let ns = scale[1];
    //     let us = scale[2];
    //     let ms = scale[3];
    //     let s = scale[4];
    //     let m = scale[5];
    //     let h = scale[6];
    //     let d = scale[7];
    //     if d == 0 {

    //     }
    // });

// }
