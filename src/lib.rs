type BaseNumType = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositeUnit {
    pub values: Vec<BaseNumType>,
    pub sign: Sign,
}

fn radix_floor(val: BaseNumType, radix: BaseNumType) -> BaseNumType{
    let n = val/radix;
    return radix*n;
}

impl CompositeUnit {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            sign: Sign::Positive,
        }
    }
    fn zero_out_after(&mut self, idx: usize, scale: &Scale) {
        let len = self.values.len();
        for i in idx..self.values.len() {
            // units are stored from smallest to largest, but composite values the other way
            self.values[i] = scale.units[len-i-1].starts_at;
        }
    }
    fn radix_floor_unit(&mut self, unit_index: usize, radix: BaseNumType) {
            self.values[unit_index] = radix_floor(self.values[unit_index], radix);
    }
    fn iter_big_to_small(&self) -> impl Iterator<Item = &BaseNumType> {
        return self.values.iter();
    } 
}

#[derive(Clone, Debug)]
pub struct Unit{
    pub name: &'static str,
    pub tick_base: Vec<BaseNumType>,
    pub fmt_fn: Option<fn(CompositeUnit) -> String>,
    pub size: BaseNumType,
    starts_at: BaseNumType,
}

#[derive(Clone)]
pub struct UnitBuilder<'a>{
    name: &'static str,
    scale_builder: &'a ScaleBuilder,
    size: BaseNumType,
    tick_base: Vec<BaseNumType>,
    fmt_fn: Option<fn(CompositeUnit) -> String>,
    starts_at: BaseNumType,
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
            starts_at: 0,
        };
    }
    // chainable methods
    pub fn name(&mut self, name: &'static str) -> &mut Self {
        self.name = name;
        self
    }
    pub fn relative_to_last (&mut self, rscale: BaseNumType) -> &mut Self {
        self.size = if let Some(last) = self.scale_builder.last_unit() {
            last.size * rscale
        } else {
            rscale
        };
        self
    }
    pub fn relative_to (&mut self, unit_name: &'static str, rscale: BaseNumType) -> &mut Self {
        self.size = if let Some(unit) = self.scale_builder.get_unit(unit_name) {
            unit.size * rscale
        } else {
            panic!("Unit {unit_name} not defined");
        };
        self
    }
    pub fn tick_base (&mut self, tick_base: Vec<BaseNumType>) -> &mut Self {
        self.tick_base = tick_base;
        self
    }
    pub fn starts_at (&mut self, starts_at: BaseNumType) -> &mut Self {
        self.starts_at = starts_at;
        self
    }
    // builder
    pub fn build(self) -> Unit{
        return Unit{
            name: self.name,
            tick_base: self.tick_base,
            fmt_fn: self.fmt_fn,
            size: self.size,
            starts_at: self.starts_at,
        }
    }
}

#[derive(Clone)]
pub struct ScaleBuilder{
    name: &'static str,
    to_unit_counts_fn: Option<fn(BaseNumType) -> CompositeUnit>,
    from_unit_counts_fn: Option<fn(&CompositeUnit) -> Result<BaseNumType, ScaleError>>,
    fmt_fn: Option<fn(&CompositeUnit) -> String>,
    units: Vec<Unit>,
}

impl ScaleBuilder{
    pub fn new(name: &'static str) -> Self{
        return Self {
            name: name,
            to_unit_counts_fn: None,
            from_unit_counts_fn: None,
            fmt_fn: None,
            units: Vec::new(),
        }
    }
    // chainable methods
    pub fn to_composite(&mut self, cb: fn(BaseNumType) -> CompositeUnit) -> &mut Self{
        self.to_unit_counts_fn = Some(cb);
        self
    }
    pub fn from_composite(&mut self, cb: fn(&CompositeUnit) -> Result<BaseNumType, ScaleError>) -> &mut Self{
        self.from_unit_counts_fn = Some(cb);
        self
    }
    pub fn fmt(&mut self, cb: fn(&CompositeUnit) -> String) -> &mut Self{
        self.fmt_fn = Some(cb);
        self
    }
    pub fn unit<F>(&mut self, cb: F) -> &mut Self
        where for<'a> F: FnOnce(&'a mut UnitBuilder<'a>) -> &'a UnitBuilder<'a>
        // where F: FnOnce(&mut UnitBuilder) -> R
    {
        let mut ub = UnitBuilder::new(self);
        let ub = cb(&mut ub);
        self.units.push(ub.clone().build());
        self
    }
    // builder
    pub fn build(&mut self) -> Scale {
        return Scale{
            name: self.name,
            to_unit_counts_fn: self.to_unit_counts_fn,
            from_unit_counts_fn: self.from_unit_counts_fn,
            fmt_fn: self.fmt_fn,
            units: self.units.clone(),
        }
    }

    // private methods
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
}

#[derive(Debug)]
pub struct Scale{
    pub name: &'static str,
    to_unit_counts_fn: Option<fn(BaseNumType) -> CompositeUnit>,
    from_unit_counts_fn: Option<fn(&CompositeUnit) -> Result<BaseNumType, ScaleError>>,
    fmt_fn: Option<fn(&CompositeUnit) -> String>,
    units: Vec<Unit>,
}

// Extremely cheap error for result. Result<xxx, ScaleError> should be equivalent to Option<xxx>
#[derive(Copy, Clone, Debug)]
pub enum ScaleError {
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


impl Scale{
    fn units_big_to_small(&self) -> impl Iterator<Item = &Unit>{
        self.units.iter().rev()
    }


    // public methods
    pub fn get_unit(&self, name: &'static str) -> Option<&Unit>{
        for unit in self.units.iter() {
            if unit.name == name {
                return Some(unit);
            }
        }
        return None;
    }
    // conversion from base unit
    pub fn to_composite(&self, value: BaseNumType) -> CompositeUnit {
        if let Some(to_composite) = self.to_unit_counts_fn {
            return to_composite(value);
        }
        let mut result = CompositeUnit::new();
        let mut value = value.clone();
        for unit in self.units_big_to_small() {
            let n = value / unit.size;
            value -= unit.size * n;
            result.values.push(n);
        }
        return result;
    }
    // conversion to base units
    pub fn from_composite(&self, composite_unit: &CompositeUnit) -> Result<BaseNumType, ScaleError> {
        if let Some(from_composite) = self.from_unit_counts_fn {
            return from_composite(composite_unit);
        }
        let mut result: BaseNumType = 0;
        for (unit, value) in self.units_big_to_small().zip(composite_unit.iter_big_to_small()) {
            result += unit.size * *value;
        }
        return Ok(result);
    }
    pub fn fmt_composite(&self, composite_unit: &CompositeUnit) -> String {
        if let Some(fmt_fn) = self.fmt_fn{
            return fmt_fn(composite_unit);
        }
        let mut result = String::new();
        for (unit, value) in self.units_big_to_small().zip(composite_unit.iter_big_to_small()) {
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
    pub fn fmt_base_unit(&self, value: BaseNumType) -> String {
        return self.fmt_composite(&self.to_composite(value));
    }
    pub fn is_unit_counts_canonical(&self, composite_unit: &CompositeUnit) -> bool {
        if let Ok(value) = self.from_composite(composite_unit) {
            return self.to_composite(value) == *composite_unit;
        } else {
            return false;
        }
    }
    pub fn increment_unit(&self, composite_unit: &CompositeUnit, unit_index: usize, radix: BaseNumType) -> CompositeUnit{
        let mut composite = composite_unit.clone();
        composite.zero_out_after(unit_index+1, self);
        let mut current_unit_index = unit_index;
        composite.values[current_unit_index] += radix;
        loop{
            if self.is_unit_counts_canonical(&composite) {
                break;
            }
            let len = self.units.len();
            composite.values[current_unit_index] = self.units[len-current_unit_index-1].starts_at;
            current_unit_index -= 1;
            composite.values[current_unit_index] += 1;
        }
        return composite;
    }
    pub fn ticks(&self, start: BaseNumType, end: BaseNumType, max_cnt: usize) -> Ticks {
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
                if radix * tick_unit.size > min_gap {
                    break 'outer;
                }
                radix = base_multiplier * factor;
            }
        }
        let composite_start = self.to_composite(start);
        let mut composite = composite_start.clone();
        let mut ticks = Ticks{
            labels: Vec::new(),
            aux_labels: Vec::new(),
        };
        composite.radix_floor_unit(tick_unit_index, radix);
        composite.zero_out_after(tick_unit_index + 1, self);
        if composite != composite_start {
            composite = self.increment_unit(&composite, tick_unit_index, radix);
        }
        loop {
            let val = self.from_composite(&composite).unwrap();// guaranteed to be canonical by
            // increment_unit
            if val > end {
                break;
            }
            ticks.labels.push((self.fmt_composite(&composite), val));
            composite = self.increment_unit(&composite, tick_unit_index, radix);
        }
        return ticks;
    }
}

#[derive(Debug)]
pub struct Ticks {
    pub labels: Vec<(String, BaseNumType)>,// relative ticks
    pub aux_labels: Vec<(String, BaseNumType)>,// for showing full formatted composite_unit values once in a while, so the user can know the absolute location
}
