use nicer_ticks::ScaleBuilder;
use nicer_ticks::CompositeUnit;
use nicer_ticks::ScaleError;
use nicer_ticks::Sign;

fn main() {
    let scale = ScaleBuilder::new("datetime")
        .unit(|u|u.name("ms"))
        .unit(|u|u.name("s").relative_to_last(1000))
        .unit(|u|u.name("m").relative_to_last(60))
        .unit(|u|u.name("h").relative_to_last(60))
        .unit(|u|u.name("d").relative_to_last(24).starts_at(1))
        .unit(|u|u.name("mon").relative_to_last(30).starts_at(1))
        .unit(|u|u.name("year").relative_to("d", 365))
        .from_composite(|composite|{
            // offsetting with zeroth element = 0 because month starts from 1 
            const MONTH_LENGTHS: [u8; 13] = [
                // JAN FEB MAR APR MAY JUN JUL AUG SEP OCT NOV DEC
                0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
            ];
            const MONTH_OFFSETS: [u16; 13] = [
                // JAN FEB MAR APR MAY  JUN  JUL  AUG  SEP  OCT  NOV  DEC
                0, 0,  31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334
            ];

            let year = composite.values[0];
            let month = composite.values[1];
            let day = composite.values[2];
            let hour = composite.values[3];
            let minute = composite.values[4];
            let second = composite.values[5];
            let millisecond = composite.values[6];

            let mut days_sum = (year - 1970) * 365;
            days_sum += year.div_euclid(4)   - (1970u64).div_euclid(4);
            days_sum -= year.div_euclid(100) - (1970u64).div_euclid(100);
            days_sum += year.div_euclid(400) - (1970u64).div_euclid(400);

            let is_leap_year = (year.rem_euclid(4) == 0)
                && !((year.rem_euclid(100) == 0) && !(year.rem_euclid(400) == 0));

            if !(1..=12).contains(&month){
                return Err(ScaleError::OutOfBounds);
            }

            let month_length = if is_leap_year && month == 2 {
                29
            } else {
                MONTH_LENGTHS[month as usize] as u64
            };
            let month_offset = MONTH_OFFSETS[month as usize] as u64
                + if is_leap_year && month > 3 {1} else {0};

            if day < 1 || day > month_length {
                return Err(ScaleError::OutOfBounds);
            }

            days_sum += month_offset + (day - 1);

            if !(0..24).contains(&hour)
            || !(0..60).contains(&minute)
            || !(0..60).contains(&second)
            || !(0..1000).contains(&millisecond) {
                return Err(ScaleError::OutOfBounds);
            }
            let time_ms = days_sum * 24*60*60*1000 + hour * 60*60*1000 + minute * 60*1000 + second * 1000 + millisecond;
            return Ok(time_ms);
        })
        .to_composite(|time_ms|{
            // going to calculate it from milliseconds -> year, but it could go the other way,
            // in case leap seconds have to be considered
            let day_rem = time_ms.rem_euclid(24*60*60*1000);
            let days = time_ms.div_euclid(24*60*60*1000);

            let day_length_4 = 4*365 + 1;
            let day_length_100 = day_length_4 * 25 - 1;
            let day_length_400 = day_length_100 * 4 + 1;

            let days_1970 = ((1970 / 400) * day_length_400) + ((370 / 100) * day_length_100) + (70 / 4) * day_length_4 + (365 * 2);
            let days_normalized = days + days_1970;

            let mut days_tmp = days_normalized;
            let years_400 = days_tmp.div_euclid(day_length_400);
            days_tmp = days_tmp.rem_euclid(day_length_400);
            let years_100 = days_tmp.div_euclid(day_length_100);
            days_tmp = days_tmp.rem_euclid(day_length_100);
            let years_4 = days_tmp.div_euclid(day_length_4);
            days_tmp = days_tmp.rem_euclid(day_length_4);
            let years_1 = days_tmp.div_euclid(365);
            days_tmp = days_tmp.rem_euclid(365);

            let res_year = years_400 * 400 + years_100 * 100 + years_4 * 4 + years_1;
            
            let is_leap_year = (res_year.rem_euclid(4) == 0)
                && !((res_year.rem_euclid(100) == 0) && !(res_year.rem_euclid(400) == 0));

            let month_length = [
                31u8,
                if is_leap_year { 29 } else { 28 },
                31, 30, 31, 30,
                31, 31, 30, 31, 30, 31,
            ];

            let mut res_month = 1;
            for &len in &month_length {
                if days_tmp < len as u64 {
                    break;
                }
                res_month += 1;
                days_tmp -= len as u64;
            }

            // guaranteed to be positive
            let res_day = days_tmp + 1;

            let mut ms_tmp = day_rem;
            let res_hour = ms_tmp/(60*60*1000);
            ms_tmp = ms_tmp%(60*60*1000);
            let res_minute = ms_tmp/(60*1000);
            ms_tmp = ms_tmp%(60*1000);
            let res_second = ms_tmp/(1000);
            ms_tmp = ms_tmp%(1000);
            let res_millisecond = ms_tmp;

            return CompositeUnit {
                values: vec![res_year, res_month, res_day, res_hour, res_minute, res_second, res_millisecond],
                sign: Sign::Positive,
            };
        })
        .build();

    println!("{:?}", scale.from_composite(&CompositeUnit{
        values: vec![1970u64, 1, 1, 0, 0, 0, 0],
        sign: Sign::Positive,
    }).unwrap());

    println!("{:?}", scale.from_composite(&CompositeUnit{
        values: vec![2026u64, 2, 3, 13, 0, 47, 927],
        sign: Sign::Positive,
    }).unwrap());

    println!("{:?}", scale.to_composite(scale.from_composite(&CompositeUnit{
        values: vec![2026u64, 2, 3, 13, 0, 47, 927],
        sign: Sign::Positive,
    }).unwrap()));
    // println!("start: {}", scale.fmt_base_unit(3000));
    println!("{:?}", scale.ticks(1770123647927, 1801659647927, 10));
    // println!("end: {}", scale.fmt_base_unit(6000));
}
