# Nicer Ticks

`nicer-ticks` is a Rust library for generating **human-readable tick labels for axes** using **custom, possibly irregular unit systems**. Unlike standard tick generators (like `d3.ticks`), this library allows you to define **non-base-10 units** such as historical measurements, imperial units, or calendar periods (months, quarters), and automatically generates nicely spaced axis ticks.

---

## Features

- Define custom units with arbitrary relationships.
- Support for **regularly spaced units** (`relative_to_last`) or **irregular units** (`relative_to`).
- Convert between a base numeric type and composite units (`CompositeUnit`).
- Format numbers in a readable, unit-aware form.
- Generate ticks for chart axes with proper labels.

---

## Example: Japanese Shakkanhō Units

```rust
use nicer_ticks::ScaleBuilder;

let scale = ScaleBuilder::new("尺貫法")
    .unit(|u| u.name("寸"))
    .unit(|u| u.name("尺").relative_to_last(10))
    .unit(|u| u.name("丈").relative_to_last(10))
    .unit(|u| u.name("間").relative_to("尺", 6))
    .unit(|u| u.name("町").relative_to_last(60))
    .unit(|u| u.name("里").relative_to_last(36))
    .build();

println!("町: {}", scale.get_unit("町").unwrap().size);
println!("里: {}", scale.get_unit("里").unwrap().size);

// Convert a base number to a composite unit
println!("{:?}", scale.to_composite(3600));

// Format numbers using human-readable units
println!("{}", scale.fmt_base_unit(3605));
println!("{}", scale.fmt_base_unit(129600));
println!("{}", scale.fmt_base_unit(12345678923));

// Generate ticks for chart axes
println!("start: {}", scale.fmt_base_unit(3000));
println!("{:?}", scale.ticks(3000, 6000, 10));
println!("end: {}", scale.fmt_base_unit(6000));
```

## Output
```
町: 3600
里: 129600
CompositeUnit { values: [0, 1, 0, 0, 0, 0], sign: Positive }
1 町 5 寸
1 里
95259 里 31 町 15 間 2 尺 3 寸
start: 50 間
Ticks { labels: [("50 間", 3000), ("1 町", 3600), ...], aux_labels: [] }
end: 1 町 40 間
```

---


## Irregular Units

For irregular units such as months, quarters, or custom calendars, you can define conversions using callbacks:
```rust
scale
    .to_composite(|base| CompositeUnit { /* ... */ })
    .from_composite(|comp| Ok(/* base */))
    .fmt(|comp| format!("{:?}", comp));
```


This allows you to completely customize how base numeric values are converted to and displayed as composite units.


---

## `CompositeUnit` Struct

The core data structure for representing a number in a combination of units:
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositeUnit {
    pub values: Vec<BaseNumType>,
    pub sign: Sign,
}
```


* `values`: stores each unit component (e.g., [0,1,0,0,0,0] for 1 町)
* `sign`: positive or negative

---

`nicer-ticks` is perfect for charts, data visualization, and applications where non-standard units must be displayed clearly and consistently.

