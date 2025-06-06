use std::fs::File;
use actix_files as fs;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use rstar::{RTree, RTreeObject, AABB};
use image::{ImageBuffer, Rgba};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
struct DataPoint {
    #[serde(rename = "X")]
    x: f64,
    #[serde(rename = "Y")]
    y: f64,
}

impl RTreeObject for DataPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

struct AppState {
    tree: RTree<DataPoint>,
}

fn tile2mercator(xtile: u32, ytile: u32, zoom: u32) -> (f64, f64) {
    let n = 2f64.powi(zoom as i32);
    let lon_deg = xtile as f64 / n * 360.0 - 180.0;
    let lat_rad = ((std::f64::consts::PI * (1.0 - 2.0 * ytile as f64 / n)).sinh()).atan();
    let lat_deg = lat_rad.to_degrees();
    lnglat_to_meters(lon_deg, lat_deg)
}

fn lnglat_to_meters(lon: f64, lat: f64) -> (f64, f64) {
    // Web Mercator projection (EPSG:3857) conversion
    let origin_shift = std::f64::consts::PI * 6378137.0;

    // Easting calculation
    let x = lon * origin_shift / 180.0;

    // Northing calculation - note the 90+lat adjustment
    let y = ((90.0 + lat) * std::f64::consts::PI / 360.0)
        .tan()
        .ln()
        * origin_shift
        / std::f64::consts::PI;

    (x, y)
}

fn generate_tile(zoom: u32, x: u32, y: u32, tree: &RTree<DataPoint>) -> Vec<u8> {
    let (xleft, ytop) = tile2mercator(x, y, zoom);
    let (xright, ybottom) = tile2mercator(x + 1, y + 1, zoom);

    let bbox = AABB::from_corners([xleft, ybottom], [xright, ytop]);
    let points = tree.locate_in_envelope(&bbox);

    let width = 256u32;
    let height = 256u32;
    let mut counts = vec![0u32; (width * height) as usize];

    for p in points {
        let px = ((p.x - xleft) / (xright - xleft) * width as f64) as i32;
        let py = ((ytop - p.y) / (ytop - ybottom) * height as f64) as i32;
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = (py as u32 * width + px as u32) as usize;
            counts[idx] += 1;
        }
    }

    let max_count = counts.iter().copied().max().unwrap_or(0);

    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
    if max_count > 0 {
        for (i, cnt) in counts.into_iter().enumerate() {
            let val = cnt as f32 / max_count as f32;
            let color = color_map(val);
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            img.put_pixel(x, y, color);
        }
    }

    use std::io::Cursor;
    let mut bytes: Vec<u8> = Vec::new();
    {
        let mut cursor = Cursor::new(&mut bytes);
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();
    }
    bytes
}

fn color_map(v: f32) -> Rgba<u8> {
    if !v.is_finite() || v <= 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    let r = (255.0 * v) as u8;
    let b = 255 - r;
    Rgba([r, 0, b, 255])
}

fn get_f64_by_name(row: &parquet::record::Row, name: &str) -> Option<f64> {
    for (n, field) in row.get_column_iter() {
        if n == name {
            return match field {
                Field::Double(v) => Some(*v),
                Field::Float(v) => Some(*v as f64),
                Field::Int(v) => Some(*v as f64),
                Field::Long(v) => Some(*v as f64),
                Field::UInt(v) => Some(*v as f64),
                Field::ULong(v) => Some(*v as f64),
                _ => None,
            };
        }
    }
    None
}

#[get("/")]
async fn index() -> impl Responder {
    fs::NamedFile::open("./www/index.html")
}

#[get("/tiles/{zoom}/{x}/{y}.png")]
async fn tile(path: web::Path<(u32, u32, u32)>, data: web::Data<AppState>) -> HttpResponse {
    let (z, x, y) = path.into_inner();
    let img = generate_tile(z, x, y, &data.tree);
    HttpResponse::Ok().content_type("image/png").body(img)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let file = File::open("data/stored.parquet").expect("data/stored.parquet not found");
    let reader = SerializedFileReader::new(file).expect("failed to open parquet");
    let iter = reader.get_row_iter(None).expect("unable to iterate parquet");
    let mut points = Vec::new();
    for record in iter {
        if let Ok(row) = record {
            let x = get_f64_by_name(&row, "X").unwrap_or(0.0);
            let y = get_f64_by_name(&row, "Y").unwrap_or(0.0);
            points.push(DataPoint { x, y });
        }
    }

    let tree = RTree::bulk_load(points);
    let data = web::Data::new(AppState { tree });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(index)
            .service(tile)
            .service(fs::Files::new("/lib", "./www/lib"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
