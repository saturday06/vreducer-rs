mod cleaner;
mod debug;
mod gltf;
mod reducer;
mod version;

pub use self::cleaner::*;
pub use self::debug::*;
pub use self::gltf::*;
pub use self::reducer::*;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const GLTF_MAGIC: u32 = 0x46546c67;
const JSON_TYPE: u32 = 0x4e4f534a;

pub struct Vrm {
    version: u32,
    total_chunk_bytes: u32,
    pub chunk0: Value,
    chunks: Vec<Vec<u8>>,
}

impl Vrm {
    pub fn save(&self, path: &Path) -> Result<(), Box<std::error::Error>> {
        let mut file = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?,
        );
        let gltf_string = self.chunk0.to_string();
        let mut gltf_encoded = gltf_string.as_bytes().to_vec();
        if gltf_encoded.len() % 4 != 0 {
            gltf_encoded.resize((gltf_encoded.len() + 3) / 4 * 4, 0x20);
        }
        let glb_length: usize = 12 + 8 + gltf_encoded.len() + self.total_chunk_bytes as usize;
        file.write_u32::<LE>(GLTF_MAGIC)?;
        file.write_u32::<LE>(self.version)?;
        file.write_u32::<LE>(glb_length as u32)?;

        file.write_u32::<LE>(gltf_encoded.len() as u32)?;
        file.write_u32::<LE>(JSON_TYPE)?;
        file.write_all(&mut gltf_encoded)?;

        for mut chunk in &self.chunks {
            file.write_u32::<LE>(chunk.len() as u32)?;
            file.write_u32::<LE>(CHUNK_TYPE)?;
            file.write_all(&mut chunk)?;
        }

        Ok(())
    }

    // https://github.com/ousttrue/UniGLTF/blob/71188cbb88eced710c7c1c550bbde09d756ecc3a/Core/Scripts/IO/ImporterContext.cs#L274
    pub fn upgrade_chunk0(v: Value) -> Value {
        let mut chunk0 = v.clone();
        if chunk0.pointer("extensions/VRM/exporterVersion").is_some() {
            return chunk0;
        }

        let vrm_version = if let Some(x) = chunk0
            .get("extensions")
            .and_then(|v| v.get("VRM"))
            .and_then(|v| v.get("version"))
            .and_then(|v| v.as_str())
        {
            x
        } else {
            return chunk0;
        };

        // https://github.com/dwango/UniVRM/releases/tag/v0.36
        let versions = vrm_version
            .split(".")
            .map(|v| v.parse::<u64>())
            .collect::<Vec<_>>();
        match (versions.get(0), versions.get(1)) {
            (Some(Ok(major)), Some(Ok(minor))) => {
                if *major > 0 || *minor > 35 {
                    return chunk0;
                }
            }
            _ => return chunk0,
        }

        for image in chunk0
            .get_mut("images")
            .and_then(|v| v.as_array_mut())
            .unwrap_or(&mut Vec::new())
        {
            let name = image["name"].clone();
            let extra_name = image["extra"]["name"].clone();
            if (!name.is_string() || name.as_str().map(|s| s.len()) == Some(0))
                && extra_name.is_string()
                && extra_name.as_str().map(|s| s.len()) != Some(0)
            {
                image["name"] = extra_name;
            }
            image.as_object_mut().map(|i| i.remove("extra"));
        }

        for mesh in chunk0
            .get_mut("meshes")
            .and_then(|v| v.as_array_mut())
            .unwrap_or(&mut Vec::new())
        {
            for primitive in mesh["primitives"]
                .as_array_mut()
                .unwrap_or(&mut Vec::new())
                .iter_mut()
                .filter_map(|p| p.as_object_mut())
            {
                let mut target_names = Vec::new();
                for target in primitive["targets"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|t| t.as_object())
                {
                    target["extra"].as_object().map(|extra| {
                        extra["name"].as_str().map(|name| {
                            target_names.push(serde_json::Value::String(name.into()));
                        })
                    });
                }

                if !target_names.is_empty() {
                    let mut extras = serde_json::map::Map::new();
                    extras.insert("targetNames".into(), serde_json::Value::Array(target_names));
                    primitive.insert("extras".into(), serde_json::Value::Object(extras));
                }

                for target in primitive["targets"]
                    .as_array_mut()
                    .unwrap_or(&mut Vec::new())
                    .iter_mut()
                    .filter_map(|t| t.as_object_mut())
                {
                    for key in &["JOINTS_0", "TEXCOORD_0", "WEIGHTS_0"] {
                        if target[*key].as_i64() == Some(-1) {
                            target.remove(*key);
                        };
                    }
                    target.remove("extra");
                }

                if primitive["targets"].as_array().map(|target| target.len()) == Some(0) {
                    primitive.remove("targets");
                }
            }
        }

        chunk0
    }

    /// VRM読み込み
    pub fn load(path: &Path) -> Result<Vrm, Box<std::error::Error>> {
        Self::load_reader(BufReader::new(std::fs::File::open(path)?))
    }

    /// VRM読み込み
    pub fn load_reader<R>(mut reader: R) -> Result<Vrm, Box<std::error::Error>>
    where
        R: Read,
    {
        let gltf_magic = reader.read_u32::<LE>()?;
        let version = reader.read_u32::<LE>()?;
        let length = reader.read_u32::<LE>()?;
        assert_eq!(gltf_magic, GLTF_MAGIC);

        let json_length = reader.read_u32::<LE>()?; // TODO: json lengthを信用しない方法
        let json_type = reader.read_u32::<LE>()?;

        assert_eq!(json_type, JSON_TYPE);

        let mut json_bytes = Vec::new();
        json_bytes.resize(json_length as usize, 0);
        reader.read_exact(&mut json_bytes)?;
        let json_string = String::from_utf8(json_bytes)?;

        let mut chunk0: Value = Vrm::upgrade_chunk0(serde_json::from_str(&json_string)?);
        chunk0 = reduce_vroid(chunk0);
        chunk0 = clean(chunk0);
        let (chunk0, relocator) = relocate_buffers(chunk0);
        let chunks = relocator.relocate(reader, length - 20 - json_length)?;

        Ok(Vrm {
            version,
            total_chunk_bytes: relocator.total_chunk_bytes(),
            chunk0,
            chunks,
        })
    }
}
