use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

// 定义一个结构体来存储已知的桩号、纬度和经度
#[derive(Debug)]
struct KnownLocation {
    station: f64,
    latitude: f64,
    longitude: f64,
}

// 定义一个结构体来存储测量数据
#[derive(Debug)]
struct Measurement {
    station: f64,
    depth: f64,
    resistivity: f64,
}

// 定义一个结构体来存储插值结果
#[derive(Debug)]
struct ResultData {
    latitude: f64,
    longitude: f64,
    depth: f64,
    resistivity: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // 读取已知的桩号和经纬度数据
    let known_data = read_known_locations("location.txt")?;
    println!("Known Data Length: {}", known_data.len());

    // 读取测量数据文件中的桩号、深度和电阻率数据
    let measurement_data = read_measurement_data("data.dat")?;
    println!("Measurement Data Length: {}", measurement_data.len());

    // 电阻率阈值
    let resistivity_threshold = 1.70141e15;

    // 初始化结果数组
    let mut result_data = Vec::new();

    // 循环遍历测量数据
    for measurement in &measurement_data {
        if measurement.resistivity <= resistivity_threshold {
            // 使用已知的桩号和经纬度数据进行插值
            for i in 0..known_data.len().saturating_sub(1) {
                if known_data[i].station <= measurement.station
                    && measurement.station <= known_data[i + 1].station
                {
                    let x1 = known_data[i].station;
                    let lat1 = known_data[i].latitude;
                    let lon1 = known_data[i].longitude;

                    let x2 = known_data[i + 1].station;
                    let lat2 = known_data[i + 1].latitude;
                    let lon2 = known_data[i + 1].longitude;

                    // 计算线性插值
                    let lat_target = lat1 + (measurement.station - x1) * (lat2 - lat1) / (x2 - x1);
                    let lon_target = lon1 + (measurement.station - x1) * (lon2 - lon1) / (x2 - x1);

                    // 将结果添加到结果数组
                    result_data.push(ResultData {
                        latitude: lat_target,
                        longitude: lon_target,
                        depth: measurement.depth,
                        resistivity: measurement.resistivity,
                    });
                    println!(
                        "Interpolated Result: ({}, {}, {}, {})",
                        lat_target, lon_target, measurement.depth, measurement.resistivity
                    );
                    break;
                }
            }
        }
    }

    // 将结果写入新的文件
    write_result_data("result.txt", &result_data)?;

    Ok(())
}

fn clean_number_string(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_digit(10) || *c == '.' || *c == '-')
        .collect()
}

// 读取已知的桩号和经纬度数据
fn read_known_locations(path: &str) -> Result<Vec<KnownLocation>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .flexible(true) // Enable flexible delimiter handling
        .has_headers(false)
        .from_reader(file);

    let mut data = Vec::new();
    for result in reader.records() {
        let record = result?;
        if record.len() >= 3 {
            // Skip any empty fields and parse only non-empty ones
            let values: Vec<&str> = record
                .iter()
                .filter(|&field| !field.trim().is_empty())
                .collect();

            if values.len() >= 3 {
                // Clean and parse each value
                let station_str = clean_number_string(values[0]);
                let latitude_str = clean_number_string(values[1]);
                let longitude_str = clean_number_string(values[2]);

                // Try to parse the cleaned strings
                if let (Ok(station), Ok(latitude), Ok(longitude)) = (
                    station_str.parse::<f64>(),
                    latitude_str.parse::<f64>(),
                    longitude_str.parse::<f64>(),
                ) {
                    data.push(KnownLocation {
                        station,
                        latitude,
                        longitude,
                    });
                }
            }
        }
    }
    Ok(data)
}

// 读取测量数据文件中的桩号、深度和电阻率数据
fn read_measurement_data(path: &str) -> Result<Vec<Measurement>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .delimiter(b' ') // 使用空格作为分隔符
        .has_headers(false)
        .from_reader(file);

    let mut data = Vec::new();
    for result in reader.records() {
        let record = result?;
        if record.len() >= 3 {
            let station: f64 = record.get(0).unwrap_or("0").parse()?;
            let depth: f64 = record.get(1).unwrap_or("0").parse()?;
            let resistivity: f64 = record.get(2).unwrap_or("0").parse()?;
            data.push(Measurement {
                station,
                depth,
                resistivity,
            });
        }
    }
    Ok(data)
}

// 将结果写入新的文件
fn write_result_data(path: &str, data: &[ResultData]) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = WriterBuilder::new().delimiter(b' ').from_writer(file);

    for item in data {
        writer.write_record(&[
            item.latitude.to_string(),
            item.longitude.to_string(),
            item.depth.to_string(),
            item.resistivity.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}
