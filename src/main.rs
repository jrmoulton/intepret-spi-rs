use anyhow::Result;
use plotters::{
    prelude::IntoDrawingArea,
    style::{Color, IntoFont},
};
use serde::{de::Unexpected, Deserialize, Deserializer};

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Sample {
    #[serde(rename(deserialize = "Time [s]"))]
    time: f32,
    #[serde(rename = "Device 1")]
    #[serde(deserialize_with = "bool_from_int")]
    device_1: bool,
    #[serde(rename = "Device 2")]
    #[serde(deserialize_with = "bool_from_int")]
    device_2: bool,
    #[serde(rename = "Data")]
    #[serde(deserialize_with = "bool_from_int")]
    data: bool,
    #[serde(rename = "Clock")]
    #[serde(deserialize_with = "bool_from_int")]
    clock: bool,
}

#[allow(dead_code)]
struct Byte {
    time: f32,
    data: u8,
}

#[derive(Default)]
struct Command {
    time: f32,
    data: [u8; 3],
}

fn main() -> Result<()> {
    let mut rdr = csv::Reader::from_path("data/digital.csv")
        .expect("The file containing the raw data wasn't found in the data directory");
    let raw_data: Result<Vec<Sample>, csv::Error> = rdr.deserialize().into_iter().collect();
    let raw_data = raw_data.unwrap();
    let mut device_1_clocked_data: Vec<&Sample> = Vec::new();
    for i in 0..raw_data.len() - 1 {
        // If device 1 active and clock went from high to low
        if !raw_data[i].device_1 && raw_data[i].clock && !raw_data[i + 1].clock {
            device_1_clocked_data.push(&raw_data[i]);
        }
    }
    let mut time_and_byte: Vec<Byte> = Vec::new();
    for bit_chunk in device_1_clocked_data.chunks(8) {
        time_and_byte.push(Byte {
            time: bit_chunk.get(0).unwrap().time,
            data: bit_chunk
                .iter()
                .fold(0, |acc, byte| acc << 1 | byte.data as u8),
        });
    }
    let mut commands: Vec<Command> = Vec::new();
    for byte_chunk in time_and_byte.chunks(3) {
        let mut command = Command::default();
        byte_chunk
            .iter()
            .enumerate()
            .for_each(|(index, byte)| command.data[index] = byte.data);
        command.time = byte_chunk.get(0).unwrap().time;
        commands.push(command);
    }
    let root = plotters::backend::BitMapBackend::new("images/rust_plot.png", (2000, 400))
        .into_drawing_area();
    root.fill(&plotters::style::WHITE)?;
    let mut chart = plotters::chart::ChartBuilder::on(&root)
        .caption(
            "Raw SPI Data Containing A Sine Wave",
            ("sans-serif", 70).into_font(),
        )
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(-6.75f32..-6.52f32, 0f32..0xFFFF as f32)?;
    chart.configure_mesh().draw()?;
    chart
        .draw_series(plotters::series::LineSeries::new(
            commands.iter().take(1024 * 5).map(|command| {
                (
                    command.time,
                    (command.data[1] as u16 * 255 + command.data[2] as u16) as f32,
                )
            }),
            &plotters::style::RED,
        ))?
        .label("Sine wave")
        .legend(|(x, y)| {
            plotters::element::PathElement::new(vec![(x, y), (x + 20, y)], &plotters::style::RED)
        });
    chart
        .configure_series_labels()
        .background_style(&plotters::style::WHITE.mix(0.8))
        .border_style(&plotters::style::BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}
