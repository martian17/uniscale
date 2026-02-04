use nicer_ticks::ScaleBuilder;



fn main() {
    let scale = ScaleBuilder::new("尺貫法")
        .unit(|u|u.name("寸"))
        .unit(|u|u.name("尺").relative_to_last(10))
        .unit(|u|u.name("丈").relative_to_last(10))
        .unit(|u|u.name("間").relative_to("尺", 6))
        .unit(|u|u.name("町").relative_to_last(60))
        .unit(|u|u.name("里").relative_to_last(36))
        .build();

    println!("町: {}", scale.get_unit("町").unwrap().size);
    println!("里: {}", scale.get_unit("里").unwrap().size);
    println!("{:?}", scale.to_composite(3600));
    println!("{}", scale.fmt_base_unit(3605));
    println!("{}", scale.fmt_base_unit(129600));
    println!("{}", scale.fmt_base_unit(12345678923));
    // let mut composite = scale.to_composite(1);
    // for i in 0..1000000000 {
    //     if i%1000000 == 0 {
    //         println!("{}", scale.fmt(&composite));
    //     }
    //     composite = scale.increment_unit(&composite, 5, 1);
    // }
    println!("start: {}", scale.fmt_base_unit(3000));
    println!("{:?}", scale.ticks(3000, 6000, 10));
    println!("end: {}", scale.fmt_base_unit(6000));
}
